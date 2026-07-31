# 添加任务暂存层设计 — 主从详情式逐条配置

> 日期：2026-08-01
> 状态：已确认，待实现计划
> 关联文档：`docs/design/07-tool-config-architecture.md`、`docs/design/06-feature-status.md`

## 1. 背景与问题

当前 `AddTaskDialog.vue`（876 行）把「输入」与「全部配置」混在同一弹窗，存在三个真问题：

1. **引擎专属选项对直链是骗人的（无歧义被破坏）**：限速、下载范围、容器格式、字幕格式、仅下载字幕——这些 `TaskOverrides` 字段**只有 N_m3u8DL-RE 引擎会消费**。给一个 `.mp4` 直链设「容器格式=MKV」「仅下载字幕」是完全静默的无操作。
2. **多链接被一锅烩**：所有行共用同一套 `overrides`，无法逐条给不同文件名/保存位置/流选择；流选择器只在「单链接+流媒体」时触发，批量场景完全绕开。
3. **职责混杂**：弹窗既管输入编排，又持有所有引擎字段 `ref`，违反高内聚低耦合。

## 2. 引擎能力事实（来自后端 `args.rs`）

### 2.1 N_m3u8DL-RE（处理 HLS / DASH / MSS 流媒体）——能力丰富

- 通用：保存位置、文件名、临时目录
- 下载：线程数、重试、超时、限速 `-R`
- 流选择：`--auto-select`、`-sv/-sa/-ss` 选流、`-dv/-da/-ds` 弃流
- 混流：`-M format=muxer`、`--skip-merge`
- 网络：代理、自定义请求头、base-url、append-url-params
- 字幕：`--sub-format`、`--sub-only`、`--auto-subtitle-fix`
- 解密：`--key`、密钥文本文件、解密引擎、实时解密、自定义 HLS 加密参数
- 直播：`--live-perform-as-vod`、实时合并、保留分片、pipe-mux、VTT 修正、录制上限、等待、取数
- 范围：`--custom-range`

### 2.2 FFmpeg（处理 HTTP 直链视频 httpVideo）——能力极简

- 网络：`-user_agent`、`-headers`（Referer）、`-reconnect` 重连
- 流拷贝：`-c copy`（固定，无编码选择）
- 覆盖：`-y` / `-n`
- 进度：`-progress pipe:2`

**关键结论**：FFmpeg 的可调项（UA、Referer、重连、覆盖）全部在全局 `FfmpegConfig`，**不在 `TaskOverrides`**——意味着直链任务**几乎没有任何任务级可调项**，只剩文件名/保存位置/定时。直链行天然就该是「瘦」的。

## 3. 设计方案：主从详情式暂存（方案 A）

两个物理分离的层次，贴「层次感、避免信息过载、保持焦点与引导感」原则。

### 3.1 组件边界

| 组件 | 职责 | 依赖 |
|---|---|---|
| `AddTaskDialog.vue` | 弹窗外壳 + 二态切换状态机（list ↔ focus）+ 批次默认值 + 提交编排 | 下属三个子组件 |
| `TaskStagingList.vue`（新） | **第一层**：粘贴框 + 紧凑行清单渲染 + 批次公共默认（保存位置默认/预设/自动开始）。每行只读展示文件名、类型徽章、就绪状态 | 暂存条目数据 |
| `LinkConfigPanel.vue`（新） | **第二层**：单条链接的聚焦配置面板。**按检测到的 `UrlType` 动态渲染**引擎专属选项 + 内联流选择 | 暂存条目、`parseUrl`、流选择器 |
| `StreamSelector.vue`（复用） | 流选择，从对话框级下沉到聚焦面板内联 | 不变 |

**拆分原则**：`AddTaskDialog` 不再持有任何引擎专属字段 `ref`。这些只活在 `LinkConfigPanel`，且只在该行类型匹配时渲染——这是消除「直链设 MKV 是无操作」歧义的根本手段。

### 3.2 数据流

`AddTaskDialog` 持有 `StagedLink[]`。粘贴 → 生成数组 → 传给 `TaskStagingList`。点行 → 把该条对象传给 `LinkConfigPanel`，面板就地修改对象属性（响应式），关闭即回退列表，行变「就绪」。提交时遍历数组逐条建任务。

### 3.3 单链接零跳转

当 `StagedLink[]` 长度为 1 时，列表与聚焦面板合并显示（行即面板顶部），不强制点入；长度 >1 才启用「列表→点行进入」二段式。

## 4. 选项可见性规则与数据模型

### 4.1 「前端不碰 CLI」的边界

前端只持有两样东西：

1. **`UrlType`**：来自 `detectUrlType`，是前端唯一可见的「能力开关」。前端据此决定渲染哪些配置组，但不决定这些字段最终拼成什么参数（后端引擎的事）。
2. **`TaskOverrides` 字段集**：前端只收集「覆盖了什么」，不收集「怎么拼命令」。`buildOverrides()` 逻辑保留但从对话框级下沉到逐条。

