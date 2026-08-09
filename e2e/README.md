# StreamGrab E2E 测试（chrome-devtools MCP 驱动）

## 是什么

48 个端到端用例，在**真实 Chrome 浏览器**里以真实交互跑完前端全部功能链路：
应用启动 / 标题栏 / 托盘 / 中断恢复 / 添加任务向导（单条、批量、重复、解析失败、
流选择、定时开始、拖拽、最近目录）/ 任务列表（Tab、搜索、排序、详情、右键菜单、
删除、清除、开始全部）/ 下载生命周期（进度、日志、完成、暂停/继续/停止、出错重试、
重启恢复、定时任务）/ 设置中心（分区、i18n、主题、路径、滑块、开关、重置、导出、
工具检测与更新、FFmpeg 参数）/ 任务预设 CRUD / 剪贴板监控。

不依赖 Playwright 等浏览器测试框架：浏览器由 **chrome-devtools-mcp 服务器**驱动，
测试脚本通过其 stdio/JSON-RPC 调用 `navigate_page` / `evaluate_script` / `press_key`
等工具完成导航、交互与断言。

## 架构

```
e2e/
├── run-e2e.mjs            # 入口：启动 Vite(注入 mock) + chrome-devtools-mcp，执行全部用例
├── runner-lib.mjs         # MCP stdio 客户端 + 页面驱动器（文本断言/点击/填写/键盘/mock 控制）
├── support/tauri-mock.js  # 浏览器内 Tauri bridge mock（假后端，命令契约与 src/services 对应）
└── tests/                 # 用例：app-shell / add-task / task-management /
                           #       download-lifecycle / settings / presets / clipboard
```

- **Tauri mock 注入**：`vite.config.ts` 的 `e2eMockPlugin()` 仅在
  `VITE_E2E_MOCK=1` 时把 `tauri-mock.js` 注入 `index.html`（正常开发/打包不受影响）。
  该脚本在应用脚本前定义 `window.__TAURI_INTERNALS__`，并实现内存版“假后端”
  （任务/设置/预设 CRUD、下载事件、插件命令），数据形状与 `src/domain/*` 一致。
- **用例隔离**：每个用例通过 URL `?e2e_seed=<base64 JSON>` 传入种子数据；
  用例之间清空 `localStorage`/`sessionStorage`。刷新页面时 mock 从
  `sessionStorage` 恢复状态，用于模拟“应用重启后数据持久化”。
- **网络封闭**：启动自动更新检查的 GitHub 请求由 chrome-devtools-mcp 的
  `--blocked-url-pattern` 阻断，用例不依赖外网。

## 运行

```bash
npm run test:e2e
```

环境变量：

| 变量 | 说明 | 默认 |
| --- | --- | --- |
| `E2E_PORT` | Vite dev server 端口 | `5173` |
| `E2E_CHROME_PATH` | Chrome/Edge 可执行文件路径 | 自动探测（含 Puppeteer 缓存） |

自动探测顺序：`E2E_CHROME_PATH` → Windows/macOS/Linux 常见安装路径 →
`~/.cache/puppeteer/chrome/*`（CI 用 `npx @puppeteer/browsers install chrome@stable`）。

## 新增用例

1. 在 `e2e/tests/` 新建或追加 `test("描述", async (d) => {...})`（描述即需求规格）。
2. `d.resetAndGo(seed, path)` 开新页；`d.assertText/assertNoText/assertEval` 断言；
   `d.clickText/clickTitle/clickTaskAction/clickSwitch/fillByPlaceholder/selectOption` 交互；
   `d.mockEmit/mockState/mockCallsOf` 驱动/校验假后端。
3. 在 `e2e/run-e2e.mjs` 引入新模块。

## 测试期间发现并修复的真实缺陷

1. **设置开关全部失效**（reka Switch）：`reka-ui` 的 `SwitchRoot` 只 emit
   `update:modelValue`，而项目 7 处均监听 `@update:checked`（SettingSwitch、
   定时开始、仅下载字幕、请求头开关、混流导入开关、广告关键词开关、预设仅字幕），
   点击开关实际不会更新任何设置。已统一改为 `:model-value` + `@update:model-value`。
2. **托盘创建失败提示丢失错误信息**：`t("messages.trayWarning", fallback).replace("{error}", ...)`
   中 vue-i18n 已把 `{error}` 当插值占位符剥离，`replace` 永远找不到目标，
   提示显示为“（）”。已改为 `t(key, { error }, fallback)` 直接传命名参数。
3. **toast 互斥**：`TOAST_LIMIT=1`，连续多个 toast 时后者覆盖前者（如“开始下载”
   被“已添加 1 个任务”顶掉）。用例按此约束断言最后一个 toast 或直接断言状态/命令。

## 覆盖边界

- 本套件覆盖**前端全流程**；Rust 命令/引擎行为由 `cargo test` 覆盖。
- 真实下载执行（N_m3u8DL-RE/FFmpeg）不在此套件内，由
  `src-tauri/tests/nm3u8dl_live_pipeline.rs`（`#[ignore]`，需本地工具+网络）覆盖。
