# Design System Master File — StreamGrab

> **逻辑：** 构建具体页面时，先检查 `design-system/streamgrab/pages/[page-name].md`。
> 若存在，其规则**覆盖**本文件；否则严格遵循本文件。
> **本文件已按项目既有设计语言校准**（CLAUDE.md UI 规范优先于工具默认推荐）。

---

**项目:** StreamGrab（Tauri 2.0 桌面应用，Vue 3 + TailwindCSS + shadcn-vue）
**生成:** 2026-07-21（配色已校准为项目既有 tokens）
**类别:** 桌面工具 / 下载器 / 任务管理
**设计刻度:** Motion 4/10（功能性动画优先） | Density 7/10（任务列表适度紧凑）

---

## 全局规则

### 色彩（项目既有 tokens，唯一来源：`src/style.css`）

| 角色 | Hex | CSS 变量 | 用途 |
|------|-----|----------|------|
| 背景-基底 | `#0a0a0a` | `--bg-base` | 页面底色（深色主题） |
| 背景-表面 | `#111111` | `--bg-surface` | 卡片/面板/输入框 |
| 文字-主要 | `#fafafa` | `--text-primary` | 标题/主要文本 |
| 文字-次要 | `#888888` | `--text-secondary` | 辅助信息（确保 ≥4.5:1） |
| 强调-主色 | `#3b82f6` | `--accent-primary` | 主按钮/链接/焦点环/进度条 |
| 强调-成功 | `#22c55e` | `--accent-success` | 完成状态/速度正向指标 |
| 强调-错误 | `#ef4444` | `--accent-error` | 失败状态/危险操作 |
| 边框 | `rgba(255,255,255,0.08)` | `--color-border` | 卡片/分隔线 |

**浅色主题**：同名变量反转（白底深字），组件只引用变量、禁止硬编码 hex。
**状态色语义固定**：pending=灰、downloading=蓝、merging/muxing=蓝紫、completed=绿、failed=红、cancelled=灰。

### 字体

- **正文/标题**：系统 UI 栈（`system-ui, -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif`）——桌面应用不引入 webfont，启动快、渲染原生
- **数据展示**（速度/大小/进度/时长/命令行日志）：等宽栈（`"Cascadia Code", "JetBrains Mono", Consolas, monospace`）——下载器的核心特征是实时数字流，等宽防抖动
- **字号对比**：页面标题 20-24px/600，卡片标题 14-15px/600，正文 13-14px/400，辅助信息 12px（不小于 12px）

### 间距（CLAUDE.md 规范）

| Token | 值 | 用途 |
|-------|-----|------|
| 组件内 | 8px / 12px | 图标间隙、行内间距 |
| 组件间 | 16px / 24px | 卡片间距、区块内边距 |
| 页面边距 | 24px | 视图四周留白 |

### 阴影与深度

深色主题不依赖阴影，靠**表面层级**（`#0a0a0a` → `#111111` → `#1a1a1a` 悬停）表达层次；弹层用 `rgba(0,0,0,0.6)` 遮罩 + 1px 边框。浅色主题才使用柔和阴影（`0 4px 12px rgba(0,0,0,0.08)`）。

---

## 组件规范

### 按钮

- 主按钮：`--accent-primary` 底 + 白字，hover 亮度 +8%，active 下沉 1px，`border-radius: 8px`，过渡 150ms ease-out
- 次按钮：透明底 + 1px 边框 + `--text-primary`，hover 表面色填充
- 危险按钮：`--accent-error` 仅用于确认后的破坏性操作（删除任务/清空历史）
- 图标按钮：最小命中区 32×32px（桌面端），必须带 `title`/tooltip

### 卡片（任务卡片/设置卡片）

```
background: var(--bg-surface);
border: 1px solid rgba(255,255,255,0.08);
border-radius: 12px;
transition: border-color 150ms ease-out, background 150ms ease-out;
/* hover：边框色升至 rgba(255,255,255,0.16)，不做位移/缩放（信息类 UI 禁止 layout-shift 悬停） */
```

### 输入框

- `--bg-surface` 底 + 1px 边框，focus 时边框 `--accent-primary` + 3px 外发光环 `rgba(59,130,246,0.2)`
- placeholder 仅做提示，标签永远可见（设置项用左侧 label + 右侧控件布局）

### 进度条

- 轨道 `rgba(255,255,255,0.08)`，填充 `--accent-primary`，高度 4-6px，圆角
- 速度/ETA 数字用等宽字体，右对齐，数值变化不引起布局抖动

### 标签页（设置中心 4 标签页）

- 顶部或左侧标签，选中态：`--accent-primary` 下划线/背景 + `--text-primary`；未选中 `--text-secondary`
- 切换内容 150ms fade，禁止整页闪白

---

## 页面模式（桌面应用，非落地页）

| 页面 | 结构要点 |
|------|---------|
| 主页（任务列表） | 顶部输入区 + 工具栏（过滤/批量操作）+ 任务卡片列表；卡片默认紧凑（文件名/进度/速度），悬停展开快速操作——遵循渐进披露三层模型 |
| 添加任务对话框 | URL 输入 → 类型徽章即时反馈（检测到 HLS/直链时徽章淡入）→ 流选择 → 任务级选项 → 高级折叠；每步有可感知的状态反馈 |
| 设置中心 | 4 标签页（常规·界面 / N_m3u8DL-RE / FFmpeg / 任务预设），每页 ToolManager 卡片在顶部（路径+版本+状态徽章），设置项分组卡片 |
| 历史记录页 | 列表/表格混合，状态徽章着色，行悬停显示操作，空状态插画+引导文案 |

---

## 动画（CLAUDE.md：只为反馈使用，不为装饰）

- **时长**：100-200ms，缓动 `ease-out`（弹层入场可到 250ms）
- **列表入场**：任务卡片/历史行轻量 stagger（每项延迟 30ms，淡入 + 8px 上移，总时长 <400ms）——密集数据表格禁用回弹缓动
- **必须**：所有可点击元素 `cursor-pointer`；状态变化（任务状态切换/设置保存成功）有 Toast 或徽章过渡反馈；尊重 `prefers-reduced-motion`
- **禁止**：纯装饰动画、width/height 直接动画（用 transform/opacity）、布局抖动悬停

---

## 反模式（禁止）

- ❌ Emoji 当图标——统一 Lucide（项目已用 `lucide-vue-next`，经 `AppIcon` 组件）
- ❌ 低对比文本（<4.5:1）、灰上灰
- ❌ 组件内硬编码 hex（必须走 CSS 变量，保证主题切换）
- ❌ 瞬时状态切换（无过渡）
- ❌ 悬停才显示关键信息（键盘/触屏不可达）
- ❌ 一次性展示过多设置项（违反渐进披露）
- ❌ 全站玻璃拟态/统一 rounded-2xl/渐变标题

---

## 交付前检查清单

- [ ] 图标全部来自 Lucide（经 AppIcon），无 emoji
- [ ] 所有可点击元素 `cursor-pointer` + hover 过渡（150ms 级）
- [ ] 深色/浅色双主题均通过 4.5:1 对比（重点检查 `--text-secondary`）
- [ ] focus 态可见（键盘导航）
- [ ] `prefers-reduced-motion` 已尊重
- [ ] 数字类信息等宽字体、右对齐、无布局抖动
- [ ] 组件无硬编码 hex，全部引用 CSS 变量
- [ ] 桌面窗口常见宽度（1024/1280/1440/1920）布局无横向滚动
