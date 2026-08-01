# 添加任务弹窗重设计

> 状态：设计已确认，待实现
> 日期：2026-08-01
> 影响范围：前端 `src/components/task/` 添加任务编排；后端任务契约**零改动**

## 1. 背景与问题

当前添加任务弹窗（`AddTaskDialog.vue`）是"主从详情式暂存外壳"：

- **L1 总览**（`TaskStagingList`）：粘贴框 + 批次公共默认（保存位置 / 预设 / 自动开始）+ 行清单
- **L2 聚焦**（`LinkConfigPanel`）：单条引擎类型驱动配置 + 内联流选择
- 单链接 → 直接进 L2；多链接 → 行清单，逐条点进 L2 配置，最后"全部添加"

暴露的问题：

1. **多链接时被强行要求设路径 / 预设**：路径本应有记忆、默认取最近一次，不该出现在每条配置里。
2. **多链接堆成一张行清单，信息过载**；且每条配置丢到二级页"逐个设置再统一开始"，范式本身就错。
3. **单链接与多链接形态割裂**：两套视图、两套心智，违背"单/多应一致"的直觉。

## 2. 设计目标与原则

对齐 `CLAUDE.md` 的核心设计原则：

- **渐进式披露**：L1 常用直出，L2/L3 高级收起。任何时刻屏幕上只呈现当前必要信息。
- **80/20 法则**：粘贴 → 默认值入库是 80% 路径，必须最短；流选择 / 限速 / 容器等高级项是 20%，折叠隐藏。
- **单/多同构**：单链接与多链接走完全相同的流程，仅迭代次数不同，无分叉界面。

参考经典下载管理器（IDM / FDM / JDownloader / Motrix）的成熟范式：**薄入口 → 解析 → 逐条聚焦确认 + 批量默认逃生舱**。

## 3. 整体流程：三段式向导

```
步骤 1 粘贴          步骤 2 解析              步骤 3 逐条配置
(只有粘贴框)   →    (自动, 有进度)     →     (一条一条过)
零决策             检测类型/提取文件名         点"添加"→下一条
                   流媒体拉流信息             或"全部添加"批量入库
```

对话框状态机（`AddTaskDialog` 持有）：

```
paste ──(点击"解析并添加")──▶ parsing ──(解析完成)──▶ config[i] ──▶ done
                                                        │
                                              添加→ / 跳过 / 全部添加
```

- `paste`：一个 textarea + "解析并添加"按钮。无路径、无预设、无开关。
- `parsing`：loading 态，"正在解析 N 个链接…(k/N)"。所有链接并行解析。
- `config`：对第 `i` 条有效链接渲染配置卡；"添加"使 `i++`；"全部添加"批量入库剩余；"跳过"使 `i++` 且不入库。
- `done`：toast 汇报 + 关闭。

**单 ≈ 多的答案**：单链接 = 该流程迭代一次；多链接 = 迭代 N 次。同一套界面，无 list/focus 分叉。

## 4. 步骤 1：粘贴

- 弹窗打开时只有一个 textarea 占满内容区，placeholder：`粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 直链）`。
- 底部一个主按钮"解析并添加"（禁用直到非空）。
- 支持拖拽文本投放（沿用现有 drag/drop）。
- Enter 触发解析；Esc 关闭。

## 5. 步骤 2：解析

点击"解析并添加"后：

1. 按行切分，trim，过滤出 `http://` / `https://` 开头的行，批内去重。
2. 对每条同步执行 `detectUrlType` + `extractFileName`。
3. 流媒体类型（`isStreamingType`）并行调用 `parseUrl` 拉取 `streamInfo`。
4. UI 显示进度"正在解析 N 个链接…(k/N)"。

**分流结果：**

- **无法识别的链接**（`detectUrlType` 返回 `unknown` 或空，即非 HLS/DASH/MSS/HTTP 直链）：在解析阶段剔除，不进配置队列。结束时 toast 汇报"X 个链接无法识别已跳过"。
- **流媒体解析失败**（`parseUrl` 返回空/报错）：**不剔除**，仍进配置队列，标记为解析失败态（见 §8.2）。失败 ≠ 无效。
- 其余：正常进入步骤 3。

## 6. 步骤 3：逐条配置卡（核心）

每条有效链接渲染**同一张配置卡**。字段自上而下按使用频率排列：

