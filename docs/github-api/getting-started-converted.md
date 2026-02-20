Skip to main content

GitHub 文档

Version: Free, Pro, & Team

搜索或询问 Copilot

搜索或询问Copilot

Select language: current language is Simplified Chinese

搜索或询问 Copilot

搜索或询问Copilot

打开菜单

Open Sidebar

- REST API/
- 使用 REST API/
- 入门

主

## REST API

API Version: 2022-11-28 (latest)

- 快速入门
- REST API 简介
  - REST API 简介
  - 比较 GitHub 的 API
  - API 版本
  - 重大更改
  - OpenAPI 描述
- 使用 REST API
  - 入门
  - 速率限制
  - 分页
  - Libraries
  - 最佳做法
  - 故障排除
  - 时区
  - CORS 与 JSONP
  - 议题事件类型
  - GitHub 事件类型
- 身份验证
  - 身份验证
  - 确保 API 凭据安全
  - 适用于 GitHub Apps 安装令牌的终结点
  - 适用于 GitHub Apps 用户令牌的终结点
  - 细化 PAT 的终结点
  - GitHub 应用的权限
  - 细化 PAT 的权限
- Guides
  - 使用 JavaScript 编写脚本
  - 使用 Ruby 编写脚本
  - 为用户发现资源
  - 交付部署
  - 将数据渲染为图形
  - 处理注释
  - 构建 CI 服务器
  - 入门 - Git 数据库
  - 开始 - 检查
  - 加密机密

---

- 操作
  - Artifacts
  - 缓存
  - GitHub 托管的运行程序
  - OIDC
  - 权限
  - 机密
  - 自托管运行器组
  - 自托管运行程序
  - 变量
  - 工作流程作业
  - 工作流运行
  - 工作流
- 活动
  - 事件
  - 源
  - 通知
  - 标星
  - 关注中
- 应用
  - GitHub Apps
  - 安装
  - 市场
  - OAuth 授权
  - Webhook
- 计费
  - Budgets
  - 计费使用情况
- 分支
  - 分支
  - 受保护的分支
- 营销活动
  - 安全活动
- 检查
  - 检查运行
  - 检查套件
- 教室
  - 教室
- 代码扫描
  - 代码扫描
- 代码安全设置
  - 配置
- 行为准则
  - 行为准则
- Codespaces
  - Codespaces
  - 组织
  - 组织机密
  - 机
  - 存储库机密
  - 用户机密
- 协作者
  - 协作者
  - 邀请
- 提交
  - 提交
  - 提交注释
  - 提交状态
- Copilot
  - Copilot 指标
  - Copilot 用户管理
- 凭据
  - 撤销
- Dependabot
  - 警报
  - 存储库访问
  - 机密
- 依赖项关系图
  - 依赖项检查
  - 依赖项提交
  - 软件材料清单 (SBOM)
- 部署密钥
  - 部署密钥
- 部署
  - 部署分支策略
  - 部署
  - 环境
  - 保护规则
  - 部署状态
- 表情符号
  - 表情符号
- 企业团队
  - 企业团队成员
  - 企业团队组织
  - 企业团队
- Gists
  - Gists
  - 注释
- Git 数据库
  - Blob
  - 提交
  - 参考
  - 标记
  - 树
- Gitignore
  - Gitignore
- 交互
  - 组织
  - 存储库
  - 用户
- 问题
  - 受理人
  - 注释
  - 事件
  - 问题
  - 问题依赖项
  - 标签
  - 里程碑
  - 子问题
  - 时间线
- 许可证
  - 许可证
- Markdown
  - Markdown
- 元
  - 元
- 指标
  - Community
  - 统计信息
  - 交通
- 迁移
  - 组织
  - 源终结点
  - 用户
- 模型
  - 目录
  - 嵌入
  - 推理
- 组织
  - API 见解
  - 项目元数据
  - 项目证明
  - 阻止用户
  - 自定义属性
  - 议题类型
  - 成员
  - 网络配置
  - 组织角色
  - 组织
  - 外部协作者
  - 个人访问令牌
  - 规则套件
  - 规则
  - 安全管理员
  - Webhook
- 包
  - 包
- 页
  - 页
- 专用注册表
  - 组织配置
- 项目
  - 草稿项目内容
  - 项目字段
  - 项目物料
  - 项目
  - 项目视图
- 拉取请求
  - 拉取请求
  - 评价注释
  - 审查请求
  - 审阅
- 速率限制
  - 速率限制
- 反应
  - 反应
- 发行版本
  - 发行版本
  - 发布资产
- 存储库
  - Attestations
  - 自动链接
  - 目录
  - 自定义属性
  - 前叉
  - 存储库
  - 规则套件
  - 规则
  - 标记
  - Webhook
- Search
  - 搜寻
- 机密扫描
  - 推送保护
  - 机密扫描
- 安全通知
  - 全局安全公告
  - 存储库安全公告
- Teams
  - 成员
  - Teams
- 用户
  - 证明
  - 阻止用户
  - 电子邮件
  - 关注者
  - GPG 密钥
  - Git SSH 密钥
  - 社交帐户
  - SSH 签名密钥
  - 用户

