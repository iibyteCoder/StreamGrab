/**
 * 简体中文语言包
 */
export default {
  // 通用
  common: {
    confirm: "确认",
    cancel: "取消",
    save: "保存",
    delete: "删除",
    edit: "编辑",
    add: "添加",
    remove: "移除",
    clear: "清除",
    reset: "重置",
    loading: "加载中...",
    success: "成功",
    error: "错误",
    warning: "警告",
    info: "提示",
    copy: "复制",
    paste: "粘贴",
    selectAll: "全选",
    search: "搜索",
    noData: "暂无数据",
  },

  // 导航
  nav: {
    home: "主页",
    settings: "设置",
    history: "历史记录",
  },

  // 主页
  home: {
    title: "StreamGrab",
    subtitle: "M3U8 视频流下载器",
    inputPlaceholder: "输入下载链接（支持 M3U8/MPD/MSS），每行一个",
    download: "下载",
    scheduledDownload: "定时下载",
    selectDate: "选择日期",
    selectTime: "选择时间",
    advancedOptions: "高级选项",
    startAll: "开始全部",
    clearCompleted: "清除已完成",
    total: "总计",
    completed: "已完成",
    progress: "进度",
    dragDropHint: "支持拖放文本链接或 TXT 文件",
  },

  // 任务
  task: {
    status: {
      pending: "等待中",
      analyzing: "解析中",
      downloading: "下载中",
      paused: "已暂停",
      completed: "已完成",
      failed: "失败",
      cancelled: "已取消",
    },
    actions: {
      start: "开始",
      pause: "暂停",
      resume: "继续",
      stop: "停止",
      retry: "重试",
      remove: "删除",
      viewLog: "查看日志",
    },
    unnamed: "未命名文件",
    remaining: "剩余",
  },

  // 设置
  settings: {
    title: "设置",
    autoSaveHint: "配置应用程序选项，更改会自动保存",

    // 标签页
    tabs: {
      general: "常规",
      templates: "模板",
      download: "下载",
      mux: "混流",
      network: "网络",
      decryption: "解密",
      live: "直播",
      advanced: "高级",
      ui: "界面",
    },

    // 常规设置
    general: {
      storage: "存储位置",
      storageDesc: "设置下载和临时文件的保存位置",
      saveDir: "下载目录",
      tmpDir: "临时目录",
      behavior: "应用行为",
      behaviorDesc: "配置应用程序的默认行为",
      language: "语言",
      autoStartDownload: "自动开始下载",
      autoStartDownloadDesc: "添加任务后自动开始下载",
      minimizeToTray: "最小化到托盘",
      minimizeToTrayDesc: "关闭窗口时最小化到系统托盘",
      checkUpdate: "检查更新",
      checkUpdateDesc: "启动时自动检查新版本",
      currentVersion: "当前版本",
      latestVersion: "最新版本",
      checkNow: "检查更新",
      checking: "检查中...",
      downloadUpdate: "下载更新",
    },

    // 下载设置
    download: {
      basic: "基本设置",
      threadCount: "下载线程数",
      retryCount: "重试次数",
      timeout: "超时时间（秒）",
      maxSpeed: "限速（KB/s，0 为不限速）",
      autoSelect: "自动选择最佳流",
      selectVideo: "视频流选择器",
      selectAudio: "音频流选择器",
      selectSubtitle: "字幕流选择器",

      streamExclude: "流排除",
      streamExcludeDesc: "使用正则表达式排除不需要的流",
      dropVideo: "排除视频流",
      dropAudio: "排除音频流",
      dropSubtitle: "排除字幕流",

      adFilter: "广告过滤",
      adFilterDesc: "过滤包含指定关键字的分片",

      subtitle: "字幕设置",
      subtitleFormat: "字幕格式",
      autoFixTimeline: "自动修正时间轴",
      downloadSubtitleOnly: "仅下载字幕",

      merge: "合并设置",
      autoMerge: "自动合并",
      binaryMerge: "二进制合并",
      deleteTemp: "删除临时文件",
      writeMetaJson: "写入元数据 JSON",
      concurrentDownload: "并发下载",
    },

    // 混流设置
    mux: {
      format: "输出格式",
      muxer: "混流器",
      ffmpegPath: "FFmpeg 路径",
      mkvmergePath: "MKVMerge 路径",
      keepOriginal: "保留原文件",
      externalMedia: "外部媒体导入",
      externalMediaDesc: "导入外部音频或字幕文件进行混流",
      addAudio: "添加音频",
      addSubtitle: "添加字幕",
    },

    // 网络设置
    network: {
      proxy: "代理设置",
      useSystemProxy: "使用系统代理",
      customProxy: "自定义代理",
      customProxyPlaceholder: "http://127.0.0.1:7890",
      headers: "请求头管理",
      headersDesc: "添加自定义 HTTP 请求头",
      baseUrl: "BaseURL 替换",
    },

    // 解密设置
    decryption: {
      keys: "密钥配置",
      keysDesc: "KID:KEY 格式，每行一个",
      keyFile: "密钥文件",
      engine: "解密引擎",
      realTimeDecryption: "实时解密",
      hlsCustomMethod: "HLS 自定义解密方法",
    },

    // 直播设置
    live: {
      mode: "直播模式",
      realtimeMerge: "实时合并",
      keepSegments: "保留分片",
      durationLimit: "录制时长限制",
      waitTime: "刷新等待时间（秒）",
      segmentCount: "分片数量",
    },

    // 高级设置
    advanced: {
      paths: "程序路径",
      n_m3u8dlPath: "N_m3u8DL-RE 路径",
      ffmpegPath: "FFmpeg 路径",
      mkvmergePath: "MKVMerge 路径",
      mp4decryptPath: "MP4Decrypt 路径",
      shakaPackagerPath: "Shaka Packager 路径",
      reset: "恢复默认设置",
    },

    // 界面设置
    ui: {
      appearance: "外观",
      theme: "主题",
      themeLight: "浅色",
      themeDark: "深色",
      themeSystem: "跟随系统",
      notification: "显示通知",
      notificationDesc: "下载完成时显示系统通知",
      clipboardWatch: "剪贴板监视",
      clipboardWatchDesc: "自动检测剪贴板中的 M3U8 链接",
    },
  },

  // 历史记录
  history: {
    title: "下载历史",
    empty: "暂无下载记录",
    reDownload: "重新下载",
    openFolder: "打开文件夹",
    clearAll: "清空历史",
  },

  // 流选择器
  streamSelector: {
    title: "选择流",
    videoStreams: "视频流",
    audioStreams: "音频流",
    subtitleStreams: "字幕流",
    resolution: "分辨率",
    codec: "编码",
    bitrate: "码率",
    language: "语言",
    fps: "帧率",
    channels: "声道",
    noStreams: "无可用流",
    loading: "正在解析...",
  },

  // 模板
  template: {
    title: "配置模板",
    presets: "预设模板",
    custom: "自定义模板",
    bestQuality: "最佳质量",
    quality1080p: "1080P",
    quality720p: "720P",
    audioOnly: "仅音频",
    createNew: "新建模板",
    editTemplate: "编辑模板",
    deleteConfirm: "确定要删除此模板吗？",
  },

  // 日志查看器
  logViewer: {
    title: "任务日志",
    empty: "暂无日志",
    clear: "清除日志",
    levels: {
      info: "信息",
      warn: "警告",
      error: "错误",
      debug: "调试",
    },
  },

  // Toast 消息
  messages: {
    urlDetected: "检测到下载链接",
    urlsDetected: "检测到 {count} 个下载链接",
    urlAdded: "已添加下载链接",
    urlsAdded: "已添加 {count} 个下载链接",
    taskAdded: "已添加任务",
    tasksAdded: "已添加 {count} 个任务",
    scheduledTaskAdded: "已添加定时任务",
    downloadStarted: "开始下载",
    downloadPaused: "下载已暂停",
    downloadResumed: "下载已恢复",
    downloadCancelled: "下载已取消",
    downloadCompleted: "下载完成",
    downloadFailed: "下载失败",
    taskRemoved: "任务已删除",
    settingsSaved: "设置已保存",
    settingsReset: "设置已恢复为默认值",
    clipboardUrlDetected: "已添加下载链接",
    clipboardUrlsDetected: "已添加 {count} 个下载链接",
    updateAvailable: "发现新版本 {version}，请前往 GitHub 下载",
    noUpdate: "当前已是最新版本",
    updateCheckFailed: "检查更新失败",
    downloaderNotFound: "N_m3u8DL-RE 未找到，请确保已安装并添加到 PATH",
  },
};