```
┌──────────────────────────────────────────────────┐
│  添加下载任务                    [1/5]      [×]  │
├──────────────────────────────────────────────────┤
│  链接                                            │
│  ┌──────────────────────────────────────────┐   │
│  │ https://cdn.example.com/ep01/index.m3u8 │   │  可编辑
│  └──────────────────────────────────────────┘   │
│  ● HLS 已解析                                    │  类型徽章 + 状态
│                                                  │
│  保存位置                                        │
│  ┌──────────────────────────────────┐   ┌────┐  │
│  │ D:\Media\番剧                    │   │ 📁 │  │
│  └──────────────────────────────────┘   └────┘  │
│   ⌄ 最近: D:\Media\番剧 · E:\Downloads          │  记忆下拉
│                                                  │
│  文件名                                          │
│  ┌──────────────────────────────────────────┐   │
│  │ ep01                                     │   │  自动提取, 可编辑
│  └──────────────────────────────────────────┘   │
│                                                  │
│  ▸ 高级设置                                      │  默认收起
├──────────────────────────────────────────────────┤
│  [跳过]                    [全部添加]   [添加 →] │
└──────────────────────────────────────────────────┘
```

### 6.1 字段语义

| 字段 | 位置 | 行为 |
|---|---|---|
| 链接 | L1 常显 | 可编辑输入框。编辑**只改提交的 URL 字符串**，不触发重解析；类型徽章/流信息保持原解析结果（见 §8.3）。 |
| 类型徽章 + 状态 | L1 常显 | `HLS`/`DASH`/`MSS`/`直链视频` + `已解析`/`解析失败`。一眼看清引擎归属。 |
| 保存位置 | L1 常显 | 输入框 + 浏览按钮 + 记忆下拉。默认值 = 最近记忆（见 §7）。 |
| 文件名 | L1 常显 | 自动从 URL 提取，可编辑。 |
| 高级设置 | L2 折叠 | 手风琴，默认收起，按工具类型动态渲染（见 §8.1）。 |

### 6.2 页码指示 `[i/N]`

替代"行清单"。任何时刻屏幕只有一条链接的信息（满足"不要列表同时出现"），页码提供总量感知。这是安装向导 / iOS 设置页的标准做法。

### 6.3 底部按钮

| 按钮 | 语义 | 可见性 |
|---|---|---|
| `添加 →` | 提交当前链接（按全局 `autoStart` 决定是否立即开始），推进到下一条 | 非末条 |
| `完成 ✓` | 提交当前链接并结束 | 末条（替代"添加 →"） |
| `全部添加` | 剩余链接全部按**当前默认值**批量入库（见 §6.4） | 多链接时；单链接隐藏 |
| `跳过` | 不提交当前链接，推进到下一条（ghost 弱化，置左） | 多链接时；单链接隐藏 |

**单链接收敛**：隐藏页码、隐藏"全部添加"、隐藏"跳过"，底部只剩"完成 ✓"。单链接体验 = 粘贴 → 一张卡 → 完成。

### 6.4 "全部添加" = 批量默认逃生舱

逐条确认对 2~3 条链接舒服，但粘贴 20 条时逐条点是折磨。"全部添加"点击后，**剩余链接全部按当前已建立的默认值**一次性入库：

- 保存位置 = 当前卡输入框的值
- 高级设置 = 收起态即默认（默认选流 / 默认容器 / 无限速 / 无范围 / 无定时）
- 文件名 = 各自自动提取值

这是 FDM / JDownloader 批量添加的本质——"默认值批量入库"。常用路径（逐条）与高效路径（批量）并存，用户自选。

### 6.5 提交语义

每条提交沿用现有服务层：

```
resolveLinkToTask(linkConfig, globalDefaults)  →  ResolvedTask
if (globalAutoStart && !hasSchedule) addAndStartTask(...)
else                                  addTask(...)
```

三层合并（链接 / 批次 / 全局）简化为**两层**（链接配置 / 全局默认）——批次层随 `BatchDefaults` 一并删除（见 §10）。

提交成功后，把该条使用的保存目录推入路径记忆（见 §7）。

## 7. 保存路径记忆（新增能力）

现状只有后端全局 `default_save_dir`，无"最近使用"概念。

**决策：前端 localStorage 快速落地，不进数据库、不动 schema。**

- Key：`streamgrab:recentSaveDirs`，值为 `string[]`，上限 5，去重，最新在前（MRU-first）。
- **用 VueUse `useStorage`**（`@vueuse/core` 已在依赖）实现，不手搓 localStorage 序列化/同步。封装为 `useRecentDirs` composable，对外暴露 `dirs` / `defaultDir` / `remember(dir)`，去重 + 截断逻辑集中于此。
- **默认显示值** = `recentSaveDirs[0]`；为空则回退 `settingsStore.defaultSaveDir`；再为空则空（placeholder 提示"使用全局默认"）。
- **更新时机**：某条任务**提交成功后**，把其解析出的非空保存目录推到记忆最前。**选中/浏览不立即写**，避免随手试目录污染记忆。
- "全部添加"时，记忆以点击时刻当前卡的保存目录为准，推一次。

