//! 配置表操作
//!
//! 使用 key-value 形式存储各配置模块

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;

/// 配置键名
pub const SETTINGS_KEYS: &[&str] = &[
    "general",
    "download",
    "mux",
    "network",
    "live",
    "decryption",
    "advanced",
    "ui",
];

/// 配置数据库管理器
pub struct SettingsDb {
    conn: Mutex<Connection>,
}

impl SettingsDb {
    /// 创建配置管理器
    pub fn new(conn: Connection) -> Result<Self, String> {
        let settings_db = Self {
            conn: Mutex::new(conn),
        };

        // 初始化默认配置
        settings_db.initialize_defaults()?;

        Ok(settings_db)
    }

    /// 初始化默认配置值
    fn initialize_defaults(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        for key in SETTINGS_KEYS {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, '{}')",
                params![key],
            )
            .map_err(|e| format!("Failed to initialize settings for {}: {}", key, e))?;
        }

        Ok(())
    }

    /// 加载所有配置
    pub fn load_all(&self) -> Result<HashMap<String, JsonValue>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to query settings: {}", e))?;

        let mut settings = HashMap::new();
        for row in rows {
            let (key, value_str) = row.map_err(|e| format!("Failed to read row: {}", e))?;
            let value: JsonValue = serde_json::from_str(&value_str)
                .unwrap_or(JsonValue::Object(serde_json::Map::new()));
            settings.insert(key, value);
        }

        Ok(settings)
    }

    /// 加载单个配置模块
    #[allow(dead_code)]
    pub fn load(&self, key: &str) -> Result<JsonValue, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query setting: {}", e))?;

        match result {
            Some(value_str) => {
                let value: JsonValue = serde_json::from_str(&value_str)
                    .map_err(|e| format!("Failed to parse setting JSON: {}", e))?;
                Ok(value)
            }
            None => Ok(JsonValue::Object(serde_json::Map::new())),
        }
    }

    /// 保存单个配置模块
    pub fn save(&self, key: &str, value: &JsonValue) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let value_str = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize setting: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value_str],
        )
        .map_err(|e| format!("Failed to save setting: {}", e))?;

        Ok(())
    }

    /// 批量保存配置
    pub fn save_all(&self, settings: &HashMap<String, JsonValue>) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;

        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        for (key, value) in settings {
            let value_str = serde_json::to_string(value)
                .map_err(|e| format!("Failed to serialize setting: {}", e))?;

            tx.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value_str],
            )
            .map_err(|e| format!("Failed to save setting {}: {}", key, e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }

    /// 重置单个配置模块为默认值
    pub fn reset(&self, key: &str) -> Result<(), String> {
        self.save(key, &JsonValue::Object(serde_json::Map::new()))
    }

    /// 重置所有配置为默认值
    pub fn reset_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM settings", [])
            .map_err(|e| format!("Failed to clear settings: {}", e))?;

        drop(conn);

        self.initialize_defaults()
    }
}