后端新增工具/选项时，前端只在这张可见性表加一行，不碰引擎。

### 4.2 可见性表（前端唯一的「引擎知识」来源，集中一处）

| 配置组 | HLS / DASH / MSS（流媒体） | 直链 httpVideo | 通用 |
|---|:---:|:---:|:---:|
| 文件名 | ✓ | ✓ | 通用 |
| 保存位置 | ✓ | ✓ | 通用 |
| 定时开始 | ✓ | ✓ | 通用 |
| 限速 / 下载范围 | ✓ | — | 流媒体 |
| 容器格式 / 字幕格式 / 仅字幕 | ✓ | — | 流媒体 |
| 流选择（解析后） | ✓ | — | 流媒体 |
| 任务级解密密钥 | ✓ | — | 流媒体 |

直链行只有通用三件——FFmpeg 在 `TaskOverrides` 上几乎无可调项的直接映射。表集中放在 `LinkConfigPanel`（或 `linkOptionVisibility.ts` 常量），单一来源。

### 4.3 数据模型

```ts
/** 一条暂存链接 */
interface StagedLink {
  id: string;                 // 前端临时 id
  url: string;
  detectedType: UrlType | null;
  fileName: string;           // 自动提取，可改
  saveDir: string;            // 空 = 继承批次默认
  overrides: TaskOverrides;   // 仅流媒体行会被填充
  status: 'pending' | 'parsed' | 'ready' | 'invalid';
  streamInfo?: StreamInfo;    // 解析后缓存
}
```

- **批次默认**（`AddTaskDialog` 持有，不进 `StagedLink`）：`batchSaveDir` / `batchPresetId` / `autoStart`。是「这批的基线」，不是任务字段。
  - `batchSaveDir`：本批保存位置基线，空则回退全局 `settingsStore.defaultSaveDir`。
  - `batchPresetId`：本批预设，仅作「初值提供者」——把预设产出的 `TaskOverrides` 在生成 `StagedLink` 时拷进每个**流媒体行**的 `overrides` 作为初值；直链行不沾（预设字段对流媒体无意义）。行内可再改，**不**逐行另选预设。
  - `autoStart`：初始值取全局 `settingsStore.autoStartDownload`，弹窗内可逐批切换；不写回全局设置。
- **继承规则**（提交时计算，不在面板里算）：`effectiveSaveDir = link.saveDir || batchSaveDir || globalSaveDir`。面板输入框 placeholder 显示「将使用：批次默认 / 全局默认」，不持久化继承值。

### 4.4 高内聚低耦合

- `TaskStagingList` 只管渲染 + `emit('select', id)`，**不碰 overrides**。
- `LinkConfigPanel` 只改自己那条 `StagedLink` 的字段，**不读批次默认**（提交时合并的事）。
- `AddTaskDialog` 是唯一知道「批次默认 + 逐条 + 全局」三者合并规则的组件，是编排者。
- 合并逻辑抽成纯函数 `resolveLinkToTask(link, batchDefaults, globalDefaults)`，可单测，不依赖 Vue。

## 5. 交互流程

### 5.1 状态机

```text
open → [粘贴框] → 解析生成 StagedLink[]
                        │
            ┌───────────┴───────────┐
        len==1                   len>1
        合并态：                   列表态：
        行=面板顶部                 点行 → focus 态
        直接编辑                   编辑完回列表 → 行变 ready
                        │
                  全部 ready → [全部添加] → 落盘 → 关闭
```

二态：`view: 'list' | 'focus'`，派生量 `isSingle`。单链接时 `view` 强制 `'focus'` 且不显示返回按钮——零跳转。

### 5.2 解析时机（前端只发请求，不判定能力）

粘贴后只做轻量本地动作：
- 逐行 trim + URL 合法性 → 生成 `StagedLink[]`，`detectedType` 由 `detectUrlType` 即时填（纯前端字符串判定，已有）。
- `fileName` 由 `extractFileName` 即时填。
- **不**自动调 `parseUrl`——流解析是重操作（起 nm3u8dl/ffprobe 进程），按需触发。

进入某条流媒体行的聚焦面板时：若 `detectedType ∈ {HLS,DASH,MSS}` 且 `streamInfo` 未缓存，面板内显示「解析流」按钮；用户点后调 `parseUrl`，结果存回 `link.streamInfo` + `status='parsed'`。

### 5.3 自动解析 vs 手动解析

- **单链接流媒体**：进入聚焦面板即自动解析一次（沿用当前单链接体验，无额外点击）。
- **多链接**：进入某行聚焦面板时**不自动解析**，显示「解析流」按钮——批量下可能在多条间跳转，逐条自动解析会引发 N 个进程并发。手动触发更可控。