优先级链：`记忆[0]` > `全局 defaultSaveDir` > 空。

## 8. 高级设置

### 8.1 内联手风琴，不做二级页面

选内联展开而非二级页，理由：

- 展开时 URL/路径/文件名仍在视野内，不丢上下文；二级页需"返回"，正是被否定的模式。
- 工具维度选项数量有限，手风琴装得下。
- 流选择器复用现成 `StreamPickerInline`，直接嵌入展开区。
- FDM / Motrix / qBittorrent 的通行做法。

展开项按工具类型动态渲染，复用现有 `linkOptionVisibility`（`isOptionVisible`）逻辑：

| 工具类型 | 高级项 |
|---|---|
| 流媒体（HLS/DASH/MSS） | 流选择（`StreamPickerInline`）、容器格式 `muxFormat`、字幕 `subtitleFormat`/`subtitlesOnly`、限速 `maxSpeed`、下载范围 `customRange`、定时 `schedule`、解密密钥 `key` |
| HTTP 直链视频 | 限速 `maxSpeed`、下载范围 `customRange`、定时 `schedule` |

注：`fileName` / `saveDir` 已提升到 L1 卡身，不再属于高级项。

### 8.2 流媒体解析失败的呈现

解析失败的流媒体链接仍进配置卡，流选择区显示"解析失败 [重试]"。用户可：

- 点"重试"重新 `parseUrl`；
- 或直接添加（下载时引擎按默认行为处理）。

失败不阻塞流程。

### 8.3 编辑 URL 不重解析

配置卡内编辑 URL 只更新提交字符串，`detectedType` / `streamInfo` 保持原解析结果。不提供实时重解析（YAGNI）；需重解析则关闭弹窗重来。

## 9. 边界与杂项

| 项 | 决策 |
|---|---|
| **URL 重复检测** | 逐条"添加"命中重复 → 弹现有 `UrlDuplicateDialog`（仍添加 / 跳过）。"全部添加"命中的重复**静默跳过**，结束 toast 汇报"跳过 N 个重复"。 |
| **无"返回粘贴"导航** | 进入 `config` 步骤后不可回 `paste`（标准向导）。需补充链接则关闭重开。 |
| **键盘** | 粘贴框 Enter = 解析并添加；配置卡 Enter = 添加/完成；Esc = 关闭。 |
| **预设** | 从添加流程中**完全移除**。`usePresetManager` / `seedPresetOverrides` 不再被 `AddTaskDialog` 使用。预设功能本身不变（仍在设置中心管理）。 |
| **关闭后状态** | 每次打开重置为 `paste` 空态。 |

## 10. 模块结构与职责分层

> 约束：先按"哪些功能天然一体"做内聚分组，再按层解耦；每个文件单一职责、边界明确；相同逻辑集中一处；优先复用成熟工具（VueUse）；**不做向后兼容**（硬删除、无 shim、无重导出）。

### 10.1 内聚分析（先想清楚什么该在一起）

添加任务可拆为六个**内聚簇**，各自职责单一、边界清晰：

| 内聚簇 | 职责 | 形态 |
|---|---|---|
| 流程编排 | 三步状态机、导航（添加/跳过/全部）、提交调度、重复处理 | composable |
| 链接解析 | 粘贴文本 → 分类后的结构化链接（类型/文件名/流信息/失败标记） | 纯函数 |
| 任务映射 | 单条链接配置 + 全局默认 → 提交所需的 `ResolvedTask` | 纯函数 |
| 路径记忆 | 最近保存目录的读写与默认值解析 | composable |
| 单条配置 UI | 一条链接的 L1 字段编辑（url/路径/文件名/徽章） | 组件 |
| 高级选项 UI | 按引擎类型动态渲染的高级项 + 内联流选择 | 组件 |

"流程编排"是编排者，允许知道多个下层；但**规则性逻辑**（怎么解析、怎么映射）下沉到纯函数，编排者只做"排序 + 持状态"。这就是"不过度抽象也不完全不抽象"的落点：不为三步向导搞策略/工厂，但把可测逻辑抽成纯函数、把 UI 按职责拆件。

### 10.2 文件结构与单一职责