- REST API/
- 使用 REST API/
- 入门

# REST API 入门

了解如何使用 GitHub REST API。

## Tool navigation

- GitHub CLI
- curl
- JavaScript

将页面显示为 Markdown

## 本文内容

- 简介
- 关于对 REST API 的请求
- 发出请求
- 使用响应
- 后续步骤

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

例如，“列出存储库问题”终结点的 HTTP 方法为 `GET`。

在可能的情况下，GitHub REST API 尽量为每个操作使用适当的 HTTP 方法。

- `GET`：用于检索资源。
- `POST`：用于创建资源。
- `PATCH`：用于更新资源的属性。
- `PUT`：用于替换资源或集合。
- `DELETE`：用于删除资源。

### 路径

每个终结点都有一个路径。 REST API 参考文档介绍了每个终结点的路径。 例如，“列出存储库问题”终结点的路径为 `/repos/{owner}/{repo}/issues`。

路径中的大括号 `{}` 表示需要指定的路径参数。 路径参数修改终结点路径，在请求中是必需的。 例如，“列出存储库问题”终结点的路径参数为 `{owner}` 和 `{repo}`。 要在 API 请求中使用此路径，请将 `{repo}` 替换为想要请求问题列表的存储库的名称，并将 `{owner}` 替换为存储库所有者帐户的名称。

### 标头

标头包含有关请求和所需响应的其它信息。 以下是可在对 GitHub REST API 的请求中使用一些标头示例。 有关使用标头的请求示例，请参阅发出请求。

#### `Accept`

大多数 GitHub REST API 终结点指定应传递值为 `application/vnd.github+json` 的 `Accept` 标头。 `Accept` 标头的值为媒体类型。 有关媒体类型的详细信息，请参阅媒体类型。

#### `X-GitHub-Api-Version`

应使用此标头指定要用于请求的 REST API 版本。 有关详细信息，请参阅“API 版本”。

#### `User-Agent`

所有 API 请求都必须包含有效的 `User-Agent` 标头。 `User-Agent` 标头标识发出请求的用户或应用程序。

默认情况下，GitHub CLI 会发送有效的 `User-Agent` 标头。 但是，GitHub 建议使用 GitHub 用户名或应用程序名称作为 `User-Agent` 标头值。 这样，如果存在问题，GitHub 即可与你联系。

默认情况下，`curl` 会发送有效的 `User-Agent` 标头。 但是，GitHub 建议使用 用户名或应用程序名称作为 `User-Agent` 标头值。 这样，如果存在问题，GitHub 即可与你联系。

如果使用的是 Octokit.js SDK，则该 SDK 为你发送有效的 `User-Agent` 标头。 但是，GitHub 建议使用 GitHub 用户名或应用程序名称作为 `User-Agent` 标头值。 这样，如果存在问题，GitHub 即可与你联系。

下面的示例 `User-Agent` 是一个名为 `Awesome-Octocat-App` 的应用：

    User-Agent: Awesome-Octocat-App

没有 `User-Agent` 标头的请求将被拒绝。 如果提供无效的 `User-Agent` 标头，则将收到 `403 Forbidden` 响应。

### 媒体类型

可以通过将媒体类型添加到请求的 `Accept` 标头来指定一种或多种媒体类型。 有关 `Accept` 标头的详细信息，请参阅 `Accept`。

媒体类型指定要从 API 获取的数据格式。 媒体类型特定于资源，允许它们独立更改并支持其他资源不支持的格式。 每个 GitHub REST API 终结点的文档将描述它支持的媒体类型。 有关详细信息，请参阅 GitHub REST API 文档。

GitHub REST API 支持的最常见媒体类型是 `application/vnd.github+json` 和 `application/json`。

还可以将自定义媒体类型和某些端点搭配使用。 例如，用于管理提交和提取请求的 REST API 支持媒体类型 `diff`、`patch` 和 `sha`。 某些其他端点使用媒体类型 `full`、`raw`、`text` 或 `html`。

GitHub 的全部自定义媒体类型类似于 `application/vnd.github.PARAM+json`，其中 `PARAM` 是媒体类型的名称。 例如，要指定 `raw` 媒体类型，可以使用 `application/vnd.github.raw+json`。

有关使用媒体类型的请求示例，请参阅发出请求。

### 身份验证

许多终结点需要身份验证或是在进行身份验证后返回其他信息。 此外，进行身份验证后，每小时可以发出更多请求。

要对请求进行身份验证，需要提供具有所需作用域或权限的身份验证令牌。 有几种不同方式获取令牌：可以创建 personal access token，生成包含 GitHub App 的令牌，或使用 GitHub Actions 工作流中内置的 `GITHUB_TOKEN`。 有关详细信息，请参阅“对 REST API 进行身份验证”。

有关使用身份验证令牌的请求示例，请参阅发出请求。

注意

如果不想创建令牌，可以使用 GitHub CLI。 GitHub CLI 将自动进行身份验证，并帮助保护帐户的安全。 有关详细信息，请参阅此页面的 GitHub CLI 版本。

警告

应该像对待密码或其他敏感凭据那样对待访问令牌。 有关详细信息，请参阅“确保 API 凭据安全”。

