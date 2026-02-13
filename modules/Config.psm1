<#
.SYNOPSIS
    配置模块 - 统一的配置管理
.DESCRIPTION
    提供配置加载、保存、验证等功能
.NOTES
    版本: 1.0.0
    要求: PowerShell 7.2+
#>

#Requires -Version 7.2

# ============================================================
# 模块变量
# ============================================================
$script:ConfigPath = "./config/settings.json"
$script:Config = $null

# ============================================================
# 默认配置
# ============================================================
$script:DefaultConfig = @{
    version = "1.0.0"
    paths = @{
        inputFile  = "./m3u8.txt"
        tempDir    = "./video"
        outputDir  = "./output"
        logDir     = "./logs"
        workDir    = "./.work"
    }
    tools = @{
        downloader = "./N_m3u8DL-RE.exe"
        ffmpeg     = "./ffmpeg.exe"
    }
    download = @{
        autoSelect  = $true
        concurrent  = 1
        retryCount  = 3
        retryDelay  = 5
        timeout     = 300
        noLog       = $true
    }
    merge = @{
        codec     = "copy"
        overwrite = $true
        videoExt  = ".mp4"
        audioExt  = ".m4a"
    }
    clean = @{
        targets       = @("Logs", "video", ".work")
        includeOutput = $false
    }
    ui = @{
        showProgress     = $true
        progressHeight   = 4
        logLines         = 12
        progressBarWidth = 50
        beepOnComplete   = $true
        beepOnError      = $true
    }
}

# ============================================================
# 公共函数
# ============================================================

function Get-Config {
    <#
    .SYNOPSIS
        获取当前配置
    .PARAMETER Path
        配置文件路径（可选，不提供则使用默认路径）
    .PARAMETER Reload
        强制重新加载配置
    #>
    param(
        [string]$Path,
        [switch]$Reload
    )

    if ($Path) {
        $script:ConfigPath = $Path
    }

    if ($script:Config -and -not $Reload) {
        return $script:Config
    }

    if (Test-Path $script:ConfigPath) {
        try {
            $json = Get-Content $script:ConfigPath -Raw -Encoding UTF8
            $script:Config = $json | ConvertFrom-Json -AsHashtable

            # 合并默认值（处理缺失的配置项）
            $script:Config = Merge-Config -Base $script:DefaultConfig -Override $script:Config

            # 将工具路径转换为绝对路径
            $script:Config = Resolve-ConfigPaths -Config $script:Config
        } catch {
            Write-Warning "配置文件加载失败: $($_.Exception.Message)，使用默认配置"
            $script:Config = $script:DefaultConfig.Clone()
        }
    } else {
        $script:Config = $script:DefaultConfig.Clone()
        Save-Config -Config $script:Config
    }

    return $script:Config
}

function Resolve-ConfigPaths {
    <#
    .SYNOPSIS
        将配置中的相对路径转换为绝对路径
    #>
    param([hashtable]$Config)

    # 获取项目根目录（包含 run.ps1 的目录）
    $rootDir = if ($PSScriptRoot) {
        $PSScriptRoot
    } else {
        $PWD.Path
    }

    # 解析工具路径
    if ($Config.tools.downloader -and -not [System.IO.Path]::IsPathRooted($Config.tools.downloader)) {
        $Config.tools.downloader = Join-Path $rootDir $Config.tools.downloader
    }
    if ($Config.tools.ffmpeg -and -not [System.IO.Path]::IsPathRooted($Config.tools.ffmpeg)) {
        $Config.tools.ffmpeg = Join-Path $rootDir $Config.tools.ffmpeg
    }

    # 解析其他路径
    foreach ($key in $Config.paths.Keys) {
        if ($Config.paths[$key] -and -not [System.IO.Path]::IsPathRooted($Config.paths[$key])) {
            $Config.paths[$key] = Join-Path $rootDir $Config.paths[$key]
        }
    }

    return $Config
}

