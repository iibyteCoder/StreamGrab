<#
.SYNOPSIS
    日志模块 - 统一的日志记录功能
.DESCRIPTION
    提供文件日志、内存日志、日志轮转等功能
.NOTES
    版本: 1.0.0
    要求: PowerShell 7.2+
#>

#Requires -Version 7.2

# ============================================================
# 模块变量
# ============================================================
$script:LogConfig = @{
    EnableFileLog = $true
    LogDir        = "./logs"
    MaxLogSize    = 10MB
    MaxLogFiles   = 10
    LogLevel      = "Debug"  # Debug, Info, Warning, Error
}

$script:LogBuffer = [System.Collections.Generic.List[string]]::new()
$script:MaxBufferSize = 1000

# ============================================================
# 私有函数
# ============================================================
function Get-LogFilePath {
    param([string]$LogName = "main")
    $date = Get-Date -Format "yyyyMMdd"
    return Join-Path $script:LogConfig.LogDir "${LogName}_${date}.log"
}

function Test-LogRotation {
    param([string]$LogPath)

    if (-not (Test-Path $LogPath)) { return }

    $file = Get-Item $LogPath
    if ($file.Length -gt $script:LogConfig.MaxLogSize) {
        $timestamp = Get-Date -Format "HHmmss"
        $newName = $LogPath -replace '\.log$', "_${timestamp}.log"
        Rename-Item $LogPath $newName -Force

        # 清理旧日志
        $logFiles = Get-ChildItem (Split-Path $LogPath) -Filter "*.log" |
        Sort-Object LastWriteTime -Descending |
        Select-Object -Skip $script:LogConfig.MaxLogFiles
        $logFiles | Remove-Item -Force
    }
}

# ============================================================
# 公共函数
# ============================================================

function Initialize-Logger {
    <#
    .SYNOPSIS
        初始化日志系统
    .PARAMETER LogDir
        日志目录
    .PARAMETER EnableFileLog
        是否启用文件日志
    #>
    param(
        [string]$LogDir = "./logs",
        [bool]$EnableFileLog = $true
    )

    $script:LogConfig.LogDir = $LogDir
    $script:LogConfig.EnableFileLog = $EnableFileLog

    if ($EnableFileLog -and -not (Test-Path $LogDir)) {
        New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
    }
}

function Write-Log {
    <#
    .SYNOPSIS
        写入日志
    .PARAMETER Message
        日志消息
    .PARAMETER Level
        日志级别: Debug, Info, Warning, Error
    .PARAMETER LogName
        日志文件名（不含扩展名）
    .PARAMETER NoConsole
        不输出到控制台
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Message,

        [ValidateSet('Debug', 'Info', 'Warning', 'Error')]
        [string]$Level = 'Info',

        [string]$LogName = "main",

        [switch]$NoConsole
    )

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"12
    $logEntry = "[$timestamp] [$Level] $Message"

    # 添加到内存缓冲区
    $script:LogBuffer.Add($logEntry)
    if ($script:LogBuffer.Count -gt $script:MaxBufferSize) {
        $script:LogBuffer.RemoveAt(0)
    }

    # 写入文件
    if ($script:LogConfig.EnableFileLog) {
        $logPath = Get-LogFilePath -LogName $LogName
        Test-LogRotation -LogPath $logPath
        Add-Content -Path $logPath -Value $logEntry -Encoding UTF8
    }

    # 控制台输出
    if (-not $NoConsole) {
        $color = switch ($Level) {
            'Error' { 'Red' }
            'Warning' { 'Yellow' }
            'Debug' { 'Gray' }
            default { 'White' }
        }
        # 控制台输出由 UI 模块的 Write-Status 处理，这里只记录
    }
}

function Get-LogBuffer {
    <#
    .SYNOPSIS
        获取内存中的日志缓冲区
    .PARAMETER Count
        获取最近 N 条日志
    #>
    param([int]$Count = 100)

    if ($Count -ge $script:LogBuffer.Count) {
        return $script:LogBuffer.ToArray()
    }
    return $script:LogBuffer.GetRange($script:LogBuffer.Count - $Count, $Count).ToArray()
}

function Clear-LogBuffer {
    <#
    .SYNOPSIS
        清空日志缓冲区
    #>
    $script:LogBuffer.Clear()
}

function Export-LogBuffer {
    <#
    .SYNOPSIS
        导出日志缓冲区到文件
    .PARAMETER Path
        导出路径
    #>
    param([string]$Path)

    $script:LogBuffer | Out-File -FilePath $Path -Encoding UTF8
}

# 导出模块成员
Export-ModuleMember -Function @(
    'Initialize-Logger',
    'Write-Log',
    'Get-LogBuffer',
    'Clear-LogBuffer',
    'Export-LogBuffer'
)