尽管无需身份验证即可访问某些 REST API 终结点，但 GitHub CLI 要求先进行身份验证，然后才能使用 `api` 子命令发出 API 请求。 使用 `auth login` 子命令向 GitHub 进行身份验证。 有关详细信息，请参阅发出请求。

要对请求进行身份验证，需要提供具有所需作用域或权限的身份验证令牌。 有几种不同方式获取令牌：可以创建 personal access token，生成包含 GitHub App 的令牌，或使用 GitHub Actions 工作流中内置的 `GITHUB_TOKEN`。 有关详细信息，请参阅“对 REST API 进行身份验证”。

有关使用身份验证令牌的请求示例，请参阅发出请求。

警告

应该像对待密码或其他敏感凭据那样对待访问令牌。 有关详细信息，请参阅“确保 API 凭据安全”。

### 参数

许多 API 方法要求或允许在请求的参数中发送其他信息。 有几种不同类型的参数：路径参数、正文参数和查询参数。

#### 路径参数

路径参数会修改终结点路径。 这些是请求中的必需参数： 有关详细信息，请参阅 Path。

#### 正文参数

正文参数使你可以将其他数据传递给 API。 上述参数可以是可选参数，也可以是必需参数，具体取决于终结点。 例如，正文参数可能允许在创建新问题时指定问题标题，或在启用/禁用功能时指定某些设置。 每个 GitHub REST API 终结点的文档将描述它支持的正文参数。 有关详细信息，请参阅 GitHub REST API 文档。

例如，“创建问题”终结点要求为请求中的新问题指定标题。 此外，还允许选择指定其他信息，例如要放入问题正文中的文本、要分配给新问题的用户或要应用于新问题的标签。 有关使用正文参数的请求示例，请参阅发出请求。

必须对请求进行身份验证才能传递正文参数。 有关详细信息，请参阅身份验证。

#### Query parameters

查询参数使你可以控制为请求返回的数据。 这些参数通常是可选的。 每个 GitHub REST API 终结点的文档将描述它支持的任何查询参数。 有关详细信息，请参阅 GitHub REST API 文档。

例如，“列出公共事件”终结点 默认返回 30 个问题。 可以使用 `per_page` 查询参数返回 2 个问题，而不是 30 个问题。 可以使用 `page` 查询参数仅提取结果的第一页。 有关使用查询参数的请求示例，请参阅发出请求。

## 发出请求

本节演示了如何使用 GitHub CLI 向 GitHub REST API 发出经过身份验证的请求。

### 1\. 设置

在 macOS、Windows 或 Linux 上安装 GitHub CLI。 有关安装说明的详细信息，请参阅 GitHub CLI 存储库中的安装。

### 2\. 身份验证

1. 若要向 GitHub 进行身份验证，请从终端运行以下命令。
   gh auth login

可以使用 `--scopes` 选项指定所需的作用域。 如果要使用创建的令牌进行身份验证，可以使用 `--with-token` 选项。 有关详细信息，请参阅 GitHub CLI `auth login` 文档。

2. 选择要进行身份验证的位置：
   - 如果通过 GitHub.com 访问 GitHub，请选择“GitHub.com”\*\*\*\*。
   - 如果通过其他域访问 GitHub，请选择“其他”，然后输入主机名（例如 `octocorp.ghe.com`）\*\*\*\*。

3. 按照屏幕上的其余提示操作。

选择 HTTPS 作为 Git 操作的首选协议时，GitHub CLI 将自动存储 Git 凭据，并对询问是否要使用 GitHub 凭据向 Git 进行身份验证的提示回答“是”。 此选项非常有用，因为这允许直接使用 `git push`、`git pull` 等 Git 命令，无需设置单独的凭据管理器或使用 SSH。

### 3\. 为请求选择终结点

1. 选择要向其发出请求的终结点。 可以浏览 GitHub 的 REST API 文档，了解可用于与 GitHub 交互的终结点。

2. 标识终结点的 HTTP 方法和路径。 将随请求一起发送这些内容。 有关详细信息，请参阅 HTTP 方法和路径。

例如，“创建问题”终结点使用 HTTP 方法和 `POST` 路径 `/repos/{owner}/{repo}/issues`。

3. 标识任何必需的路径参数。 必需的路径参数显示在终结点路径的大括号 `{}` 中。 将每个参数占位符替换为想要的值。 有关详细信息，请参阅 Path。

例如，“创建问题”终结点使用路径 `/repos/{owner}/{repo}/issues`，路径参数为 `{owner}` 和 `{repo}`。 要在 API 请求中使用此路径，请将 `{repo}` 替换为想要创建新问题的存储库的名称，并将 `{owner}` 替换为存储库所有者帐户的名称。

### 4\. 使用 GitHub CLI 发出请求

使用 GitHub CLI `api` 子命令发出 API 请求。 有关详细信息，请参阅 GitHub CLI `api` 文档。

在请求中，指定以下选项和值：

- **\--method** 后跟 HTTP 方法和终结点的路径。 有关详细信息，请参阅 HTTP 方法和路径。

