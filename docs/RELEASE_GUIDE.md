# StreamGrab 发布流程指南

> 本文档记录了如何发布新版本的完整流程，适合学习参考。

## 目录

1. [发布前置条件](#发布前置条件)
2. [版本号规范](#版本号规范)
3. [手动发布流程](#手动发布流程)
4. [自动发布原理](#自动发布原理)
5. [常见问题](#常见问题)

---

## 发布前置条件

在发布之前，请确保：

- [ ] 所有代码已提交到 `main` 分支
- [ ] 功能测试通过（`npm run tauri dev` 本地测试）
- [ ] 类型检查通过（`npm run type-check`）
- [ ] CHANGELOG.md 已更新（可选但推荐）

---

## 版本号规范

本项目采用 [语义化版本](https://semver.org/lang/zh-CN/) 规范：`主版本号.次版本号.修订号`

| 版本类型 | 示例 | 适用场景 |
|---------|------|---------|
| **主版本 (Major)** | `1.0.0` → `2.0.0` | 重大架构变更、不兼容的 API 修改 |
| **次版本 (Minor)** | `0.1.0` → `0.2.0` | 新增功能、向后兼容的改进 |
| **修订版本 (Patch)** | `0.2.0` → `0.2.1` | Bug 修复、文档更新、小改进 |

---

## 手动发布流程

### 第一步：更新版本号

需要同时更新 **两个文件** 中的版本号：

```bash
# 1. package.json（前端版本）
# 找到 "version": "x.x.x" 并更新

# 2. src-tauri/tauri.conf.json（应用版本）
# 找到 "version": "x.x.x" 并更新
```

**示例**：

```json
// package.json
{
  "version": "0.2.1"
}

// src-tauri/tauri.conf.json
{
  "version": "0.2.1"
}
```

### 第二步：提交版本更新

```bash
# 添加更改的文件
git add package.json src-tauri/tauri.conf.json

# 提交（使用 chore 类型）
git commit -m "chore: bump version to 0.2.1"

# 推送到远程
git push origin main
```

### 第三步：创建 Git Tag

Tag 是 Git 中用于标记特定提交的标签，用于标识发布版本：

```bash
# 创建带注释的 tag（-a 表示 annotated，-m 是注释信息）
git tag -a v0.2.1 -m "Release v0.2.1: 修复文档和许可证"

# 查看所有 tag
git tag -l
```

### 第四步：推送 Tag 到 GitHub

```bash
# 推送单个 tag
git push origin v0.2.1

# 或者推送所有 tag
git push origin --tags
```

### 第五步：等待自动构建

推送 tag 后，GitHub Actions 会自动触发构建流程：

1. 打开 **https://github.com/iibyteCoder/StreamGrab/actions**
2. 查看名为「发布」的工作流
3. 等待构建完成（约 10-20 分钟）

构建会在以下平台进行：
- Windows (windows-latest)
- macOS (macos-latest)
- Ubuntu (ubuntu-22.04)

### 第六步：发布 Release

构建完成后：

1. 打开 **https://github.com/iibyteCoder/StreamGrab/releases**
2. 找到刚创建的 **Draft Release**（草稿）
3. 编辑 Release Notes（更新说明）
4. 点击 **Publish release** 正式发布

---

## 自动发布原理

### GitHub Actions 工作流

项目配置了自动发布工作流 `.github/workflows/release.yml`：

```yaml
name: 发布

on:
  push:
    tags:
      - 'v*'  # 当推送 v 开头的 tag 时触发

jobs:
  release:
    # 在 3 个平台并行构建
    strategy:
      matrix:
        include:
          - platform: 'macos-latest'
          - platform: 'ubuntu-22.04'
          - platform: 'windows-latest'

    steps:
      - name: 构建 Tauri 应用
        uses: tauri-apps/tauri-action@v0
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'StreamGrab ${{ github.ref_name }}'
          releaseDraft: true  # 创建为草稿
```

### 触发流程图

```
git push origin v0.2.1
        │
        ▼
GitHub 检测到 tag 推送
        │
        ▼
触发 release.yml 工作流
        │
        ▼
┌───────────────────────────────────────┐
│  并行构建 (Windows / macOS / Ubuntu)   │
│  - 安装依赖                            │
│  - 编译 Rust 后端                      │
│  - 构建前端                            │
│  - 打包安装程序                        │
└───────────────────────────────────────┘
        │
        ▼
创建 Draft Release（包含安装包）
        │
        ▼
开发者手动 Publish 发布
```

---

## 完整命令速查表

```bash
# ========== 发布新版本 ==========

# 1. 更新版本号（手动编辑 package.json 和 tauri.conf.json）

# 2. 提交版本更新
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: bump version to x.x.x"
git push origin main

# 3. 创建并推送 tag
git tag -a vx.x.x -m "Release vx.x.x: 简短描述"
git push origin vx.x.x

# 4. 等待 GitHub Actions 构建完成
# 5. 在 GitHub Releases 页面发布 Draft

# ========== 其他常用命令 ==========

# 查看所有 tag
git tag -l

# 查看某个 tag 详情
git show v0.2.0

# 删除本地 tag
git tag -d v0.2.0

# 删除远程 tag
git push origin --delete v0.2.0

# 查看远程 tag
git ls-remote --tags origin

# 查看当前状态
git status
git log --oneline -5
```

---

## 常见问题

### Q1: Tag 推送失败怎么办？

```bash
# 检查 tag 是否存在
git tag -l "v0.2.1"

# 重新推送
git push origin v0.2.1
```

### Q2: 构建失败了怎么排查？

1. 打开 GitHub Actions 页面
2. 点击失败的工作流
3. 查看具体步骤的错误日志
4. 修复问题后，删除远程 tag 重新推送：

```bash
# 删除远程 tag
git push origin --delete v0.2.1

# 重新创建并推送
git tag -a v0.2.1 -m "Release v0.2.1"
git push origin v0.2.1
```

### Q3: 如何撤回已发布的 Release？

1. 进入 GitHub Releases 页面
2. 点击要撤回的 Release
3. 点击右侧的 **Delete** 按钮
4. 删除对应的 tag：

```bash
git push origin --delete v0.2.1
git tag -d v0.2.1
```

### Q4: 版本号写错了怎么办？

如果还没推送 tag：

```bash
# 删除本地 tag
git tag -d v0.2.1

# 重新创建
git tag -a v0.2.2 -m "Release v0.2.2"
```

如果已经推送：

```bash
# 删除远程和本地 tag
git push origin --delete v0.2.1
git tag -d v0.2.1

# 重新创建并推送
git tag -a v0.2.2 -m "Release v0.2.2"
git push origin v0.2.2
```

---

## 最佳实践

1. **先测试后发布**：确保本地 `npm run tauri dev` 和 `npm run tauri:build` 都能正常工作

2. **更新 CHANGELOG**：在发布前更新 `CHANGELOG.md`，记录本次更新的内容

3. **使用有意义的 tag 注释**：简要说明这次发布的主要变更

4. **检查 Draft Release**：发布前检查生成的安装包是否完整

5. **版本号一致性**：确保 `package.json` 和 `tauri.conf.json` 的版本号一致

---

## 相关链接

- [GitHub Actions 文档](https://docs.github.com/cn/actions)
- [Tauri 发布指南](https://tauri.app/v1/guides/distribution/sign-windows)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Git Tag 文档](https://git-scm.com/book/zh/v2/Git-基础-打标签)
