//! 密钥库表操作
//!
//! 管理解密密钥

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 密钥记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRecord {
    pub id: String,
    pub kid: Option<String>,
    pub key: String,
    pub name: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// 密钥数据库管理器
pub struct KeysDb {
    conn: Mutex<Connection>,
}

impl KeysDb {
    /// 创建密钥管理器
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 加载所有密钥
    pub fn load_all(&self) -> Result<Vec<KeyRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, kid, key, name, created_at, last_used_at FROM keys ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let keys = stmt
            .query_map([], |row| {
                Ok(KeyRecord {
                    id: row.get(0)?,
                    kid: row.get(1)?,
                    key: row.get(2)?,
                    name: row.get(3)?,
                    created_at: row.get(4)?,
                    last_used_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query keys: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect keys: {}", e))?;

        Ok(keys)
    }

    /// 根据 ID 获取密钥
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Result<Option<KeyRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT id, kid, key, name, created_at, last_used_at FROM keys WHERE id = ?1",
                params![id],
                |row| {
                    Ok(KeyRecord {
                        id: row.get(0)?,
                        kid: row.get(1)?,
                        key: row.get(2)?,
                        name: row.get(3)?,
                        created_at: row.get(4)?,
                        last_used_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query key: {}", e))?;

        Ok(result)
    }

    /// 根据 KID 获取密钥
    #[allow(dead_code)]
    pub fn get_by_kid(&self, kid: &str) -> Result<Option<KeyRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT id, kid, key, name, created_at, last_used_at FROM keys WHERE kid = ?1",
                params![kid],
                |row| {
                    Ok(KeyRecord {
                        id: row.get(0)?,
                        kid: row.get(1)?,
                        key: row.get(2)?,
                        name: row.get(3)?,
                        created_at: row.get(4)?,
                        last_used_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query key by kid: {}", e))?;

        Ok(result)
    }

    /// 添加密钥
    pub fn add(&self, key: &KeyRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO keys (id, kid, key, name, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                key.id,
                key.kid,
                key.key,
                key.name,
                key.created_at,
                key.last_used_at
            ],
        )
        .map_err(|e| format!("Failed to insert key: {}", e))?;

        Ok(())
    }

    /// 更新密钥
    pub fn update(&self, key: &KeyRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let rows_affected = conn
            .execute(
                "UPDATE keys SET kid = ?1, key = ?2, name = ?3, last_used_at = ?4 WHERE id = ?5",
                params![key.kid, key.key, key.name, key.last_used_at, key.id],
            )
            .map_err(|e| format!("Failed to update key: {}", e))?;

        if rows_affected == 0 {
            return Err("Key not found".to_string());
        }

        Ok(())
    }

    /// 删除密钥
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let rows_affected = conn
            .execute("DELETE FROM keys WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete key: {}", e))?;

        if rows_affected == 0 {
            return Err("Key not found".to_string());
        }

        Ok(())
    }

    /// 清除所有密钥
    pub fn clear(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM keys", [])
            .map_err(|e| format!("Failed to clear keys: {}", e))?;

        Ok(())
    }

    /// 记录密钥使用时间
    pub fn record_usage(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE keys SET last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| format!("Failed to record key usage: {}", e))?;

        Ok(())
    }
}
