//! 设置仓储
//!
//! 两类配置存储：
//! - `app_settings`：应用级配置（单行 JSON）
//! - `tool_settings`：按工具分行的配置 JSON（新增工具零 DDL）
//!
//! 保存支持整体写入与递归合并的部分更新（`patch_*`）

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::domain::config::AppSettings;
use crate::domain::download::ToolId;
use crate::domain::task::TaskRecord;
use crate::shared::{AppError, AppResult};

/// 设置仓储
#[derive(Clone)]
pub struct SettingsRepository {
    conn: Arc<Mutex<Connection>>,
}

fn lock(conn: &Mutex<Connection>) -> AppResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| AppError::database(format!("数据库锁获取失败: {e}")))
}

impl SettingsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ===== 应用设置 =====

    /// 加载应用设置（无记录时返回默认值）
    pub fn load_app_settings(&self) -> AppResult<AppSettings> {
        let conn = lock(&self.conn)?;
        let json: Option<String> = conn
            .query_row(
                "SELECT settings_json FROM app_settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match json {
            Some(j) => serde_json::from_str(&j).unwrap_or_default(),
            None => AppSettings::default(),
        })
    }

    /// 整体保存应用设置
    pub fn save_app_settings(&self, settings: &AppSettings) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "INSERT INTO app_settings (id, settings_json) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET settings_json = excluded.settings_json",
            [serde_json::to_string(settings)?],
        )?;
        Ok(())
    }

    /// 部分更新应用设置（递归合并），返回合并后的完整配置
    pub fn patch_app_settings(&self, partial: &Value) -> AppResult<AppSettings> {
        let mut current = serde_json::to_value(self.load_app_settings()?)?;
        deep_merge(&mut current, partial);
        let merged: AppSettings =
            serde_json::from_value(current).map_err(|e| AppError::config(e.to_string()))?;
        self.save_app_settings(&merged)?;
        Ok(merged)
    }

    // ===== 工具配置 =====

    /// 加载工具配置（无记录时返回默认值）
    pub fn load_tool_config<T: DeserializeOwned + Default>(&self, tool_id: ToolId) -> AppResult<T> {
        let conn = lock(&self.conn)?;
        let json: Option<String> = conn
            .query_row(
                "SELECT config_json FROM tool_settings WHERE tool_id = ?1",
                [tool_id.as_str()],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match json {
            Some(j) => serde_json::from_str(&j).unwrap_or_default(),
            None => T::default(),
        })
    }

    /// 整体保存工具配置
    pub fn save_tool_config<T: Serialize>(&self, tool_id: ToolId, config: &T) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "INSERT INTO tool_settings (tool_id, config_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(tool_id) DO UPDATE SET config_json = excluded.config_json, updated_at = excluded.updated_at",
            params![tool_id.as_str(), serde_json::to_string(config)?, TaskRecord::now()],
        )?;
        Ok(())
    }

    /// 部分更新工具配置（递归合并），返回合并后的 JSON
    pub fn patch_tool_config(&self, tool_id: ToolId, partial: &Value) -> AppResult<Value> {
        let conn = lock(&self.conn)?;
        let current: Option<String> = conn
            .query_row(
                "SELECT config_json FROM tool_settings WHERE tool_id = ?1",
                [tool_id.as_str()],
                |r| r.get(0),
            )
            .optional()?;
        drop(conn);

        let mut merged: Value = current
            .map(|j| serde_json::from_str(&j))
            .transpose()?
            .unwrap_or_else(|| Value::Object(Default::default()));
        deep_merge(&mut merged, partial);

        let typed: Value = merged.clone();
        self.save_tool_config(tool_id, &typed)?;
        Ok(merged)
    }

    // ===== 导入导出 =====

    /// 导出全部设置
    pub fn export_all(&self) -> AppResult<Value> {
        Ok(serde_json::json!({
            "app": serde_json::to_value(self.load_app_settings()?)?,
            "tools": {
                ToolId::Nm3u8dl.as_str(): serde_json::to_value(self.load_tool_config::<serde_json::Value>(ToolId::Nm3u8dl)?)?,
                ToolId::Ffmpeg.as_str(): serde_json::to_value(self.load_tool_config::<serde_json::Value>(ToolId::Ffmpeg)?)?,
            },
        }))
    }

    /// 导入设置（部分导入：只合并存在的字段）
    pub fn import_all(&self, value: &Value) -> AppResult<()> {
        if let Some(app) = value.get("app") {
            self.patch_app_settings(app)?;
        }
        if let Some(tools) = value.get("tools").and_then(|t| t.as_object()) {
            for (tool_id, config) in tools {
                let id: ToolId = tool_id
                    .parse()
                    .map_err(|e: AppError| AppError::config(e.to_string()))?;
                self.patch_tool_config(id, config)?;
            }
        }
        Ok(())
    }
}

