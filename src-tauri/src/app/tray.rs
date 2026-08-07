//! 系统托盘模块
//!
//! 提供最小化到托盘功能

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime,
};

/// 系统托盘状态
///
/// 托盘创建失败时前端应提示用户——否则「关闭时最小化到托盘」会把窗口隐藏，
/// 却没有图标可供恢复，应用像「消失」一样。managed state 注入，供 `get_tray_status` 命令读取。
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStatus {
    /// 托盘是否创建成功
    pub created: bool,
    /// 创建失败原因（成功时为 None）
    pub error: Option<String>,
}

/// 创建系统托盘
pub fn create_tray<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<TrayIcon<R>, Box<dyn std::error::Error>> {
    // 创建菜单项
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    // 创建菜单
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    // 加载托盘图标
    let icon = load_tray_icon()?;

    // 创建托盘
    // 注意：`show_menu_on_left_click(false)` 让「左键单击托盘图标」触发 Click 事件（on_tray_icon_event → 显示窗口）。
    // 若设为 true（也是 tray-icon crate 的默认值），Windows 上左键只会弹出菜单，
    // 单击显示窗口的逻辑永远不触发，最小化到托盘后无法单击恢复。
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    // 显示主窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    // 退出应用
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // 单击托盘图标显示窗口
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(tray)
}

/// 加载托盘图标
fn load_tray_icon() -> Result<tauri::image::Image<'static>, Box<dyn std::error::Error>> {
    // 使用内嵌的图标数据
    // 32x32 PNG 图标
    let icon_bytes = include_bytes!("../../icons/32x32.png");
    let img = image::load_from_memory(icon_bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(tauri::image::Image::new_owned(
        rgba.into_raw(),
        width,
        height,
    ))
}