- **\--header** :
  - **`Accept`：** 在 `Accept` 标头中传递媒体类型。 要在标头 `Accept` 中传递多个媒体类型，请使用逗号分隔媒体类型：`Accept: application/vnd.github+json,application/vnd.github.diff`。 有关详细信息，请参阅 `Accept` 和媒体类型。
  - **`X-GitHub-Api-Version`：** 在 `X-GitHub-Api-Version` 标头中传递 API 版本。 有关详细信息，请参阅 `X-GitHub-Api-Version`。

- **`-f`** 或 **`-F`** 后跟任何采用 `key=value` 格式的正文参数或查询参数。 使用 `-F` 选项传递数字、布尔或 null 参数。 使用 `-f` 选项传递字符串参数。

某些终结点使用属于数组的查询参数。 要在查询字符串中发送数组，请为每个数组项使用查询参数一次，并在查询参数名称后追加 `[]`。 例如，要提供两个存储库 ID 的数组，请使用 `-f repository_ids[]=REPOSITORY_A_ID -f repository_ids[]=REPOSITORY_B_ID`。

如果不需要在请求中指定任何正文参数或查询参数，请省略此选项。 有关详细信息，请参阅正文参数和查询参数。 有关示例，请参阅使用正文参数的示例请求和使用查询参数的示例请求。

#### 示例请求

以下示例请求使用“获取 Octocat”终结点将 Octocat 返回为 ASCII 艺术。

Shell

    gh api --method GET /octocat \
    --header 'Accept: application/vnd.github+json' \
    --header "X-GitHub-Api-Version: 2022-11-28"



    gh api --method GET /octocat \
    --header 'Accept: application/vnd.github+json' \
    --header "X-GitHub-Api-Version: 2022-11-28"

#### 使用查询参数的示例请求

“列出公共事件”终结点默认返回 30 个问题。 以下示例使用 `per_page` 查询参数返回两个问题而不是 30 个，查询参数 `page` 仅提取结果的第一页。

Shell

    gh api --method GET /events -F per_page=2 -F page=1
    --header 'Accept: application/vnd.github+json' \



    gh api --method GET /events -F per_page=2 -F page=1
    --header 'Accept: application/vnd.github+json' \

#### 使用正文参数的示例请求

以下示例使用“创建问题”终结点在 the octocat/Spoon-Knife 存储库中创建新问题。 在响应中，找到问题的 `html_url`，并在浏览器中导航到问题。

Shell

    gh api --method POST /repos/octocat/Spoon-Knife/issues \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28" \
    -f title='Created with the REST API' \
    -f body='This is a test issue created by the REST API' \



    gh api --method POST /repos/octocat/Spoon-Knife/issues \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28" \
    -f title='Created with the REST API' \
    -f body='This is a test issue created by the REST API' \

本部分演示了如何使用 `curl` 向 GitHub REST API 发出经过身份验证的请求。

### 1\. 设置

必须在计算机上安装 `curl`。 要检查是否安装了 `curl`，请在命令行中运行 `curl --version`。

- 如果输出是有关 `curl` 版本的信息，则表示已安装 `curl`。
- 如果收到类似 `command not found: curl` 的消息，则表示未安装 `curl`。 下载并安装 `curl`。 有关详细信息，请参阅 curl 下载页面。

### 2\. 为请求选择终结点

1. 选择要向其发出请求的终结点。 可以浏览 GitHub 的 REST API 文档，了解可用于与 GitHub 交互的终结点。

2. 标识终结点的 HTTP 方法和路径。 将随请求一起发送这些内容。 有关详细信息，请参阅 HTTP 方法和路径。

例如，“创建问题”终结点使用 HTTP 方法和 `POST` 路径 `/repos/{owner}/{repo}/issues`。

3. 标识任何必需的路径参数。 必需的路径参数显示在终结点路径的大括号 `{}` 中。 将每个参数占位符替换为想要的值。 有关详细信息，请参阅 Path。

例如，“创建问题”终结点使用路径 `/repos/{owner}/{repo}/issues`，路径参数为 `{owner}` 和 `{repo}`。 要在 API 请求中使用此路径，请将 `{repo}` 替换为想要创建新问题的存储库的名称，并将 `{owner}` 替换为存储库所有者帐户的名称。

### 3\. 创建身份验证凭据

创建访问令牌对请求进行身份验证。 可以保存令牌并将其用于多个请求。 为令牌提供访问终结点所需的任何作用域或权限。 将会在 `Authorization` 标头中与请求一起发送此令牌。 有关详细信息，请参阅身份验证。

### 4\. 发出 `curl` 请求。

使用 `curl` 命令发出请求。 有关详细信息，请参阅 curl 文档。

在请求中指定以下选项和值：

- **`--request` 或 `-X`** 后跟 HTTP 方法作为值。 有关更多信息，请参阅 HTTP 方法。

- **`--url`** 后跟完整路径作为值。 完整路径是包含 GitHub REST API（`https://api.github.com`）的基本 URL 和终结点的路径的 URL，如下所示：`https://api.github.com/PATH`。将 `PATH` 替换为终结点的路径。 有关详细信息，请参阅 Path。