/// 递归合并 JSON 对象：对象逐键递归，其余类型整体替换
fn deep_merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, value) in patch_map {
                deep_merge(base_map.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{Nm3u8dlConfig, ToolConfigs};
    use crate::infrastructure::db::schema;

    fn test_repo() -> SettingsRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        SettingsRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn app_settings_default_then_round_trip() {
        let repo = test_repo();
        let loaded = repo.load_app_settings().unwrap();
        assert_eq!(loaded, AppSettings::default());

        let mut modified = loaded;
        modified.minimize_to_tray = true;
        modified.default_save_dir = "D:/Downloads".into();
        repo.save_app_settings(&modified).unwrap();

        assert_eq!(repo.load_app_settings().unwrap(), modified);
    }

    #[test]
    fn tool_config_is_per_tool() {
        let repo = test_repo();
        let mut nm = Nm3u8dlConfig::default();
        nm.thread_count = 32;
        repo.save_tool_config(ToolId::Nm3u8dl, &nm).unwrap();

        let loaded_nm: Nm3u8dlConfig = repo.load_tool_config(ToolId::Nm3u8dl).unwrap();
        assert_eq!(loaded_nm.thread_count, 32);

        // FFmpeg 未保存 → 默认值，互不干扰
        let loaded_ff: crate::domain::config::FfmpegConfig =
            repo.load_tool_config(ToolId::Ffmpeg).unwrap();
        assert_eq!(loaded_ff, crate::domain::config::FfmpegConfig::default());
    }

    #[test]
    fn patch_deep_merges_nested_objects() {
        let repo = test_repo();
        repo.save_tool_config(ToolId::Nm3u8dl, &Nm3u8dlConfig::default())
            .unwrap();

        let patch = serde_json::json!({
            "thread_count": 16,
            "network": { "use_system_proxy": false, "custom_proxy": "http://127.0.0.1:7890" }
        });
        let merged = repo.patch_tool_config(ToolId::Nm3u8dl, &patch).unwrap();

        assert_eq!(merged["thread_count"], 16);
        assert_eq!(merged["network"]["use_system_proxy"], false);
        assert_eq!(merged["network"]["custom_proxy"], "http://127.0.0.1:7890");
        // 未 patch 的嵌套字段保留
        assert_eq!(merged["network"]["append_url_params"], false);
        assert_eq!(merged["retry_count"], 3);

        // 持久化生效
        let reloaded: Nm3u8dlConfig = repo.load_tool_config(ToolId::Nm3u8dl).unwrap();
        assert_eq!(reloaded.thread_count, 16);
        assert!(!reloaded.network.use_system_proxy);
    }

    #[test]
    fn export_import_round_trip() {
        let repo = test_repo();
        let mut app = AppSettings::default();
        app.language = crate::domain::config::Language::EnUs;
        repo.save_app_settings(&app).unwrap();
        let mut nm = Nm3u8dlConfig::default();
        nm.max_speed = "20M".into();
        repo.save_tool_config(ToolId::Nm3u8dl, &nm).unwrap();

        let exported = repo.export_all().unwrap();
        assert_eq!(exported["app"]["language"], "en-US");
        assert_eq!(exported["tools"]["nm3u8dl"]["max_speed"], "20M");

        // 导入到新仓储
        let repo2 = test_repo();
        repo2.import_all(&exported).unwrap();
        assert_eq!(
            repo2.load_app_settings().unwrap().language,
            crate::domain::config::Language::EnUs
        );
        let nm2: Nm3u8dlConfig = repo2.load_tool_config(ToolId::Nm3u8dl).unwrap();
        assert_eq!(nm2.max_speed, "20M");

        // ToolConfigs 默认值兜底
        let _ = ToolConfigs::default();
    }
}
