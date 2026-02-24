//! 环境配置加载器
//!
//! 从 TOML 配置文件加载环境特定配置

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// 应用环境配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库文件名或路径
    pub path: String,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别: DEBUG, INFO, WARN, ERROR, OFF
    pub level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::default_for_env("prod")
    }
}

impl AppConfig {
    /// 根据环境名称获取默认配置
    pub fn default_for_env(env: &str) -> Self {
        match env {
            "dev" | "development" => Self {
                database: DatabaseConfig {
                    path: "streamgrab_dev.db".to_string(),
                },
                logging: LoggingConfig {
                    level: "DEBUG".to_string(),
                },
            },
            _ => Self {
                database: DatabaseConfig {
                    path: "streamgrab.db".to_string(),
                },
                logging: LoggingConfig {
                    level: "INFO".to_string(),
                },
            },
        }
    }

    /// 加载配置
    ///
    /// 优先级：
    /// 1. 环境变量 STREAMGRAB_ENV 决定使用哪个配置文件
    /// 2. 如果配置文件不存在，使用默认值
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        // 1. 确定环境
        let env = std::env::var("STREAMGRAB_ENV").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "dev".to_string()
            } else {
                "prod".to_string()
            }
        });

        log::info!("Loading configuration for environment: {}", env);

        // 2. 配置文件路径
        let config_file = format!("config.{}.toml", env);
        let config_path = config_dir.join(&config_file);

        // 3. 如果配置文件不存在，使用默认值
        if !config_path.exists() {
            log::info!("Config file not found at {:?}, using defaults", config_path);
            return Ok(Self::default_for_env(&env));
        }

        // 4. 读取并解析配置文件
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", config_path, e))?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {:?}: {}", config_path, e))?;

        log::info!("Configuration loaded from {:?}", config_path);
        Ok(config)
    }

    /// 获取数据库完整路径
    pub fn get_database_path(&self, config_dir: &Path) -> PathBuf {
        // 如果路径是绝对路径，直接使用
        if std::path::Path::new(&self.database.path).is_absolute() {
            PathBuf::from(&self.database.path)
        } else {
            // 否则相对于配置目录
            config_dir.join(&self.database.path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_dev() {
        let config = AppConfig::default_for_env("dev");
        assert_eq!(config.database.path, "streamgrab_dev.db");
        assert_eq!(config.logging.level, "DEBUG");
    }

    #[test]
    fn test_default_config_prod() {
        let config = AppConfig::default_for_env("prod");
        assert_eq!(config.database.path, "streamgrab.db");
        assert_eq!(config.logging.level, "INFO");
    }

    #[test]
    fn test_default_config_unknown() {
        let config = AppConfig::default_for_env("unknown");
        assert_eq!(config.database.path, "streamgrab.db");
        assert_eq!(config.logging.level, "INFO");
    }
}
