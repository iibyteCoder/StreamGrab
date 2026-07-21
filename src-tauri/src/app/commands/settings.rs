//! 设置命令
//!
//! 应用设置（单行 JSON）+ 按工具分离的配置（tool_settings 表）+ 导入导出

use tauri::State;

use super::api;
use crate::domain::config::{AppSettings, FfmpegConfig, Nm3u8dlConfig};
use crate::domain::download::ToolId;
use crate::infrastructure::Database;
use crate::shared::AppError;

fn parse_tool_id(tool_id: &str) -> Result<ToolId, String> {
    tool_id.parse::<ToolId>().map_err(|e| e.to_string())
}

/// 获取应用设置
#[tauri::command(rename_all = "camelCase")]
pub async fn get_app_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    api(db.settings.load_app_settings())
}

/// 整体保存应用设置
#[tauri::command(rename_all = "camelCase")]
pub async fn save_app_settings(
    db: State<'_, Database>,
    settings: AppSettings,
) -> Result<(), String> {
    api(db.settings.save_app_settings(&settings))
}

/// 部分更新应用设置（递归合并），返回合并后的完整配置
#[tauri::command(rename_all = "camelCase")]
pub async fn patch_app_settings(
    db: State<'_, Database>,
    partial: serde_json::Value,
) -> Result<AppSettings, String> {
    api(db.settings.patch_app_settings(&partial))
}

/// 获取工具配置（按 tool_id）
#[tauri::command(rename_all = "camelCase")]
pub async fn get_tool_settings(
    db: State<'_, Database>,
    tool_id: String,
) -> Result<serde_json::Value, String> {
    let id = parse_tool_id(&tool_id)?;
    api(db.settings.load_tool_config::<serde_json::Value>(id))
}

/// 整体保存工具配置（先做类型校验再存储）
#[tauri::command(rename_all = "camelCase")]
pub async fn save_tool_settings(
    db: State<'_, Database>,
    tool_id: String,
    config: serde_json::Value,
) -> Result<(), String> {
    let id = parse_tool_id(&tool_id)?;
    api((|| {
        // 类型校验：拒绝结构错误的配置
        match id {
            ToolId::Nm3u8dl => {
                let _: Nm3u8dlConfig = serde_json::from_value(config.clone())
                    .map_err(|e| AppError::config(format!("N_m3u8DL-RE 配置格式错误: {e}")))?;
            }
            ToolId::Ffmpeg => {
                let _: FfmpegConfig = serde_json::from_value(config.clone())
                    .map_err(|e| AppError::config(format!("FFmpeg 配置格式错误: {e}")))?;
            }
        }
        db.settings.save_tool_config(id, &config)
    })())
}

/// 部分更新工具配置（递归合并），返回合并后的完整配置
#[tauri::command(rename_all = "camelCase")]
pub async fn patch_tool_settings(
    db: State<'_, Database>,
    tool_id: String,
    partial: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let id = parse_tool_id(&tool_id)?;
    api(db.settings.patch_tool_config(id, &partial))
}

/// 导出全部设置（应用 + 全部工具配置）
#[tauri::command(rename_all = "camelCase")]
pub async fn export_config(db: State<'_, Database>) -> Result<serde_json::Value, String> {
    api(db.settings.export_all())
}

/// 导入设置（从 JSON 文件，部分导入：只合并存在的字段）
#[tauri::command(rename_all = "camelCase")]
pub async fn import_config(db: State<'_, Database>, file_path: String) -> Result<(), String> {
    api((|| {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| AppError::config(format!("读取配置文件失败: {e}")))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::config(format!("配置文件不是有效 JSON: {e}")))?;
        db.settings.import_all(&value)
    })())
}