要使用查询参数，先在路径末尾添加 `?`，然后采用 `parameter_name=value` 形式追加查询参数名称和值。 使用 `&` 分隔多个查询参数。 如果需要在查询字符串中发送数组，请为每个数组项使用查询参数一次，并在查询参数名称后追加 `[]`。 例如，要提供两个存储库 ID 的数组，请使用 `?repository_ids[]=REPOSITORY_A_ID&repository_ids[]=REPOSITORY_B_ID`。 有关详细信息，请参阅查询参数。 有关示例，请参阅使用查询参数的示例请求。

- **`--header` 或 `-H`：**
  - **`Accept`：** 在 `Accept` 标头中传递媒体类型。 要在标头 `Accept` 中传递多个媒体类型，请使用逗号分隔媒体类型，例如：`Accept: application/vnd.github+json,application/vnd.github.diff`。 有关详细信息，请参阅 `Accept` 和媒体类型。
  - **`X-GitHub-Api-Version`：** 在 `X-GitHub-Api-Version` 标头中传递 API 版本。 有关详细信息，请参阅 `X-GitHub-Api-Version`。
  - **`Authorization`：** 在 `Authorization` 标头中传递身份验证令牌。 在大多数情况下，可以使用 `Authorization: Bearer` 或 `Authorization: token` 传递令牌。 但是，如果要传递 JSON Web 令牌 (JWT)，则必须使用 `Authorization: Bearer`。 有关详细信息，请参阅身份验证。 有关使用 `Authorization` 标头的请求示例，请参阅使用正文参数的示例请求。

- **`--data` 或 `-d`** 后跟 JSON 对象中的任何正文参数。 如果不需要在请求中指定任何正文参数，请忽略此选项。 有关详细信息，请参阅正文参数。 有关示例，请参阅使用正文参数的示例请求。

#### 示例请求

以下示例请求使用“获取 Octocat”终结点将 Octocat 返回为 ASCII 艺术。

Shell

    curl --request GET \
    --url "https://api.github.com/octocat" \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28"



    curl --request GET \
    --url "https://api.github.com/octocat" \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28"

#### 使用查询参数的示例请求

“列出公共事件”终结点默认返回 30 个问题。 以下示例使用 `per_page` 查询参数返回两个问题而不是 30 个，查询参数 `page` 仅提取结果的第一页。

Shell

    curl --request GET \
    --url "https://api.github.com/events?per_page=2&page=1" \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28" \
      https://api.github.com/events



    curl --request GET \
    --url "https://api.github.com/events?per_page=2&page=1" \
    --header "Accept: application/vnd.github+json" \
    --header "X-GitHub-Api-Version: 2022-11-28" \
      https://api.github.com/events

#### 使用正文参数的示例请求

以下示例使用创建议题终结点在 octocat/Spoon-Knife 仓库中创建新的议题。将 `YOUR-TOKEN` 替换为在上一步中创建的身份验证令牌。

注意

如果使用的是 fine-grained personal access token，则必须将 `octocat/Spoon-Knife` 替换为属于你或所在组织的存储库。 令牌必须有权访问该存储库，并且对存储库问题具有读取和写入权限。 有关详细信息，请参阅“管理个人访问令牌”。

Shell

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

本节演示如何使用 JavaScript 和 Octokit.js 向 GitHub REST API 发出请求。 有关更详细的指南，请参阅 使用 REST API 和 JavaScript 编写脚本。

### 1\. 设置

须安装 `octokit` 才能使用以下示例中所示的 Octokit.js 库。

- 安装 `octokit`。 例如，`npm install octokit`。 有关安装或加载 `octokit` 的其他方式，请参阅 Octokit.js 自述文件。

### 2\. 为请求选择终结点

1. 选择要向其发出请求的终结点。 可以浏览 GitHub 的 REST API 文档，了解可用于与 GitHub 交互的终结点。

2. 标识终结点的 HTTP 方法和路径。 将随请求一起发送这些内容。 有关详细信息，请参阅 HTTP 方法和路径。

例如，“创建问题”终结点使用 HTTP 方法和 `POST` 路径 `/repos/{owner}/{repo}/issues`。

3. 标识任何必需的路径参数。 必需的路径参数显示在终结点路径的大括号 `{}` 中。 将每个参数占位符替换为想要的值。 有关详细信息，请参阅 Path。

例如，“创建问题”终结点使用路径 `/repos/{owner}/{repo}/issues`，路径参数为 `{owner}` 和 `{repo}`。 要在 API 请求中使用此路径，请将 `{repo}` 替换为想要创建新问题的存储库的名称，并将 `{owner}` 替换为存储库所有者帐户的名称。

### 3\. 创建访问令牌。

创建访问令牌对请求进行身份验证。 可以保存令牌并将其用于多个请求。 为令牌提供访问终结点所需的任何作用域或权限。 将会在 `Authorization` 标头中与请求一起发送此令牌。 有关详细信息，请参阅身份验证。

### 4\. 使用 Octokit.js 发出请求

1. 在脚本中导入 `octokit`。 例如，`import { Octokit } from "octokit";`。 有关导入 `octokit` 的其他方式，请参阅 Octokit.js 自述文件。

2. 首先，使用令牌创建 `Octokit` 的实例。 将 `YOUR-TOKEN` 替换为令牌。