```text
src/composables/
  useAddTaskWizard.ts     # 新增：状态机 + 导航 + 提交调度（调纯函数 + service/store/toast）
  useRecentDirs.ts        # 新增：基于 useStorage 的最近目录读写 + defaultDir
  index.ts                # 增补导出上述两者

src/components/task/
  AddTaskDialog.vue       # 重写：薄壳——步骤切换 + 导航按钮 + 重复弹窗渲染；粘贴/解析两步内联
  LinkConfigCard.vue      # 新增：单条链接 L1 字段 + 承载高级手风琴；纯展示编辑，不碰 service
  LinkAdvancedSection.vue # 新增：引擎驱动的高级项表单 + StreamPickerInline + 解析重试
  parseLinks.ts           # 新增：纯函数 text → 分类链接（无 Vue / 无副作用）
  parseLinks.test.ts      # 新增：单测
  resolveLinkToTask.ts    # 重写：两层映射（去掉 batch / preset 入参）
  resolveLinkToTask.test.ts # 更新：两层语义
  addTaskTypes.ts         # 新增（取代 staging-types.ts）：WizardStep / StagedLink + 集中展示映射
```

**硬删除（无向后兼容）：** `TaskStagingList.vue`、`LinkConfigPanel.vue`（拆分为 Card + AdvancedSection）、`staging-types.ts`、`BatchDefaults`、`LinkStatus`、`seedPresetOverrides`，以及 preset 在添加流程中的全部引用。

**职责边界要点：**

- `AddTaskDialog.vue` 从当前 357 行"什么都干"瘦身为纯编排渲染壳（步骤切换 + 导航 + 弹窗）。
- `LinkConfigCard` **不碰** service/流程——目录浏览通过 emit `browse-save-dir` 交还编排者（沿用"子组件不直接调 service"约定）。
- `LinkAdvancedSection` 只管"按类型渲染哪些选项"，复用 `linkOptionVisibility` + `StreamPickerInline`，不重复实现。
- 粘贴步与解析步内容极少（各数行），**内联于壳中**，不单独成件——避免过度碎片化。
- `StagedLink` 简化：去掉 `LinkStatus` 状态机枚举，改为显式字段 `streamInfo?` + `parseFailed: boolean`（失败 ≠ 无效，无效在解析阶段已剔除）。

### 10.3 分层与依赖方向

```text
UI 层        AddTaskDialog ─▶ LinkConfigCard ─▶ LinkAdvancedSection ─▶ StreamPickerInline
                │   v-model 编辑 StagedLink；组件内无服务/流程知识
编排层        useAddTaskWizard ─▶ useRecentDirs
                │   状态机 + 提交调度；唯一直接触达 service/store/toast 的层
纯逻辑层      parseLinks        resolveLinkToTask
                │   无 Vue、无副作用、可单测
服务/状态层    services(task/settings/system) · stores(task/settings) · useDownloader(parseUrl/addAndStartTask)
```

依赖**单向向下**：UI → 编排 → （纯逻辑 + 服务/状态）。纯函数不依赖任何响应式或服务；组件不直接触达服务（一律经编排层）。

### 10.4 集中化（消除分散）

- **类型/状态展示映射**：当前 `LinkConfigPanel` 内联 `typeBadgeLabel`、`TaskStagingList` 内联 `statusLabel/statusColor`，属重复分散。新设计统一到**单一映射**（置于 `addTaskTypes.ts`），卡片与高级区共用。
- **保存位置默认值解析**（recent > global > placeholder）：集中在 `useRecentDirs.defaultDir`，UI 不各自拼 placeholder。

### 10.5 复用成熟能力（不造轮子）

- 路径记忆用 **VueUse `useStorage`**（依赖已具备），不手搓 localStorage 序列化/跨标签同步。
- 流选择复用 `StreamPickerInline`；动态选项复用 `linkOptionVisibility`；解析/提交/目录浏览复用 `useDownloader` / `taskStore` / `systemService`。
- 向导本身足够简单，一个 composable 表达即可——**不引入额外状态库**。

### 10.6 后端

任务契约（`addTask` / `addAndStartTask`）与 `AppSettings` **零改动**。路径记忆走前端 `useStorage`，不动 schema。

## 11. 明确不做（YAGNI）

- 路径记忆不入库、不做 schema 迁移。
- 编辑 URL 不实时重解析。
- 不做"返回粘贴"状态回退。
- 添加流程不引入预设选择。
- 不为"全部添加"提供逐条重复弹窗（静默跳过 + 汇报）。

## 12. 验证思路（实现期）

- 单链接：粘贴 → 一张卡（无页码/全部添加/跳过）→ 完成 → 任务入库并按 autoStart 开始。
- 多链接：逐条"添加"推进、页码正确；"全部添加"批量入库剩余。
- 路径记忆：提交后记忆更新、下次打开默认值为最近、下拉列出历史、上限 5 去重。
- 无效链接解析阶段剔除并汇报；流媒体解析失败可重试/可添加。
- 重复 URL：逐条弹窗 / 批量静默跳过。
- 前端单测覆盖 `useRecentDirs`、简化后的 `resolveLinkToTask`、向导状态流转。
