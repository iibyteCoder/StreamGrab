# REST API 入门 - GitHub 文档

> 原文地址: https://docs.github.com/zh/rest/using-the-rest-api/getting-started-with-the-rest-api?apiVersion=2022-11-28
>
> 下载时间: 2026-02-20

---

## 简介

本文介绍如何通过 GitHub CLI、`curl` 或 JavaScript 使用 GitHub REST API。 有关快速入门指南，请参阅 GitHub REST API 快速入门。

## 关于对 REST API 的请求

本节介绍构成 API 请求的元素：

- HTTP 方法
- Path
- 标头
- 媒体类型
- 身份验证
- Parameters

每个对 REST API 的请求都包含一个 HTTP 方法和一个路径。 取决于 REST API 终结点，可能还需要指定请求标头、身份验证信息、查询参数或正文参数。

REST API 参考文档介绍了每个终结点的 HTTP 方法、路径和参数。 它还显示每个终结点的示例请求和响应。 有关详细信息，请查看 REST 参考文档。

### HTTP 方法

终结点的 HTTP 方法定义它对给定资源执行的操作类型。 常见的一些 HTTP 方法有 `GET`、`POST`、`DELETE` 和 `PATCH`。 REST API 参考文档介绍了每个终结点的 HTTP 方法。

例如，"列出存储库问题"终结点的 HTTP 方法为 `GET`。

在可能的情况下，GitHub REST API 尽量为每个操作使用适当的 HTTP 方法。

- `GET`：用于检索资源。
- `POST`：用于创建资源。
- `PATCH`：用于更新资源的属性。
- `PUT`：用于替换资源或集合。
- `DELETE`：用于删除资源。

### 路径

每个终结点都有一个路径。 REST API 参考文档介绍了每个终结点的路径。 例如，"列出存储库问题"终结点的路径为 `/repos/{owner}/{repo}/issues`。

路径中的大括号 `{}` 表示需要指定的路径参数。 路径参数修改终结点路径，在请求中是必需的。 例如，"列出存储库问题"终结点的路径参数为 `{owner}` 和 `{repo}`。 要在 API 请求中使用此路径，请将 `{repo}` 替换为想要请求问题列表的存储库的名称，并将 `{owner}` 替换为存储库所有者帐户的名称。

### 标头

标头包含有关请求和所需响应的其它信息。 以下是可在对 GitHub REST API 的请求中使用一些标头示例。 有关使用标头的请求示例，请参阅发出请求。

#### `Accept`

大多数 GitHub REST API 终结点指定应传递值为 `application/vnd.github+json` 的 `Accept` 标头。 `Accept` 标头的值为媒体类型。 有关媒体类型的详细信息，请参阅媒体类型。

#### `X-GitHub-Api-Version`

应使用此标头指定要用于请求的 REST API 版本。 有关详细信息，请参阅"API 版本"。

#### `User-Agent`

所有 API 请求都必须包含有效的 `User-Agent` 标头。 `User-Agent` 标头标识发出请求的用户或应用程序。

默认情况下，GitHub CLI 会发送有效的 `User-Agent` 标头。 但是，GitHub 建议使用 GitHub 用户名或应用程序名称作为 `User-Agent` 标头值。 这样，如果存在问题，GitHub 即可与你联系。

下面的示例 `User-Agent` 是一个名为 `Awesome-Octocat-App` 的应用：

```
User-Agent: Awesome-Octocat-App
```

没有 `User-Agent` 标头的请求将被拒绝。 如果提供无效的 `User-Agent` 标头，则将收到 `403 Forbidden` 响应。

### 媒体类型

可以通过将媒体类型添加到请求的 `Accept` 标头来指定一种或多种媒体类型。 有关 `Accept` 标头的详细信息，请参阅 `Accept`。

媒体类型指定要从 API 获取的数据格式。 媒体类型特定于资源，允许它们独立更改并支持其他资源不支持的格式。 每个 GitHub REST API 终结点的文档将描述它支持的媒体类型。 有关详细信息，请参阅 GitHub REST API 文档。

GitHub REST API 支持的最常见媒体类型是 `application/vnd.github+json` 和 `application/json`。

还可以将自定义媒体类型和某些端点搭配使用。 例如，用于管理提交和提取请求的 REST API 支持媒体类型 `diff`、`patch` 和 `sha`。 某些其他端点使用媒体类型 `full`、`raw`、`text` 或 `html`。

GitHub 的全部自定义媒体类型类似于 `application/vnd.github.PARAM+json`，其中 `PARAM` 是媒体类型的名称。 例如，要指定 `raw` 媒体类型，可以使用 `application/vnd.github.raw+json`。

### 身份验证

许多终结点需要身份验证或是在进行身份验证后返回其他信息。 此外，进行身份验证后，每小时可以发出更多请求。

要对请求进行身份验证，需要提供具有所需作用域或权限的身份验证令牌。 有几种不同方式获取令牌：可以创建 personal access token，生成包含 GitHub App 的令牌，或使用 GitHub Actions 工作流中内置的 `GITHUB_TOKEN`。 有关详细信息，请参阅"对 REST API 进行身份验证"。

**警告**: 应该像对待密码或其他敏感凭据那样对待访问令牌。 有关详细信息，请参阅"确保 API 凭据安全"。

### 参数

许多 API 方法要求或允许在请求的参数中发送其他信息。 有几种不同类型的参数：路径参数、正文参数和查询参数。

#### 路径参数

路径参数会修改终结点路径。 这些是请求中的必需参数： 有关详细信息，请参阅 Path。

#### 正文参数

