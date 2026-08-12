/**
 * StreamGrab E2E — Tauri bridge mock（浏览器内运行的假后端）
 *
 * 通过 Playwright addInitScript 在应用脚本执行前注入，提供：
 *  - window.__TAURI_INTERNALS__            —— Tauri v2 IPC 桥（invoke/事件/窗口元数据）
 *  - window.__TAURI_EVENT_PLUGIN_INTERNALS__ —— listen/unlisten 所需的内部接口
 *  - window.__STREAMGRAB_MOCK__            —— 测试控制 API（播种/发事件/断言调用）
 *
 * 命令契约与 src/services/* 一一对应（数据形状镜像 src/domain/*）。
 * 状态默认值镜像 src/domain/config.ts 的 DEFAULT_* 常量。
 * 页面刷新时用 sessionStorage 恢复状态，模拟“后端 DB 持久化 + 应用重启”。
 */
(function () {
  "use strict";

  // ============================================================
  // 默认配置（镜像 src/domain/config.ts）
  // ============================================================

  var DEFAULT_APP_SETTINGS = {
    language: "zh-CN",
    auto_start_download: true,
    minimize_to_tray: false,
    check_update: true,
    default_save_dir: "",
    default_tmp_dir: "",
    theme: "dark",
    show_notification: true,
    clipboard_watch: false,
    log_level: "INFO",
    log_file_path: "",
    no_log: false,
    max_concurrent_tasks: 5,
  };

  var DEFAULT_NETWORK_CONFIG = {
    use_system_proxy: true,
    custom_proxy: null,
    base_url: null,
    append_url_params: false,
    headers: [],
  };

  var DEFAULT_DECRYPTION_CONFIG = {
    key_text_file: null,
    engine: "MP4DECRYPT",
    bin_path: null,
    real_time_decryption: false,
    custom_hls: {
      enabled: false,
      method: "UNKNOWN",
      key_type: "hex",
      key_value: null,
      iv_type: "hex",
      iv_value: null,
    },
    keys: [],
  };

  var DEFAULT_NM3U8DL_CONFIG = {
    path: "",
    thread_count: 8,
    retry_count: 3,
    timeout: 100,
    max_speed: "",
    auto_select: true,
    select_video: null,
    select_audio: null,
    select_subtitle: null,
    drop_video: null,
    drop_audio: null,
    drop_subtitle: null,
    check_segments_count: true,
    del_after_done: true,
    skip_merge: false,
    write_meta_json: false,
    binary_merge: false,
    concurrent_download: false,
    sub_only: false,
    sub_format: "SRT",
    auto_subtitle_fix: true,
    live_perform_as_vod: false,
    live_real_time_merge: false,
    live_keep_segments: true,
    live_pipe_mux: false,
    live_fix_vtt_by_audio: false,
    live_record_limit: null,
    live_wait_time: 0,
    live_take_count: 16,
    allow_hls_multi_ext_map: false,
    url_processor_args: null,
    no_date_info: false,
    use_ffmpeg_concat_demuxer: false,
    save_pattern: null,
    ad_keywords: [],
    mux_imports: [],
    network: DEFAULT_NETWORK_CONFIG,
    decryption: DEFAULT_DECRYPTION_CONFIG,
  };

  var DEFAULT_FFMPEG_CONFIG = {
    ffmpeg_path: "",
    ffprobe_path: "",
    mux_format: "mp4",
    muxer: "ffmpeg",
    mux_bin_path: null,
    mux_skip_subtitles: false,
    mux_keep_original: false,
    reconnect_attempts: 3,
    reconnect_delay: 5,
    retry_count: 3,
    timeout: 60,
    connection_timeout: 30,
    overwrite_existing: false,
    preserve_timestamps: true,
    user_agent: null,
    referer: null,
    http_proxy: null,
    cookies: null,
    auth: { username: "", password: "" },
    max_redirects: 8,
    reconnect_on_http_error: null,
    reconnect_delay_total_max: 256,
    respect_retry_after: true,
  };

  // ============================================================
  // 工具函数
  // ============================================================

  function clone(value) {
    return JSON.parse(JSON.stringify(value));
  }

  function isPlainObject(value) {
    return value !== null && typeof value === "object" && !Array.isArray(value);
  }

  function mergeDeep(target, source) {
    if (!isPlainObject(source)) return target;
    for (var key of Object.keys(source)) {
      var sv = source[key];
      if (sv === undefined) continue;
      if (isPlainObject(sv)) {
        target[key] = mergeDeep(
          isPlainObject(target[key]) ? target[key] : {},
          sv,
        );
      } else {
        target[key] = sv;
      }
    }
    return target;
  }

  function defaultState() {
    return {
      tasks: [],
      appSettings: clone(DEFAULT_APP_SETTINGS),
      nm3u8dl: clone(DEFAULT_NM3U8DL_CONFIG),
      ffmpeg: clone(DEFAULT_FFMPEG_CONFIG),
      presets: [],
      progressHistory: {},
      clipboardText: "",
      pickedDirectory: null,
      pickedFile: null,
      fileExistsMap: {},
      trayStatus: { created: true, error: null },
      tools: {
        nm3u8dl: {
          name: "N_m3u8DL-RE",
          installed: true,
          version: "0.5.0",
          exePath: "C:\\Tools\\N_m3u8DL-RE\\N_m3u8DL-RE.exe",
          dirPath: "C:\\Tools\\N_m3u8DL-RE",
          error: null,
        },
        ffmpeg: {
          name: "FFmpeg",
          installed: true,
          version: "7.0.2",
          exePath: "C:\\Tools\\ffmpeg\\bin\\ffmpeg.exe",
          dirPath: "C:\\Tools\\ffmpeg\\bin",
          error: null,
        },
        ffprobe: {
          name: "ffprobe",
          installed: true,
          version: "7.0.2",
          exePath: "C:\\Tools\\ffmpeg\\bin\\ffprobe.exe",
          dirPath: "C:\\Tools\\ffmpeg\\bin",
          error: null,
        },
      },
      toolReleases: {
        version: "9.9.9",
        downloadUrl: "https://example.com/download/tool.zip",
        filename: "tool.zip",
        publishedAt: "2026-01-01T00:00:00Z",
      },
      parseOverrides: {},
      maximized: false,
      calls: [],
      dbPath:
        "C:\\Users\\Test\\AppData\\Roaming\\com.streamgrab.app\\streamgrab.db",
    };
  }

  var STATE_KEY = "streamgrab:e2e-mock-state";

  function loadPersistedState() {
    try {
      var raw = window.sessionStorage.getItem(STATE_KEY);
      if (raw) return JSON.parse(raw);
    } catch {
      /* ignore */
    }
    return null;
  }

  function persist() {
    try {
      window.sessionStorage.setItem(STATE_KEY, JSON.stringify(state));
    } catch {
      /* ignore */
    }
  }

  function applyState(partial) {
    if (!isPlainObject(partial)) return;
    for (var key of Object.keys(partial)) {
      var sv = partial[key];
      if (sv === undefined) continue;
      if (isPlainObject(sv) && isPlainObject(state[key])) {
        mergeDeep(state[key], sv);
      } else {
        state[key] = clone(sv);
      }
    }
  }

  /** 从 URL ?e2e_seed=<base64(JSON)> 读取测试种子（e2e 模式由 Vite 注入本脚本） */
  function seedFromUrl() {
    try {
      var raw = new URLSearchParams(window.location.search).get("e2e_seed");
      if (!raw) return null;
      var bytes = Uint8Array.from(atob(raw), function (c) {
        return c.charCodeAt(0);
      });
      return JSON.parse(new TextDecoder().decode(bytes));
    } catch {
      return null;
    }
  }

  // 初始化状态：sessionStorage（刷新恢复，模拟真实应用重启）> seed（仅首次加载）> 默认
  var seed = seedFromUrl() || window.__MOCK_SEED__ || null;
  var persisted = loadPersistedState();
  var state = persisted ? persisted : defaultState();
  if (!persisted && seed) applyState(seed);
  persist();

  // e2e 模式下压掉「启动自动检查更新」：写入 24h 节流时间戳，避免每个测试页请求 GitHub
  if (seed) {
    try {
      window.localStorage.setItem(
        "streamgrab:lastUpdateCheck",
        new Date().toISOString(),
      );
    } catch {
      /* ignore */
    }
  }

  // ============================================================
  // 回调与事件系统（Tauri v2 listen/emit 语义）
  // ============================================================

  var callbackCounter = 0;
  var callbacks = new Map(); // id -> { cb, once }
  var eventCounter = 0;
  var listeners = new Map(); // event -> Map(eventId -> cbId)

  function transformCallback(cb, once) {
    var id = ++callbackCounter;
    callbacks.set(id, { cb: cb, once: !!once });
    return id;
  }

  function unregisterCallback(id) {
    callbacks.delete(id);
  }

  function registerListener(event, cbId) {
    var eventId = "e" + ++eventCounter;
    var map = listeners.get(event);
    if (!map) {
      map = new Map();
      listeners.set(event, map);
    }
    map.set(eventId, cbId);
    return eventId;
  }

  function unregisterListener(event, eventId) {
    var map = listeners.get(event);
    if (map) {
      var cbId = map.get(eventId);
      if (cbId !== undefined) {
        callbacks.delete(cbId);
        map.delete(eventId);
      }
      if (map.size === 0) listeners.delete(event);
    }
  }

  function emitEvent(event, payload) {
    var map = listeners.get(event);
    if (!map) return;
    var entries = Array.from(map.entries());
    for (var i = 0; i < entries.length; i++) {
      var eventId = entries[i][0];
      var cbId = entries[i][1];
      var entry = callbacks.get(cbId);
      if (!entry) continue;
      entry.cb({ event: event, id: eventId, payload: payload });
      if (entry.once) {
        callbacks.delete(cbId);
        map.delete(eventId);
      }
    }
  }

  // ============================================================
  // URL 类型检测（镜像 src/domain/url.ts）
  // ============================================================

  var VIDEO_EXTENSIONS = [
    ".mp4",
    ".mkv",
    ".avi",
    ".mov",
    ".wmv",
    ".flv",
    ".webm",
    ".m4v",
    ".ts",
    ".m2ts",
    ".mp3",
    ".m4a",
    ".aac",
    ".ogg",
    ".flac",
    ".wav",
  ];

  function detectUrlType(url) {
    var u = String(url).trim().toLowerCase();
    if (u.endsWith(".m3u8") || u.includes(".m3u8?")) return "hls";
    if (u.endsWith(".mpd") || u.includes(".mpd?")) return "dash";
    if (
      u.endsWith(".ism/manifest") ||
      u.includes(".ism/manifest?") ||
      u.endsWith(".isml/manifest") ||
      u.includes(".isml/manifest?")
    ) {
      return "mss";
    }
    for (var i = 0; i < VIDEO_EXTENSIONS.length; i++) {
      var ext = VIDEO_EXTENSIONS[i];
      if (u.endsWith(ext) || u.includes(ext + "?")) return "httpVideo";
    }
    return "unknown";
  }

  function defaultStreamInfo() {
    return {
      videos: [
        {
          id: "v1",
          bandwidth: 5000000,
          codecs: "avc1.640028",
          language: "und",
          name: "1080P",
          groupId: null,
          selected: true,
          resolution: "1920x1080",
          width: 1920,
          height: 1080,
          frameRate: 25,
          videoRange: "SDR",
        },
      ],
      audios: [
        {
          id: "a1",
          bandwidth: 192000,
          codecs: "mp4a.40.2",
          language: "zh",
          name: "中文",
          groupId: null,
          selected: true,
          channels: "2ch",
          sampleRate: 48000,
          isDefault: true,
        },
      ],
      subtitles: [
        {
          id: "s1",
          bandwidth: 0,
          codecs: "",
          language: "chi",
          name: "中文",
          groupId: null,
          selected: false,
          format: "vtt",
          isDefault: false,
          isForced: false,
        },
      ],
      duration: 3725,
      segmentCount: 745,
      isLive: false,
      isEncrypted: false,
    };
  }

  // ============================================================
  // invoke 分发器（自定义命令 + Tauri 插件命令）
  // ============================================================

  function fileInfo(path) {
    var parts = String(path).replace(/\\/g, "/").split("/");
    var fileName = parts[parts.length - 1] || "";
    var dot = fileName.lastIndexOf(".");
    return {
      path: String(path),
      fileName: fileName,
      extension: dot > 0 ? fileName.slice(dot + 1) : "",
      size: 734003200,
      modified: 1760000000000,
      exists:
        state.fileExistsMap[path] === undefined
          ? true
          : state.fileExistsMap[path],
    };
  }

  function record(cmd, args) {
    state.calls.push({ command: cmd, args: clone(args || {}), at: Date.now() });
  }

  function invoke(cmd, args) {
    args = args || {};
    record(cmd, args);

    switch (cmd) {
      // ========== 任务 CRUD ==========
      case "load_all_tasks":
        return clone(state.tasks);
      case "load_recoverable_tasks":
        return clone(
          state.tasks.filter(function (t) {
            return (
              [
                "paused",
                "downloading",
                "analyzing",
                "merging",
                "muxing",
              ].indexOf(t.status) >= 0
            );
          }),
        );
      case "get_task": {
        var found = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        return found ? clone(found) : null;
      }
      case "create_task": {
        var task = clone(args.task);
        var idx = state.tasks.findIndex(function (t) {
          return t.id === task.id;
        });
        if (idx >= 0) state.tasks[idx] = task;
        else state.tasks.push(task);
        return undefined;
      }
      case "update_task_status": {
        var t1 = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        if (t1) {
          t1.status = args.status;
          if (args.error !== undefined) t1.error = args.error;
          t1.updatedAt = new Date().toISOString();
          if (args.status === "downloading" && !t1.startedAt) {
            t1.startedAt = new Date().toISOString();
          }
          if (args.status === "completed" && !t1.completedAt) {
            t1.completedAt = new Date().toISOString();
          }
        }
        return undefined;
      }
      case "update_task_output_path": {
        var t2 = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        if (t2) t2.outputPath = args.outputPath;
        return undefined;
      }
      case "update_task_media_info": {
        var t3 = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        if (t3) t3.mediaInfo = args.mediaInfo;
        return undefined;
      }
      case "save_task_overrides": {
        var t4 = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        if (t4) t4.overrides = args.overrides;
        return undefined;
      }
      case "delete_task": {
        state.tasks = state.tasks.filter(function (t) {
          return t.id !== args.taskId;
        });
        return undefined;
      }
      case "clear_finished_tasks": {
        var before = state.tasks.length;
        state.tasks = state.tasks.filter(function (t) {
          return t.status !== "completed";
        });
        return before - state.tasks.length;
      }
      case "clear_all_tasks": {
        state.tasks = [];
        return undefined;
      }
      case "mark_active_tasks_interrupted": {
        var count = 0;
        state.tasks.forEach(function (t) {
          if (
            ["downloading", "analyzing", "merging", "muxing"].indexOf(
              t.status,
            ) >= 0
          ) {
            t.status = "paused";
            t.wasInterrupted = true;
            t.updatedAt = new Date().toISOString();
            count++;
          }
        });
        return count;
      }
      case "update_task_progress": {
        var t5 = state.tasks.find(function (t) {
          return t.id === args.taskId;
        });
        if (t5) {
          t5.progress = args.progress;
          t5.updatedAt = new Date().toISOString();
        }
        var hist = state.progressHistory[args.taskId] || [];
        hist.push({
          percent: args.progress.percent,
          speed: args.progress.speed,
          downloadedSize: args.progress.downloadedSize,
          recordedAt: Date.now(),
        });
        state.progressHistory[args.taskId] = hist.slice(-500);
        return undefined;
      }
      case "get_progress_history": {
        var samples = state.progressHistory[args.taskId] || [];
        if (args.limit && samples.length > args.limit)
          samples = samples.slice(-args.limit);
        return clone(samples);
      }
      case "clear_progress_history": {
        delete state.progressHistory[args.taskId];
        return undefined;
      }

      // ========== 设置 ==========
      case "get_app_settings":
        return clone(state.appSettings);
      case "patch_app_settings":
        mergeDeep(state.appSettings, args.partial || {});
        return clone(state.appSettings);
      case "get_tool_settings":
        return clone(args.toolId === "ffmpeg" ? state.ffmpeg : state.nm3u8dl);
      case "patch_tool_settings": {
        var target = args.toolId === "ffmpeg" ? state.ffmpeg : state.nm3u8dl;
        mergeDeep(target, args.partial || {});
        return clone(target);
      }
      case "export_config":
        return clone({
          app: state.appSettings,
          tools: { nm3u8dl: state.nm3u8dl, ffmpeg: state.ffmpeg },
        });
      case "import_config":
        return undefined;

      // ========== 下载 ==========
      case "start_download":
      case "stop_download":
      case "pause_download":
      case "resume_download":
        return undefined;
      case "parse_url": {
        var override = state.parseOverrides[args.url];
        if (override) {
          if (override.error) throw new Error(override.error);
          if (override.info) return clone(override.info);
        }
        return defaultStreamInfo();
      }
      case "detect_url_type":
        return detectUrlType(args.url);
      case "get_file_info":
        return fileInfo(args.path);
      case "analyze_media_file":
        return {
          resolution: "1920x1080",
          width: 1920,
          height: 1080,
          frameRate: 25,
          videoCodec: "h264",
          videoRange: "SDR",
          audioCodec: "aac",
          audioChannels: "2ch",
          audioLanguage: "zh",
          duration: 3725,
          segmentCount: null,
          isLive: false,
          isEncrypted: false,
          fileFormat: "mp4",
          fileSize: 734003200,
          bitRate: 1576960,
        };

      // ========== 预设 ==========
      case "load_presets":
        return clone(state.presets);
      case "save_preset": {
        var preset = clone(args.preset);
        var pi = state.presets.findIndex(function (p) {
          return p.id === preset.id;
        });
        if (pi >= 0) state.presets[pi] = preset;
        else state.presets.push(preset);
        return undefined;
      }
      case "delete_preset": {
        state.presets = state.presets.filter(function (p) {
          return p.id !== args.id;
        });
        return undefined;
      }

      // ========== 工具管理 ==========
      case "get_nm3u8dl_info":
        return clone(state.tools.nm3u8dl);
      case "get_ffmpeg_info":
        return clone(state.tools.ffmpeg);
      case "get_ffprobe_info":
        return clone(state.tools.ffprobe);
      case "get_nm3u8dl_latest_release":
        return clone(state.toolReleases);
      case "get_ffmpeg_latest_release":
        return clone(state.toolReleases);
      case "download_tool": {
        var base =
          args.tool === "FFmpeg" ? state.tools.ffmpeg : state.tools.nm3u8dl;
        base.installed = true;
        base.version = state.toolReleases.version;
        base.dirPath = args.targetDir || "C:\\Tools\\updated";
        base.exePath = (
          base.dirPath +
          "\\bin\\" +
          (args.tool === "FFmpeg" ? "ffmpeg.exe" : "N_m3u8DL-RE.exe")
        ).replace(/\\\\/g, "\\");
        return base.dirPath;
      }

      // ========== 系统 ==========
      case "select_directory":
        return state.pickedDirectory;
      case "select_file":
        return state.pickedFile;
      case "open_in_explorer":
      case "open_file_in_explorer":
      case "open_file_with_default":
      case "delete_file_or_folder":
        return undefined;
      case "file_exists":
        return state.fileExistsMap[args.path] === undefined
          ? true
          : state.fileExistsMap[args.path];
      case "get_db_path":
        return state.dbPath;
      case "get_tray_status":
        return clone(state.trayStatus);
      case "download_app_update":
        return args.savePath || "C:\\Downloads\\StreamGrab-setup.exe";
      case "run_installer":
        return undefined;

      // ========== Tauri 插件命令 ==========
      case "plugin:event|listen": {
        var eventId = registerListener(args.event, args.handler);
        return eventId;
      }
      case "plugin:event|unlisten":
        unregisterListener(args.event, args.eventId);
        return undefined;
      case "plugin:event|emit":
        emitEvent(args.event, args.payload);
        return undefined;
      case "plugin:clipboard-manager|read_text":
        return state.clipboardText;
      case "plugin:clipboard-manager|write_text":
        state.clipboardText = String(args.text || "");
        return undefined;
      case "plugin:notification|is_permission_granted":
        return true;
      case "plugin:window|is_maximized":
        return state.maximized;
      case "plugin:window|toggle_maximize":
        state.maximized = !state.maximized;
        return undefined;
      case "plugin:window|minimize":
      case "plugin:window|close":
      case "plugin:window|start_resize_dragging":
        return undefined;
      case "plugin:opener|open_url":
      case "plugin:process|exit":
      case "plugin:process|restart":
        return undefined;

      default:
        throw new Error("[tauri-mock] Unknown Tauri command: " + cmd);
    }
  }

  // ============================================================
  // 暴露 Tauri 内部接口
  // ============================================================

  window.__TAURI_INTERNALS__ = {
    invoke: function (cmd, args, _options) {
      try {
        var result = invoke(cmd, args);
        persist();
        return Promise.resolve(result);
      } catch (e) {
        persist();
        return Promise.reject(e);
      }
    },
    transformCallback: transformCallback,
    unregisterCallback: unregisterCallback,
    convertFileSrc: function (filePath, _protocol) {
      return filePath;
    },
    metadata: {
      currentWindow: { label: "main", __label: "main" },
      currentWebview: { label: "main", __label: "main" },
      windows: [{ label: "main" }],
      webviews: [{ label: "main" }],
    },
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: unregisterListener,
  };

  // ============================================================
  // 测试控制 API
  // ============================================================

  window.__STREAMGRAB_MOCK__ = {
    getState: function () {
      return clone(state);
    },
    setState: function (partial) {
      applyState(partial || {});
      persist();
    },
    reset: function () {
      state = defaultState();
      listeners = new Map();
      callbacks = new Map();
      persist();
    },
    emit: function (event, payload) {
      emitEvent(event, payload);
      persist();
    },
    getCalls: function () {
      return clone(state.calls);
    },
    clearCalls: function () {
      state.calls = [];
      persist();
    },
    setClipboardText: function (text) {
      state.clipboardText = String(text || "");
      persist();
    },
    setParseResult: function (url, result) {
      if (typeof result === "string") {
        state.parseOverrides[url] = { error: result };
      } else {
        state.parseOverrides[url] = { info: clone(result) };
      }
      persist();
    },
    setDialogResult: function (kind, value) {
      if (kind === "directory") state.pickedDirectory = value;
      else state.pickedFile = value;
      persist();
    },
    setFileExists: function (path, exists) {
      state.fileExistsMap[path] = !!exists;
      persist();
    },
    setTrayStatus: function (created, error) {
      state.trayStatus = { created: !!created, error: error || null };
      persist();
    },
    listenerCount: function (event) {
      var map = listeners.get(event);
      return map ? map.size : 0;
    },
  };
})();
