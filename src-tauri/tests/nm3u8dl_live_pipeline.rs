//! 实跑集成测试：真实 N_m3u8DL-RE 进程 + 真实管道 + GBK 解码 + Nm3u8dlSession
//!
//! 精确复制生产管线（ProcessManager::spawn_reader + decode_output +
//! EngineSession::parse_chunk），验证进度事件能否从真实进程输出中解析出来。
//!
//! 依赖本地工具二进制与网络（代理），缺失时跳过而非失败。

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use streamgrab_lib::domain::download::{DownloadEngine, EngineEvent};
use streamgrab_lib::infrastructure::engines::nm3u8dl::Nm3u8dlEngine;

const EXE: &str = r"C:\Users\ZYB33\AppData\Roaming\com.streamgrab.app\tools\N_m3u8DL-RE.exe";
const FFMPEG: &str = r"C:\Users\ZYB33\AppData\Roaming\com.streamgrab.app\tools\ffmpeg-master-latest-win64-gpl-shared\bin\ffmpeg.exe";

/// 与 infrastructure/process/manager.rs::decode_output 完全一致的解码逻辑
fn decode_output(buf: &[u8]) -> String {
    use encoding_rs::GBK;
    let (decoded, _, had_errors) = GBK.decode(buf);
    if !had_errors {
        return decoded.into_owned();
    }
    String::from_utf8_lossy(buf).into_owned()
}

#[test]
#[ignore = "需要本地 N_m3u8DL-RE/ffmpeg 二进制与网络（代理）；手动运行：cargo test --test nm3u8dl_live_pipeline -- --ignored --nocapture"]
fn live_pipeline_emits_progress_events() {
    if !std::path::Path::new(EXE).exists() || !std::path::Path::new(FFMPEG).exists() {
        eprintln!("SKIP: tool binary not found on this machine");
        return;
    }

    let tmp = std::env::temp_dir().join("sg_live_pipeline_test");
    let _ = std::fs::create_dir_all(&tmp);

    let mut cmd = Command::new(EXE);
    cmd.args([
        "http://playertest.longtailvideo.com/adaptive/oceans_aes/oceans_aes.m3u8",
        "--save-dir",
        tmp.to_str().unwrap(),
        "--tmp-dir",
        tmp.to_str().unwrap(),
        "--auto-select",
        "--skip-merge",
        "--no-date-info",
        "--download-retry-count",
        "1",
        "--ffmpeg-binary-path",
        FFMPEG,
        "--custom-proxy",
        "http://127.0.0.1:7897",
        "--custom-range",
        "0-8s",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // 默认带 CREATE_NO_WINDOW（与 StreamGrab ProcessManager 一致）；
        // SG_TEST_NO_FLAG=1 时去掉该标志做对照
        if std::env::var("SG_TEST_NO_FLAG").is_err() {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
    }

    let mut child = cmd.spawn().expect("failed to spawn N_m3u8DL-RE");
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // 与 spawn_reader 一致：按 \n 切分 + decode_output + 回调
    fn spawn_reader<R: BufRead + Send + 'static>(
        name: &'static str,
        reader: R,
        tx: mpsc::Sender<String>,
    ) {
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = Vec::new();
            let mut chunks = 0usize;
            let mut bytes = 0usize;
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) => {
                        eprintln!("[reader:{name}] EOF after {chunks} chunks, {bytes} bytes");
                        break;
                    }
                    Ok(n) => {
                        chunks += 1;
                        bytes += n;
                        eprintln!(
                            "[reader:{name}] chunk#{chunks} {n}B head={:?}",
                            &buf[..n.min(60)]
                        );
                        let _ = tx.send(decode_output(&buf));
                    }
                    Err(e) => {
                        eprintln!("[reader:{name}] ERROR {e} after {chunks} chunks");
                        break;
                    }
                }
            }
        });
    }

    let (tx, rx) = mpsc::channel::<String>();
    spawn_reader("OUT", BufReader::new(stdout), tx.clone());
    spawn_reader("ERR", BufReader::new(stderr), tx.clone());
    drop(tx);

    let mut session = Nm3u8dlEngine::new().new_session();
    let mut progress_events = 0usize;
    let mut log_events = 0usize;
    let mut last_pct = -1;
    let mut max_pct = -1;

    // 与 download.rs 一致的事件处理（parse_chunk 与 finalize 两阶段复用）
    let mut handle = |ev: EngineEvent| match ev {
        EngineEvent::Progress { data } => {
            progress_events += 1;
            max_pct = max_pct.max(data.overall_percent);
            if data.overall_percent != last_pct {
                last_pct = data.overall_percent;
                println!(
                    "[progress] {}% segs={}/{} speed={} size={} action={}",
                    data.overall_percent,
                    data.downloaded_segments,
                    data.total_segments,
                    data.speed,
                    data.downloaded_size,
                    data.current_action
                );
            }
        }
        EngineEvent::Status { action } => println!("[status] {action}"),
        EngineEvent::Log { level, message } => {
            log_events += 1;
            if log_events <= 10 {
                println!("[log:{level}] {message}");
            }
        }
    };

    while let Ok(chunk) = rx.recv_timeout(Duration::from_secs(180)) {
        for ev in session.parse_chunk(&chunk) {
            handle(ev);
        }
    }
    // EOF 冲刷：与 download.rs on_complete 的 finalize 调用对齐
    for ev in session.finalize() {
        handle(ev);
    }

    let status = child.wait().unwrap();
    println!(
        "exit={status:?} progress_events={progress_events} log_events={log_events} max_pct={max_pct}"
    );

    assert!(status.success(), "N_m3u8DL-RE exited with error");
    assert!(
        progress_events > 0,
        "未从真实进程输出中解析出任何进度事件（生产 bug 复现）"
    );
    assert_eq!(max_pct, 100, "最终进度应达到 100%");
}