正文参数使你可以将其他数据传递给 API。 上述参数可以是可选参数，也可以是必需参数，具体取决于终结点。 例如，正文参数可能允许在创建新问题时指定问题标题，或在启用/禁用功能时指定某些设置。 每个 GitHub REST API 终结点的文档将描述它支持的正文参数。 有关详细信息，请参阅 GitHub REST API 文档。

必须对请求进行身份验证才能传递正文参数。 有关详细信息，请参阅身份验证。

#### Query parameters

查询参数使你可以控制为请求返回的数据。 这些参数通常是可选的。 每个 GitHub REST API 终结点的文档将描述它支持的任何查询参数。 有关详细信息，请参阅 GitHub REST API 文档。

## 发出请求

### 使用 GitHub CLI

#### 1. 设置

在 macOS、Windows 或 Linux 上安装 GitHub CLI。 有关安装说明的详细信息，请参阅 GitHub CLI 存储库中的安装。

#### 2. 身份验证

1. 若要向 GitHub 进行身份验证，请从终端运行以下命令：

   ```bash
   gh auth login
   ```

2. 选择要进行身份验证的位置：
   - 如果通过 GitHub.com 访问 GitHub，请选择 "GitHub.com"
   - 如果通过其他域访问 GitHub，请选择 "其他"，然后输入主机名（例如 `octocorp.ghe.com`）

3. 按照屏幕上的其余提示操作。

#### 3. 使用 GitHub CLI 发出请求

使用 GitHub CLI `api` 子命令发出 API 请求。 在请求中，指定以下选项和值：

- `--method` 后跟 HTTP 方法和终结点的路径
- `--header`:
  - `Accept`：在 `Accept` 标头中传递媒体类型
  - `X-GitHub-Api-Version`：在 `X-GitHub-Api-Version` 标头中传递 API 版本
- `-f` 或 `-F` 后跟任何采用 `key=value` 格式的正文参数或查询参数

**示例请求：**

```bash
gh api --method GET /octocat \
--header 'Accept: application/vnd.github+json' \
--header "X-GitHub-Api-Version: 2022-11-28"
```

### 使用 curl

#### 1. 设置

必须在计算机上安装 `curl`。 要检查是否安装了 `curl`，请在命令行中运行 `curl --version`。

#### 2. 发出 curl 请求

在请求中指定以下选项和值：

- `--request` 或 `-X` 后跟 HTTP 方法作为值
- `--url` 后跟完整路径作为值
- `--header` 或 `-H`：
  - `Accept`：在 `Accept` 标头中传递媒体类型
  - `X-GitHub-Api-Version`：在 `X-GitHub-Api-Version` 标头中传递 API 版本
  - `Authorization`：在 `Authorization` 标头中传递身份验证令牌
- `--data` 或 `-d` 后跟 JSON 对象中的任何正文参数

**示例请求：**

```bash
curl --request GET \
--url "https://api.github.com/octocat" \
--header "Accept: application/vnd.github+json" \
--header "X-GitHub-Api-Version: 2022-11-28"
```

**使用身份验证的示例：**

```bash
curl \
--request POST \
--url "https://api.github.com/repos/octocat/Spoon-Knife/issues" \
--header "Accept: application/vnd.github+json" \
--header "X-GitHub-Api-Version: 2022-11-28" \
--header "Authorization: Bearer YOUR-TOKEN" \
--data '{
  "title": "Created with the REST API",
  "body": "This is a test issue created by the REST API"
}'
```

### 使用 JavaScript (Octokit.js)

#### 1. 设置

安装 `octokit`。 例如，`npm install octokit`。

#### 2. 使用 Octokit.js 发出请求

```javascript
import { Octokit } from "octokit";

const octokit = new Octokit({
  auth: 'YOUR-TOKEN'
});

// 发送请求
await octokit.request("POST /repos/{owner}/{repo}/issues", {
  owner: "octocat",
  repo: "Spoon-Knife",
  title: "Created with the REST API",
  body: "This is a test issue created by the REST API",
});
```

## 使用响应

### 关于响应代码和标头

每个请求都会返回 HTTP 状态代码，以指示响应是否成功。 此外，响应会包含标头，以提供有关响应的更多详细信息。 以 `X-` 或 `x-` 开头的标头对于 GitHub 是自定义的。

**重要的响应标头：**

- `X-RateLimit-Limit`: 每小时允许的请求数
- `X-RateLimit-Remaining`: 剩余请求数
- `X-RateLimit-Reset`: 速率限制重置的 Unix 时间戳
- `X-RateLimit-Resource`: 资源类型
- `X-RateLimit-Used`: 已使用的请求数

### 关于响应正文

许多终结点会返回响应正文。 除非另外指定，否则响应正文会采用 JSON 格式。 空白字段包含为 `null`，而不是被省略。 所有时间戳以 ISO 8601 格式返回 UTC 时间：`YYYY-MM-DDTHH:MM:SSZ`。

### 详细表示形式与摘要表示形式

- **详细表示形式**：提取单个资源时，响应通常会包含该资源的所有属性
- **摘要表示形式**：提取资源列表时，响应将仅包含每个资源的属性子集

## 速率限制

GitHub REST API 有以下速率限制（针对认证用户）：

| 资源类型 | 限制 |
|---------|------|
| Core | 5000 次/小时 |
| Search | 30 次/分钟 |
| GraphQL | 5000 点/小时 |

未认证请求的限制为 60 次/小时。

可以通过检查响应标头来监控速率限制使用情况：

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4996
X-RateLimit-Reset: 1659645499
X-RateLimit-Used: 4
```

## 后续步骤

- 浏览 [REST API 参考文档](https://docs.github.com/zh/rest/reference)
- 了解更多关于 [身份验证](https://docs.github.com/zh/rest/authentication)
- 查看 [最佳做法](https://docs.github.com/zh/rest/guides/best-practices)
