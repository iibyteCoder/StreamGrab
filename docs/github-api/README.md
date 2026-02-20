# GitHub API 文档归档

本目录包含从 GitHub 官方文档下载的 API 参考文档。

## 文档列表

| 文件 | 描述 | 原文地址 |
|------|------|----------|
| [rest-api-getting-started.md](./rest-api-getting-started.md) | REST API 入门指南 | [GitHub Docs](https://docs.github.com/zh/rest/using-the-rest-api/getting-started-with-the-rest-api) |
| [releases-api.md](./releases-api.md) | Releases API 参考 | [GitHub Docs](https://docs.github.com/zh/rest/releases) |

## 主要内容

### REST API 入门

- HTTP 方法 (GET, POST, PATCH, PUT, DELETE)
- 路径参数、查询参数、正文参数
- 标头 (Accept, X-GitHub-Api-Version, User-Agent)
- 媒体类型
- 身份验证
- 速率限制

### Releases API

- 创建 Release
- 获取 Release 列表
- 上传 Release 资产
- 删除 Release

## 常用请求示例

### 使用 GitHub CLI

```bash
# 获取 Octocat
gh api --method GET /octocat \
  --header 'Accept: application/vnd.github+json' \
  --header "X-GitHub-Api-Version: 2022-11-28"

# 创建 Issue
gh api --method POST /repos/OWNER/REPO/issues \
  --header "Accept: application/vnd.github+json" \
  --header "X-GitHub-Api-Version: 2022-11-28" \
  -f title='Issue title' \
  -f body='Issue body'
```

### 使用 curl

```bash
# 获取 Release 列表
curl --request GET \
  --url "https://api.github.com/repos/OWNER/REPO/releases" \
  --header "Accept: application/vnd.github+json" \
  --header "X-GitHub-Api-Version: 2022-11-28" \
  --header "Authorization: Bearer YOUR-TOKEN"
```

## 速率限制

| 资源类型 | 认证用户 | 未认证 |
|---------|----------|--------|
| Core | 5000 次/小时 | 60 次/小时 |
| Search | 30 次/分钟 | 10 次/分钟 |

## 相关链接

- [GitHub REST API 官方文档](https://docs.github.com/zh/rest)
- [GitHub CLI 文档](https://cli.github.com/manual/gh_api)
- [Octokit.js 文档](https://github.com/octokit/octokit.js)
