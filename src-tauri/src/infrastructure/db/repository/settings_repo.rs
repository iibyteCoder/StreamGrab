//! 设置仓储
//!
//! `app_settings` 单行 JSON + `tool_settings` 按工具分行。
//! 读取总是返回**完整**配置（行缺失/损坏时填充类型默认值）；
//! patch 在完整配置上深合并并做类型校验——存储的 JSON 永远是完整良态配置。

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

use crate::domain::config::{AppSettings, FfmpegConfig, Nm3u8dlConfig};
use crate::domain::download::ToolId;
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
        let row: Option<String> = conn
            .query_row(
                "SELECT settings_json FROM app_settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match row {
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

    /// 部分更新应用设置：在完整配置上深合并 → 类型校验 → 保存，返回合并结果
    pub fn patch_app_settings(&self, partial: &Value) -> AppResult<AppSettings> {
        let mut merged = serde_json::to_value(self.load_app_settings()?)?;
        deep_merge(&mut merged, partial);
        let typed: AppSettings = serde_json::from_value(merged)
            .map_err(|e| AppError::config(format!("应用设置格式错误: {e}")))?;
        self.save_app_settings(&typed)?;
        Ok(typed)
    }

    // ===== 工具配置 =====

    /// 加载工具配置（无记录/损坏时返回类型默认值——永远是完整配置）
    pub fn load_tool_config<T: DeserializeOwned + Default>(&self, tool_id: ToolId) -> AppResult<T> {
        let conn = lock(&self.conn)?;
        let row: Option<String> = conn
            .query_row(
                "SELECT config_json FROM tool_settings WHERE tool_id = ?1",
                [tool_id.as_str()],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match row {
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
            params![
                tool_id.as_str(),
                serde_json::to_string(config)?,
                crate::domain::task::TaskRecord::now(),
            ],
        )?;
        Ok(())
    }

    /// 部分更新 N_m3u8DL-RE 配置：完整配置 + 深合并 + 类型校验 + 保存
    pub fn patch_nm3u8dl_config(&self, partial: &Value) -> AppResult<Nm3u8dlConfig> {
        self.patch_typed(ToolId::Nm3u8dl, partial)
    }

    /// 部分更新 FFmpeg 配置：完整配置 + 深合并 + 类型校验 + 保存
    pub fn patch_ffmpeg_config(&self, partial: &Value) -> AppResult<FfmpegConfig> {
        self.patch_typed(ToolId::Ffmpeg, partial)
    }

    /// 类型化 patch：在完整类型化配置上合并，保证存储的永远是完整良态配置
    fn patch_typed<T: DeserializeOwned + Serialize + Default>(
        &self,
        tool_id: ToolId,
        partial: &Value,
    ) -> AppResult<T> {
        let mut merged = serde_json::to_value(self.load_tool_config::<T>(tool_id)?)?;
        deep_merge(&mut merged, partial);
        let typed: T = serde_json::from_value(merged)
            .map_err(|e| AppError::config(format!("{} 配置格式错误: {e}", tool_id)))?;
        self.save_tool_config(tool_id, &typed)?;
        Ok(typed)
    }

    // ===== 导入导出 =====

    /// 导出全部设置（总是完整配置，空库也导出全量默认值）
    pub fn export_all(&self) -> AppResult<Value> {
        let app = self.load_app_settings()?;
        let nm3u8dl = self.load_tool_config::<Nm3u8dlConfig>(ToolId::Nm3u8dl)?;
        let ffmpeg = self.load_tool_config::<FfmpegConfig>(ToolId::Ffmpeg)?;
        Ok(json!({
            "app": app,
            "tools": {
                "nm3u8dl": nm3u8dl,
                "ffmpeg": ffmpeg,
            },
        }))
    }

    /// 导入设置（部分导入：在各工具的完整默认配置上深合并）
    pub fn import_all(&self, value: &Value) -> AppResult<()> {
        if let Some(app) = value.get("app") {
            self.patch_app_settings(app)?;
        }
        if let Some(tools) = value.get("tools") {
            if let Some(v) = tools.get("nm3u8dl") {
                self.patch_nm3u8dl_config(v)?;
            }
            if let Some(v) = tools.get("ffmpeg") {
                self.patch_ffmpeg_config(v)?;
            }
        }
        Ok(())
    }
}

/// 深合并 JSON 对象：对象逐键递归，其余类型整体替换
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
    use crate::infrastructure::db::schema;

    fn test_repo() -> SettingsRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        SettingsRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn empty_db_returns_full_defaults() {
        let repo = test_repo();
        // 空库：工具配置返回完整默认值而非 null
        let nm = repo
            .load_tool_config::<Nm3u8dlConfig>(ToolId::Nm3u8dl)
            .unwrap();
        assert_eq!(nm.thread_count, 8);
        assert!(nm.auto_select);
        let ff = repo
            .load_tool_config::<FfmpegConfig>(ToolId::Ffmpeg)
            .unwrap();
        assert_eq!(ff.timeout, 60);
        // 序列化不含 null 顶层
        let v = serde_json::to_value(&nm).unwrap();
        assert!(v.get("path").is_some());
    }

    #[test]
    fn patch_merges_onto_full_config() {
        let repo = test_repo();
        // 空库上只 patch 一个字段
        let merged = repo
            .patch_nm3u8dl_config(&json!({ "thread_count": 16 }))
            .unwrap();
        assert_eq!(merged.thread_count, 16);
        assert_eq!(merged.retry_count, 3); // 其余字段为完整默认值
        assert!(merged.auto_select);

        // 嵌套深合并：改 network 子对象的一个字段，其余保留
        let merged = repo
            .patch_nm3u8dl_config(&json!({ "network": { "use_system_proxy": false } }))
            .unwrap();
        assert!(!merged.network.use_system_proxy);
        assert!(merged.network.headers.is_empty());
        assert_eq!(merged.thread_count, 16);

        // 落盘的是完整配置：重新加载验证
        let reloaded = repo
            .load_tool_config::<Nm3u8dlConfig>(ToolId::Nm3u8dl)
            .unwrap();
        assert_eq!(reloaded.thread_count, 16);
        assert!(!reloaded.network.use_system_proxy);
        assert_eq!(reloaded.retry_count, 3);
    }

    #[test]
    fn patch_rejects_invalid_types() {
        let repo = test_repo();
        let result = repo.patch_nm3u8dl_config(&json!({ "thread_count": "not-a-number" }));
        assert!(result.is_err());
    }

    #[test]
    fn export_import_round_trip_on_empty_db() {
        let repo = test_repo();
        let exported = repo.export_all().unwrap();
        // 空库也导出完整默认值
        assert_eq!(exported["tools"]["nm3u8dl"]["thread_count"], 8);
        assert_eq!(exported["tools"]["ffmpeg"]["timeout"], 60);

        let repo2 = test_repo();
        repo2.import_all(&exported).unwrap();
        let nm = repo2
            .load_tool_config::<Nm3u8dlConfig>(ToolId::Nm3u8dl)
            .unwrap();
        assert_eq!(nm.thread_count, 8);
    }
}
