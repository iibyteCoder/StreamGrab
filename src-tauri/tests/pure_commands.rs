//! 纯命令测试：不依赖 Tauri 运行时 / DB / 原生工具，直接调用命令函数。
//!
//! 覆盖 `detect_url_type`（URL 分派）与 `get_file_info`（文件信息），
//! 零运行时依赖，跨平台稳定。

use streamgrab_lib::app::commands::download::{detect_url_type, get_file_info};

#[tokio::test]
async fn detect_url_type_classifies_streams_and_direct() {
    // HLS
    assert_eq!(
        detect_url_type("https://example.com/index.m3u8".into())
            .await
            .unwrap(),
        "hls"
    );
    // DASH
    assert_eq!(
        detect_url_type("https://example.com/manifest.mpd".into())
            .await
            .unwrap(),
        "dash"
    );
    // MSS
    assert_eq!(
        detect_url_type("https://example.com/video.ism/manifest".into())
            .await
            .unwrap(),
        "mss"
    );
    // HTTP 直链（FFmpeg 引擎）
    assert_eq!(
        detect_url_type("https://example.com/movie.mp4".into())
            .await
            .unwrap(),
        "httpVideo"
    );
    assert_eq!(
        detect_url_type("https://example.com/song.mp3?sig=1".into())
            .await
            .unwrap(),
        "httpVideo"
    );
    // 未知
    assert_eq!(
        detect_url_type("https://example.com/page".into())
            .await
            .unwrap(),
        "unknown"
    );
}

#[tokio::test]
async fn get_file_info_reports_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("sample.mp4");
    std::fs::write(&file, b"hello").unwrap();

    let info = get_file_info(file.to_string_lossy().into_owned())
        .await
        .unwrap();
    assert!(info.exists);
    assert_eq!(info.file_name, "sample.mp4");
    assert_eq!(info.extension, "mp4");
    assert_eq!(info.size, 5);
    assert!(info.modified.is_some());
}

#[tokio::test]
async fn get_file_info_errors_on_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let res = get_file_info(dir.path().join("nope.mp4").to_string_lossy().into_owned()).await;
    assert!(res.is_err(), "缺失文件应返回错误");
}