JavaScript

         const octokit = new Octokit({
           auth: 'YOUR-TOKEN'
         });


         const octokit = new Octokit({
           auth: 'YOUR-TOKEN'
         });


3. 使用 `octokit.request` 执行请求。
   - 将 HTTP 方法和路径作为 `request` 方法的第一个参数发送。 有关详细信息，请参阅 HTTP 方法和路径。
   - 将对象中的所有路径、查询和正文参数指定为 `request` 方法的第二个参数。 有关详细信息，请参阅参数。

在以下示例请求中，HTTP 方法为 `POST`，路径为 `/repos/{owner}/{repo}/issues`，路径参数为 `owner: "octocat"` 和 `repo: "Spoon-Knife"`，正文参数为 `title: "Created with the REST API"` 和 `body: "This is a test issue created by the REST API"`。

注意

如果使用的是 fine-grained personal access token，则必须将 `octocat/Spoon-Knife` 替换为属于你或所在组织的存储库。 令牌必须有权访问该存储库，并且对存储库问题具有读取和写入权限。 有关详细信息，请参阅“管理个人访问令牌”。

JavaScript

    await octokit.request("POST /repos/{owner}/{repo}/issues", {
      owner: "octocat",
      repo: "Spoon-Knife",
      title: "Created with the REST API",
      body: "This is a test issue created by the REST API",
    });


    await octokit.request("POST /repos/{owner}/{repo}/issues", {
      owner: "octocat",
      repo: "Spoon-Knife",
      title: "Created with the REST API",
      body: "This is a test issue created by the REST API",
    });

`request` 方法会自动传递 `Accept: application/vnd.github+json` 标头。 若要传递其他标头或不同的 `Accept` 标头，请将 `headers` 属性添加到作为第二个参数传递的对象。 `headers` 属性的值是将标头名称作为键并将标头值作为值的对象。

例如，以下代码会发送值为 `text/plain` 的 `content-type` 标头和值为 `2022-11-28` 的 `X-GitHub-Api-Version` 标头。

JavaScript

    await octokit.request("GET /octocat", {
      headers: {
        "content-type": "text/plain",
        "X-GitHub-Api-Version": "2022-11-28",
      },
    });


    await octokit.request("GET /octocat", {
      headers: {
        "content-type": "text/plain",
        "X-GitHub-Api-Version": "2022-11-28",
      },
    });

## 使用响应

发出请求后，API 会返回响应状态代码、响应头，并可能返回响应正文。

### 关于响应代码和标头

每个请求都会返回 HTTP 状态代码，以指示响应是否成功。 有关响应代码的详细信息，请参阅 MDN HTTP 响应状态代码文档。

此外，响应会包含标头，以提供有关响应的更多详细信息。 以 `X-` 或 `x-` 开头的标头对于 GitHub 是自定义的。 例如，`x-ratelimit-remaining` 和 `x-ratelimit-reset` 标头会告知你在一段时间内可以发出的请求数。

要查看状态代码和标头，请在发送请求时使用 `--include` 或 `--i` 选项。

例如，此请求获取在 octocat/Spoon-Knife 存储库中的问题列表：

    gh api \
    --header 'Accept: application/vnd.github+json' \
    --method GET /repos/octocat/Spoon-Knife/issues \
    -F per_page=2 --include

它会返回如下所示的响应代码和标头：

    HTTP/2.0 200 OK
    Access-Control-Allow-Origin: *
    Access-Control-Expose-Headers: ETag, Link, Location, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Used, X-RateLimit-Resource, X-RateLimit-Reset, X-OAuth-Scopes, X-Accepted-OAuth-Scopes, X-Poll-Interval, X-GitHub-Media-Type, X-GitHub-SSO, X-GitHub-Request-Id, Deprecation, Sunset
    Cache-Control: private, max-age=60, s-maxage=60
    Content-Security-Policy: default-src 'none'
    Content-Type: application/json; charset=utf-8
    Date: Thu, 04 Aug 2022 19:56:41 GMT
    Etag: W/"a63dfbcfdb73621e9d2e89551edcf9856731ced534bd7f1e114a5da1f5f73418"
    Link: <https://api.github.com/repositories/1300192/issues?per_page=1&page=2>; rel="next", <https://api.github.com/repositories/1300192/issues?per_page=1&page=14817>; rel="last"
    Referrer-Policy: origin-when-cross-origin, strict-origin-when-cross-origin
    Server: GitHub.com
    Strict-Transport-Security: max-age=31536000; includeSubdomains; preload
    Vary: Accept, Authorization, Cookie, Accept-Encoding, Accept, X-Requested-With
    X-Accepted-Oauth-Scopes: repo
    X-Content-Type-Options: nosniff
    X-Frame-Options: deny
    X-Github-Api-Version-Selected: 2022-08-09
    X-Github-Media-Type: github.v3; format=json
    X-Github-Request-Id: 1C73:26D4:E2E500:1EF78F4:62EC2479
    X-Oauth-Client-Id: 178c6fc778ccc68e1d6a
    X-Oauth-Scopes: gist, read:org, repo, workflow
    X-Ratelimit-Limit: 15000
    X-Ratelimit-Remaining: 14996
    X-Ratelimit-Reset: 1659645499
    X-Ratelimit-Resource: core
    X-Ratelimit-Used: 4
    X-Xss-Protection: 0