这条「多链接不自动解析」是关键的性能与焦点保护，避免后台风暴。

### 5.4 流选择的内联化

`StreamSelector` 不再是对话框级弹窗，而是 `LinkConfigPanel` 内解析完成后的折叠区：解析后展示视频/音频/字幕流列表供选，选中写回 `link.overrides.selection`。不再有「弹窗套弹窗」。

### 5.5 提交语义

- 顶部「全部添加」：遍历 `StagedLink[]`，对每条 `resolveLinkToTask` → 调 `addAndStartTask`（autoStart）或 `taskStore.addTask`。
- **就绪门槛**：`ready` 才提交；`pending`/`invalid` 的行被跳过并在行内标注（红色「未配置/无效」）。**不因某条没配好而阻塞其他条**。
- **URL 重复**：提交时逐条走现有 `checkUrlExists`，遇重复仍弹 `UrlDuplicateDialog`，确认后 `skipUrlCheck` 加入——逻辑与现有一致，仅搬到逐条循环里。
- **部分失败**：逐条 try/catch（现有 `addTasks` 已是这模式），统计成功数 toast，不回滚已成功条。

### 5.6 边界与空态

- 空粘贴：「全部添加」禁用，提示「请输入有效链接」。
- 全部 invalid：禁用提交，列表显示行级错误。
- 单链接直链（httpVideo）：聚焦面板只露通用三件，**没有「解析流」按钮**（直链无流可选），直接就绪可添加——FFmpeg 能力极简的直接映射。
- 拖拽/批量粘贴 TXT：现有 `handleDrop` 逻辑保留，搬进 `TaskStagingList`，解析后并入同一个 `StagedLink[]`。

## 6. 与现有体系衔接

### 6.1 三层配置归属（不重复造轮子）

| 归宿 | 内容 | 调整时机 |
|---|---|---|
| **设置中心（常驻，随时调）** | 线程/重试/超时、网络请求头/代理、解密密钥库、字幕默认、混流默认、直播策略、FFmpeg UA/Referer/重连/覆盖 | 任何时候改，下一个新任务生效 |
| **批次默认（暂存层第一层）** | `batchSaveDir` / `batchPresetId` / `autoStart` | 仅本次批次，关闭弹窗即弃 |
| **逐条覆盖（暂存层第二层）** | 文件名/保存位置/定时/限速/范围/容器/字幕/仅字幕/流选择/任务级密钥 | 随任务持久化为 `TaskOverrides` |

**关键不变式**：暂存层产出的最终就是 `TaskOverrides`，与现有 `addAndStartTask(url, fileName, saveDir, overrides)` / `taskStore.addTask` 契约**完全一致**。后端命令、引擎 `args.rs`、数据库 schema **一行不改**。这是「低耦合」最硬的验证——新增一整层交互，下游零改动。

### 6.2 渐进披露三层重新校准

加了暂存层后，原 `CLAUDE.md` 的 Level 1/2/3（URL/悬停/导航）更新为面向「添加任务」主线的新三层：

| 层级 | 内容 | 触发 |
|---|---|---|
| **L1 总览** | 粘贴框 + 紧凑行清单（文件名/类型徽章/就绪态）+ 批次默认三件 + 全部添加 | 打开弹窗即见 |
| **L2 聚焦** | 单条引擎专属配置（按类型动态）+ 内联流选择 | 点行进入 |
| **L3 全局** | 设置中心（引擎全局配置/工具管理） | 独立页导航 |

原则不变（默认只见 L1），但 L1 的「焦点」从「单个 URL 输入框」升级为「这批要下什么的清单总览」——每行只露四个元素，信息密度可控。

### 6.3 删除的东西（减负）

- 删 `AddTaskDialog` 里的 `maxSpeedInput / customRangeInput / muxFormatInput / subtitleFormatInput / subtitlesOnlyInput` 五个 `ref` 及其模板块——移入 `LinkConfigPanel` 且按类型渲染。
- 删对话框级 `showAdvanced` 折叠区（"更多选项"按钮及其包裹的文件名/保存位置/预设/定时/高级）——拆到 L1（批次默认：保存位置/预设/自动开始）与 L2（逐条：文件名/定时/引擎专属）。
- `handlePresetChange` 的副作用式覆盖改为「预设→每行 `overrides` 初值」，逻辑下沉。

净效果：`AddTaskDialog` 从 876 行、同时管输入+所有配置的大组件，瘦身成纯编排外壳；引擎专属知识集中到 `LinkConfigPanel` + 一张可见性表。

## 7. 任务追踪

完成后更新 `docs/design/06-feature-status.md` 中「添加任务」相关行为新的暂存层交互，并新增「多链接逐条配置」一行。

## 8. 不在本期范围

- 后端命令/引擎/数据库 schema 改动（本期纯前端）。
- 设置中心 UI 重构（仅引用，不改）。
- 历史页/任务卡片改造。