function Merge-Config {
    <#
    .SYNOPSIS
        递归合并配置（深度合并）
    .PARAMETER Base
        基础配置
    .PARAMETER Override
        覆盖配置
    #>
    param(
        [hashtable]$Base,
        [hashtable]$Override
    )

    $result = $Base.Clone()

    foreach ($key in $Override.Keys) {
        if ($result.ContainsKey($key) -and $result[$key] -is [hashtable] -and $Override[$key] -is [hashtable]) {
            $result[$key] = Merge-Config -Base $result[$key] -Override $Override[$key]
        } else {
            $result[$key] = $Override[$key]
        }
    }

    return $result
}

function Save-Config {
    <#
    .SYNOPSIS
        保存配置到文件
    .PARAMETER Config
        要保存的配置（可选，不提供则保存当前配置）
    .PARAMETER Path
        保存路径（可选）
    #>
    param(
        [hashtable]$Config,
        [string]$Path
    )

    if (-not $Config) {
        $Config = $script:Config
    }

    if (-not $Config) {
        $Config = $script:DefaultConfig.Clone()
    }

    $savePath = if ($Path) { $Path } else { $script:ConfigPath }

    # 确保目录存在
    $dir = Split-Path $savePath -Parent
    if ($dir -and -not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }

    # 保存配置
    $json = $Config | ConvertTo-Json -Depth 10
    $json | Out-File -FilePath $savePath -Encoding UTF8 -Force
}

function Set-ConfigValue {
    <#
    .SYNOPSIS
        设置配置项
    .PARAMETER Key
        配置键（支持点号分隔的路径，如 "ui.beepOnComplete"）
    .PARAMETER Value
        配置值
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Key,

        [Parameter(Mandatory)]
        $Value
    )

    $config = Get-Config
    $keys = $Key -split '\.'
    $current = $config

    for ($i = 0; $i -lt $keys.Count - 1; $i++) {
        $k = $keys[$i]
        if (-not $current.ContainsKey($k)) {
            $current[$k] = @{}
        }
        $current = $current[$k]
    }

    $current[$keys[-1]] = $Value
    $script:Config = $config
}

function Get-ConfigValue {
    <#
    .SYNOPSIS
        获取配置项
    .PARAMETER Key
        配置键（支持点号分隔的路径）
    .PARAMETER Default
        默认值（如果键不存在）
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Key,

        $Default = $null
    )

    $config = Get-Config
    $keys = $Key -split '\.'
    $current = $config

    foreach ($k in $keys) {
        if ($current -is [hashtable] -and $current.ContainsKey($k)) {
            $current = $current[$k]
        } else {
            return $Default
        }
    }

    return $current
}

function Test-Config {
    <#
    .SYNOPSIS
        验证配置（检查必要的工具和路径）
    .PARAMETER Config
        要验证的配置
    #>
    param([hashtable]$Config)

    if (-not $Config) {
        $Config = Get-Config
    }

    $errors = @()

    # 检查下载器
    if (-not (Test-Path $Config.tools.downloader)) {
        $errors += "下载器不存在: $($Config.tools.downloader)"
    }

    # 检查 FFmpeg
    if (-not (Test-Path $Config.tools.ffmpeg)) {
        $errors += "FFmpeg 不存在: $($Config.tools.ffmpeg)"
    }

    # 检查输入文件
    if (-not (Test-Path $Config.paths.inputFile)) {
        $errors += "输入文件不存在: $($Config.paths.inputFile)"
    }

    return @{
        Valid  = $errors.Count -eq 0
        Errors = $errors
    }
}

function Reset-Config {
    <#
    .SYNOPSIS
        重置配置为默认值
    #>
    $script:Config = $script:DefaultConfig.Clone()
    Save-Config
}

# 导出模块成员
Export-ModuleMember -Function @(
    'Get-Config',
    'Save-Config',
    'Set-ConfigValue',
    'Get-ConfigValue',
    'Test-Config',
    'Reset-Config',
    'Resolve-ConfigPaths'
)