在此示例中，响应代码为 `200`，指示请求成功。

使用 Octokit.js 发出请求时，`request` 方法会返回承诺。 如果请求成功，则承诺会解析为包含响应的 HTTP 状态代码 (`status`) 和响应标头 (`headers`) 的对象。 如果发生错误，则承诺会解析为包含响应的 HTTP 状态代码 (`status`) 和响应标头 (`response.headers`) 的对象。

如果发生错误，则可以使用 `try/catch` 块进行捕获。 例如，如果以下脚本中的请求成功，则脚本会记录状态代码和 `x-ratelimit-remaining` 标头的值。 如果请求未成功，脚本会记录状态代码、标头的 `x-ratelimit-remaining` 值和错误消息。

在以下示例中，将 `REPO-OWNER` 替换为存储库所有者的帐户的名称，并将 `REPO-NAME` 替换为存储库的名称。

JavaScript

    try {
      const result = await octokit.request("GET /repos/{owner}/{repo}/issues", {
        owner: "REPO-OWNER",
        repo: "REPO-NAME",
        per_page: 2,
      });

      console.log(`Success! Status: ${result.status}. Rate limit remaining: ${result.headers["x-ratelimit-remaining"]}`)

    } catch (error) {
      console.log(`Error! Status: ${error.status}. Rate limit remaining: ${error.headers["x-ratelimit-remaining"]}. Message: ${error.response.data.message}`)
    }



    try {
      const result = await octokit.request("GET /repos/{owner}/{repo}/issues", {
        owner: "REPO-OWNER",
        repo: "REPO-NAME",
        per_page: 2,
      });

      console.log(`Success! Status: ${result.status}. Rate limit remaining: ${result.headers["x-ratelimit-remaining"]}`)

    } catch (error) {
      console.log(`Error! Status: ${error.status}. Rate limit remaining: ${error.headers["x-ratelimit-remaining"]}. Message: ${error.response.data.message}`)
    }

要查看状态代码和标头，请在发送请求时使用 `--include` 或 `--i` 选项。

例如，此请求获取在 octocat/Spoon-Knife 存储库中的问题列表：

    curl --request GET \
    --url "https://api.github.com/repos/octocat/Spoon-Knife/issues?per_page=2" \
    --header "Accept: application/vnd.github+json" \
    --header "Authorization: Bearer YOUR-TOKEN" \
    --include

它会返回如下所示的响应代码和标头：

    HTTP/2 200
    server: GitHub.com
    date: Thu, 04 Aug 2022 20:07:51 GMT
    content-type: application/json; charset=utf-8
    cache-control: public, max-age=60, s-maxage=60
    vary: Accept, Accept-Encoding, Accept, X-Requested-With
    etag: W/"7fceb7e8c958d3ec4d02524b042578dcc7b282192e6c939070f4a70390962e18"
    x-github-media-type: github.v3; format=json
    link: <https://api.github.com/repositories/1300192/issues?per_page=2&sort=updated&direction=asc&page=2>; rel="next", <https://api.github.com/repositories/1300192/issues?per_page=2&sort=updated&direction=asc&page=7409>; rel="last"
    access-control-expose-headers: ETag, Link, Location, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Used, X-RateLimit-Resource, X-RateLimit-Reset, X-OAuth-Scopes, X-Accepted-OAuth-Scopes, X-Poll-Interval, X-GitHub-Media-Type, X-GitHub-SSO, X-GitHub-Request-Id, Deprecation, Sunset
    access-control-allow-origin: *
    strict-transport-security: max-age=31536000; includeSubdomains; preload
    x-frame-options: deny
    x-content-type-options: nosniff
    x-xss-protection: 0
    referrer-policy: origin-when-cross-origin, strict-origin-when-cross-origin
    content-security-policy: default-src 'none'
    x-ratelimit-limit: 15000
    x-ratelimit-remaining: 14996
    x-ratelimit-reset: 1659645535
    x-ratelimit-resource: core
    x-ratelimit-used: 4
    accept-ranges: bytes
    content-length: 4936
    x-github-request-id: 14E0:4BC6:F1B8BA:208E317:62EC2715

在此示例中，响应代码为 `200`，指示请求成功。

### 关于响应正文

许多终结点会返回响应正文。 除非另外指定，否则响应正文会采用 JSON 格式。 空白字段包含为 `null`，而不是被省略。 所有时间戳以 ISO 8601 格式返回 UTC 时间：`YYYY-MM-DDTHH:MM:SSZ`。

与指定所需信息的 GraphQL API 不同，REST API 通常会返回比所需信息更多的信息。 如果需要，可以分析响应以拉取特定信息片段。

例如，可使用 `>` 将响应重定向到文件。 在以下示例中，将 `REPO-OWNER` 替换为存储库所有者的帐户的名称，并将 `REPO-NAME` 替换为存储库的名称。

Shell

    gh api \
    --header 'Accept: application/vnd.github+json' \
    --method GET /repos/REPO-OWNER/REPO-NAME/issues \
    -F per_page=2 > data.json



    gh api \
    --header 'Accept: application/vnd.github+json' \
    --method GET /repos/REPO-OWNER/REPO-NAME/issues \
    -F per_page=2 > data.json

