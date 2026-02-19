//! 对话框相关命令
//!
//! 处理文件/目录选择对话框

use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;

/// 文件过滤器
#[derive(Debug, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// 选择目录
#[tauri::command]
pub async fn select_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    log::info!("Opening directory picker");

    let folder_path = app.dialog().file().blocking_pick_folder();

    match folder_path {
        Some(path) => {
            let path_str = path.to_string();
            log::info!("Selected directory: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            log::info!("Directory selection cancelled");
            Ok(None)
        }
    }
}

/// 选择文件
#[tauri::command]
pub async fn select_file(
    app: tauri::AppHandle,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    log::info!("Opening file picker with filters: {:?}", filters);

    let mut dialog = app.dialog().file();

    // 添加文件过滤器
    if let Some(filter_list) = filters {
        for filter in filter_list {
            dialog = dialog.add_filter(
                filter.name,
                &filter
                    .extensions
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            );
        }
    }

    let file_path = dialog.blocking_pick_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string();
            log::info!("Selected file: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            log::info!("File selection cancelled");
            Ok(None)
        }
    }
}