然后可以使用 jq 获取每个问题的标题和创建者 ID：

Shell

    jq '.[] | {title: .title, authorID: .user.id}' data.json



    jq '.[] | {title: .title, authorID: .user.id}' data.json

前面两个命令返回类似于下面这样的内容：

    {
      "title": "Update index.html",
      "authorID": 10701255
    }
    {
      "title": "Edit index file",
      "authorID": 53709285
    }

有关 jq 的详细信息，请参阅 jq 文档。

例如，可以获取每个问题的标题和创建者 ID： 在以下示例中，将 `REPO-OWNER` 替换为存储库所有者的帐户的名称，并将 `REPO-NAME` 替换为存储库的名称。

JavaScript

    try {
      const result = await octokit.request("GET /repos/{owner}/{repo}/issues", {
        owner: "REPO-OWNER",
        repo: "REPO-NAME",
        per_page: 2,
      });

      const titleAndAuthor = result.data.map(issue => {title: issue.title, authorID: issue.user.id})

      console.log(titleAndAuthor)

    } catch (error) {
      console.log(`Error! Status: ${error.status}. Message: ${error.response.data.message}`)
    }



    try {
      const result = await octokit.request("GET /repos/{owner}/{repo}/issues", {
        owner: "REPO-OWNER",
        repo: "REPO-NAME",
        per_page: 2,
      });

      const titleAndAuthor = result.data.map(issue => {title: issue.title, authorID: issue.user.id})

      console.log(titleAndAuthor)

    } catch (error) {
      console.log(`Error! Status: ${error.status}. Message: ${error.response.data.message}`)
    }

例如，可使用 `>` 将响应重定向到文件。 在以下示例中，将 `REPO-OWNER` 替换为存储库所有者帐户的名称，并将 `REPO-NAME` 替换为存储库的名称。

Shell

    curl --request GET \
    --url "https://api.github.com/repos/REPO-OWNER/REPO-NAME/issues?per_page=2" \
    --header "Accept: application/vnd.github+json" \
    --header "Authorization: Bearer YOUR-TOKEN" > data.json



    curl --request GET \
    --url "https://api.github.com/repos/REPO-OWNER/REPO-NAME/issues?per_page=2" \
    --header "Accept: application/vnd.github+json" \
    --header "Authorization: Bearer YOUR-TOKEN" > data.json

然后可以使用 jq 获取每个问题的标题和创建者 ID：

Shell

    jq '.[] | {title: .title, authorID: .user.id}' data.json



    jq '.[] | {title: .title, authorID: .user.id}' data.json

前面两个命令返回类似于下面这样的内容：

    {
      "title": "Update index.html",
      "authorID": 10701255
    }
    {
      "title": "Edit index file",
      "authorID": 53709285
    }

有关 jq 的详细信息，请参阅 jq 文档。

#### 详细表示形式与摘要表示形式

响应可以包含资源的所有属性，也可以仅包含属性的子集，具体取决于是提取单个资源还是资源列表。

- 提取具体某个存储库等这样的*单个资源*时，响应通常会包含该资源的所有属性。 这就是资源的“详细”表示形式。
- 提取*资源列表*（如多个存储库的列表）时，响应将仅包含每个资源的属性子集。 这就是资源的“摘要”表示形式。

请注意，授权有时会影响表示形式中包含的详细信息量。

这是因为 API 提供的某些属性的计算成本很高，因此 GitHub 会从摘要表示形式中排除这些属性。 要获得这些属性，可以提取详细表示形式。

本文档提供每种 API 方法的示例响应。 示例响应说明了该方法返回的所有属性。

#### 超媒体

所有资源都可以具有一个或多个链接到其他资源的 `*_url` 属性。 这些属性旨在提供明确的 URL，使适当的 API 客户端不需要自己构建 URL。 强烈建议 API 客户端使用这些属性。 这样做有助于开发者未来更容易升级 API。 所有 URL 都应该是适当的 RFC 6570 URI 模板。

然后，可以使用 uri_template gem 之类的内容来扩展这些模板：

    >> tmpl = URITemplate.new('/notifications{?since,all,participating}')
    >> tmpl.expand
    => "/notifications"

    >> tmpl.expand all: 1
    => "/notifications?all=1"

    >> tmpl.expand all: 1, participating: 1
    => "/notifications?all=1&participating=1"

## 后续步骤

本文演示了如何在存储库中列出和创建问题。 有关更多做法，请尝试对问题添加注释、编辑问题的标题或关闭问题。 有关详细信息，请参阅“创建问题注释”终结点和“更新问题”终结点。

有关可以使用的操作的详细信息，请参阅“REST 参考文档”。

## 帮助和支持

### 是否找到了所需的内容？

是 否

隐私策略

### 仍需帮助？

询问 GitHub 社区

联系支持人员

## Legal

此内容中的一些内容可能是机器翻译的或 AI 翻译的内容。

- © 2026 GitHub, Inc.
- 术语
- 隐私
- 状态
- 定价
- 专家服务
- 博客
