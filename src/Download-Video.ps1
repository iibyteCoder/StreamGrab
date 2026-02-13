<#
.SYNOPSIS
    M3U8 批量下载器 - 重构版

.DESCRIPTION
    本脚本专为 PowerShell 7.2+ 设计，提供清晰的控制台UI和批量下载功能。
    支持从配置文件读取设置，提供重试机制和详细的进度显示。

.PARAMETER InputFile
    输入文件路径，包含要下载的 M3U8 URL 列表

.PARAMETER OutputDir
    输出目录路径

.PARAMETER ConfigPath
    配置文件路径

.PARAMETER RetryCount
    失败重试次数

.EXAMPLE
    ./Download-Video.ps1
    ./Download-Video.ps1 -InputFile "./mylist.txt" -OutputDir "./downloads"

.NOTES
    版本: 6.0
    要求: PowerShell 7.2+, Windows Terminal (推荐)
#>

#Requires -Version 7.2

# ============================================================
# 参数定义
# ============================================================
param(
    [string]$InputFile,
    [string]$OutputDir,
    [string]$ConfigPath = "./config/settings.json",
    [int]$RetryCount,
    [switch]$SkipConfigCheck
)

# ============================================================
# 导入模块
# ============================================================
$modulePath = Split-Path $PSScriptRoot -Parent
Import-Module (Join-Path $modulePath "modules/UI.psm1") -Force
Import-Module (Join-Path $modulePath "modules/Config.psm1") -Force
Import-Module (Join-Path $modulePath "modules/Logger.psm1") -Force

# ============================================================
# 加载配置
# ============================================================
$config = Get-Config -Path $ConfigPath

# 命令行参数覆盖配置文件
if ($InputFile) { $config.paths.inputFile = $InputFile }
if ($OutputDir) { $config.paths.tempDir = $OutputDir }
if ($RetryCount) { $config.download.retryCount = $RetryCount }

# 简化变量访问
$InputFile = $config.paths.inputFile
$OutputDir = $config.paths.tempDir
$Downloader = $config.tools.downloader
$uiConfig = $config.ui

# ============================================================
# 状态管理（使用线程安全集合）
# ============================================================
$script:state = @{
    TotalTasks     = 0
    CompletedTasks = 0
    SuccessCount   = 0
    FailureCount   = 0
    CurrentTasks   = [System.Collections.Concurrent.ConcurrentDictionary[string, string]]::new()
    Logs           = [System.Collections.Concurrent.ConcurrentQueue[string]]::new()
    StartTime      = [DateTime]::Now
}

# ============================================================
# 获取模块资源
# ============================================================
$theme = Get-Theme
$icons = Get-Icons
$ansi = Get-Ansi

# ============================================================
# 初始化日志
# ============================================================
Initialize-Logger -LogDir $config.paths.logDir -EnableFileLog $true

# ============================================================
# UI 函数
# ============================================================
function Initialize-UI {
    Write-Host $ansi.HideCursor
    Write-Title -Title "M3U8 批量下载器" -Version "6.0"
}

function Write-LogEntry {
    param(
        [string]$Message,
        [string]$Type = "Info",
        [string]$Details = ""
    )

    $timestamp = Get-Date -Format "HH:mm:ss"
    $typeColor = switch ($Type) {
        "Success" { $theme.Success }
        "Failure" { $theme.Failure }
        "Warning" { $theme.Warning }
        "Title"   { $theme.Title }
        "Accent"  { $theme.Accent }
        default   { $theme.Info }
    }

    $iconChar = switch ($Type) {
        "Success" { $icons.Success }
        "Failure" { $icons.Failure }
        "Warning" { "!" }
        "Title"   { "▶" }
        "Accent"  { "►" }
        default   { $icons.Info }
    }

    $logEntry = "$($theme.Dim)[$timestamp]$($theme.Reset) ${typeColor}${iconChar}$($theme.Reset) $Message"
    if ($Details) {
        $logEntry += "`n           $($theme.Dim)$Details$($theme.Reset)"
    }

    # 线程安全：使用 ConcurrentQueue
    $script:state.Logs.Enqueue($logEntry)

    # 限制日志行数
    while ($script:state.Logs.Count -gt $uiConfig.logLines) {
        $script:state.Logs.TryDequeue([ref]$null) | Out-Null
    }

    # 写入文件日志（线程安全）
    Write-Log -Message $Message -Level $(if ($Type -eq "Failure") { "Error" } else { "Info" }) -LogName "download"

    Update-UI
}

function Update-UI {
    # 保存光标位置
    Write-Host -NoNewline $ansi.SaveCursor

    # 移动到日志区域开始位置（标题下方）
    $logStartLine = 6
    Write-Host -NoNewline ($ansi.MoveTo -f $logStartLine, 0)

    # 清除日志区域（增加一行用于显示当前并发任务）
    $logAreaHeight = $uiConfig.logLines + 1
    for ($i = 0; $i -lt $logAreaHeight; $i++) {
        Write-Host $ansi.ClearLine
        if ($i -lt $logAreaHeight - 1) {
            Write-Host -NoNewline ($ansi.MoveDown -f 1)
        }
    }

    # 重新定位到日志区域开始
    Write-Host -NoNewline ($ansi.MoveTo -f $logStartLine, 0)

    # 显示当前正在执行的任务
    $currentTasksList = $script:state.CurrentTasks.Values | Where-Object { $_ } | ForEach-Object { $_ }
    if ($currentTasksList) {
        $currentTasksText = "$($theme.Dim)正在执行: $($theme.Reset)$($theme.Accent)$($currentTasksList -join ', ')$($theme.Reset)"
        Write-Host "$($ansi.ClearLine)$currentTasksText"
        Write-Host -NoNewline ($ansi.MoveDown -f 1)
    } else {
        Write-Host "$($ansi.ClearLine)$($theme.Dim)正在执行: 无$($theme.Reset)"
        Write-Host -NoNewline ($ansi.MoveDown -f 1)
    }

    # 输出日志（从 ConcurrentQueue 转换为数组）
    $logArray = $script:state.Logs.ToArray()
    foreach ($log in $logArray) {
        Write-Host "$($ansi.ClearLine)$log"
        Write-Host -NoNewline ($ansi.MoveDown -f 1)
    }

    # 显示进度条（固定在底部）
    $elapsed = [DateTime]::Now - $script:state.StartTime
    $progressLines = Show-ProgressBar `
        -Current $script:state.CompletedTasks `
        -Total $script:state.TotalTasks `
        -Success $script:state.SuccessCount `
        -Failure $script:state.FailureCount `
        -Width $uiConfig.progressBarWidth `
        -Elapsed $elapsed

    $hostHeight = $Host.UI.RawUI.WindowSize.Height
    $progressStartLine = $hostHeight - $uiConfig.progressHeight

    Write-Host -NoNewline ($ansi.MoveTo -f $progressStartLine, 0)

    foreach ($line in $progressLines) {
        Write-Host "$($ansi.ClearLine)$line"
        Write-Host -NoNewline ($ansi.MoveDown -f 1)
    }

    # 恢复光标位置
    Write-Host -NoNewline $ansi.RestoreCursor
}

function Close-UI {
    Write-Host $ansi.ShowCursor

    # 最终统计
    $totalTime = [DateTime]::Now - $script:state.StartTime

    Write-SummaryBox -Title "任务完成统计" -Items @{
        "总耗时" = $totalTime.ToString('hh\:mm\:ss')
        "成功"   = "$($script:state.SuccessCount) 个"
        "失败"   = "$($script:state.FailureCount) 个"
        "总计"   = "$($script:state.TotalTasks) 个任务"
    }
}

# ============================================================
# 核心下载函数（线程安全）
# ============================================================
function Invoke-DownloadTask {
    param(
        [string]$Url,
        [string]$SaveName,
        [int]$TaskNumber,
        [int]$TotalTasks,
        [hashtable]$SharedState,
        [hashtable]$Config,
        [string]$OutputDir,
        [string]$Downloader
    )

    $taskPad = $TaskNumber.ToString().PadLeft($TotalTasks.ToString().Length, '0')
    $taskId = "task_$TaskNumber"
    $taskLabel = if ($SaveName) { "[$taskPad/$TotalTasks] $SaveName" } else { "[$taskPad/$TotalTasks]" }

    # 注册当前任务
    $SharedState.CurrentTasks.TryAdd($taskId, $taskLabel) | Out-Null

    try {
        # 构建参数
        $arguments = @(
            $Url,
            "--save-dir", $OutputDir,
            "--auto-select"
        )

        if ($Config.download.noLog) {
            $arguments += "--no-log"
        }

        if ($SaveName) {
            $arguments += "--save-name", $SaveName
        }

        # 重试逻辑
        $maxRetries = $Config.download.retryCount
        $retryDelay = $Config.download.retryDelay
        $success = $false

        for ($attempt = 1; $attempt -le $maxRetries -and -not $success; $attempt++) {
            try {
                if ($attempt -gt 1) {
                    Start-Sleep -Seconds $retryDelay
                }

                # 执行下载
                & $Downloader $arguments 2>&1 | Out-Null
                $exitCode = $LASTEXITCODE

                if ($exitCode -eq 0) {
                    [System.Threading.Interlocked]::Increment([ref]$SharedState.SuccessCount) | Out-Null
                    Write-LogEntry -Message "$taskLabel 下载完成" -Type "Success"
                    $success = $true
                } else {
                    if ($attempt -eq $maxRetries) {
                        [System.Threading.Interlocked]::Increment([ref]$SharedState.FailureCount) | Out-Null
                        Write-LogEntry -Message "$taskLabel 下载失败 (退出码: $exitCode)" -Type "Failure"
                    }
                }
            } catch {
                if ($attempt -eq $maxRetries) {
                    [System.Threading.Interlocked]::Increment([ref]$SharedState.FailureCount) | Out-Null
                    Write-LogEntry -Message "$taskLabel 执行错误: $($_.Exception.Message)" -Type "Failure"
                }
            }
        }
    } finally {
        # 移除当前任务
        $SharedState.CurrentTasks.TryRemove($taskId, [ref]$null) | Out-Null
        [System.Threading.Interlocked]::Increment([ref]$SharedState.CompletedTasks) | Out-Null
    }
}

# ============================================================
# 主程序
# ============================================================

# 前置检查
if (-not (Test-Path $Downloader)) {
    Write-Status -Message "下载程序 '$Downloader' 不存在" -Type Failure
    exit 1
}

if (-not (Test-Path $InputFile)) {
    Write-Status -Message "输入文件 '$InputFile' 未找到" -Type Failure
    exit 1
}

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# 读取任务
$m3u8Lines = Get-Content $InputFile -Encoding UTF8 | Where-Object { $_.Trim() -ne '' -and -not $_.Trim().StartsWith('#') }
$script:state.TotalTasks = @($m3u8Lines).Count

if ($script:state.TotalTasks -eq 0) {
    Write-Status -Message "输入文件为空或没有有效的 URL" -Type Warning
    exit 0
}

# 初始化UI
Initialize-UI
Write-LogEntry -Message "发现 $($script:state.TotalTasks) 个下载任务" -Type "Accent"
Write-LogEntry -Message "输出目录: $OutputDir" -Type "Info"
Write-LogEntry -Message "重试次数: $($config.download.retryCount)" -Type "Info"
Write-LogEntry -Message "并发数: $($config.download.concurrent)" -Type "Info"

# 准备并行执行需要的变量
$sharedState = $script:state
$totalTasks = $script:state.TotalTasks
$downloader = $Downloader
$outputDir = $OutputDir

# 启动后台UI更新（因为并行执行不会自动更新UI）
$uiUpdateJob = Start-ThreadJob -ScriptBlock {
    param($state, $uiConfig, $theme, $icons, $ansi)

    function Show-ProgressBar {
        param($Current, $Total, $Success, $Failure, $Width, $Elapsed)

        $percent = if ($Total -gt 0) { $Current / $Total } else { 0 }
        $filled = [math]::Floor($percent * $Width)
        $empty = $Width - $filled

        $progressBar = "$($theme.Success)" + "█" * $filled + "$($theme.Dim)" + "░" * $empty + "$($theme.Reset)"

        $elapsedText = $Elapsed.ToString('hh\:mm\:ss')
        $rate = if ($Elapsed.TotalSeconds -gt 0) { [math]::Round($Current / $Elapsed.TotalSeconds, 1) } else { 0 }
        $eta = if ($rate -gt 0) { [TimeSpan]::FromSeconds(($Total - $Current) / $rate) } else { [TimeSpan]::Zero }
        $etaText = if ($eta.TotalHours -gt 1) { $eta.ToString('hh\:mm\:ss') } elseif ($eta.TotalMinutes -gt 1) { $eta.ToString('mm\:ss') } else { "$($eta.TotalSeconds.ToString('F0'))s" }

        $lines = @(
            "┌─ $elapsedText 已耗时 | 预计剩余: $etaText ─────────────────────────────┐"
            "│ $progressBar │"
            "└─ $Current/$Total 完成 │ $($theme.Success)+$Success$($theme.Reset) 成功 │ $($theme.Failure)-$Failure$($theme.Reset) 失败 ─┘"
        )

        return $lines
    }

    while ($state.CompletedTasks -lt $state.TotalTasks) {
        Start-Sleep -Milliseconds 200

        # 保存光标位置
        Write-Host -NoNewline $ansi.SaveCursor

        # 显示进度条
        $elapsed = [DateTime]::Now - $state.StartTime
        $progressLines = Show-ProgressBar `
            -Current $state.CompletedTasks `
            -Total $state.TotalTasks `
            -Success $state.SuccessCount `
            -Failure $state.FailureCount `
            -Width $uiConfig.progressBarWidth `
            -Elapsed $elapsed

        $hostHeight = $Host.UI.RawUI.WindowSize.Height
        $progressStartLine = $hostHeight - $uiConfig.progressHeight

        Write-Host -NoNewline ($ansi.MoveTo -f $progressStartLine, 0)

        foreach ($line in $progressLines) {
            Write-Host "$($ansi.ClearLine)$line"
            Write-Host -NoNewline ($ansi.MoveDown -f 1)
        }

        # 恢复光标位置
        Write-Host -NoNewline $ansi.RestoreCursor
    }
} -ArgumentList $sharedState, $uiConfig, $theme, $icons, $ansi

# 执行下载（并发模式）
$concurrentCount = $config.download.concurrent
Write-LogEntry -Message "并发下载模式: 最多 $concurrentCount 个任务同时执行" -Type "Accent"

# 准备任务列表
$tasks = @()
$taskNumber = 0
foreach ($line in $m3u8Lines) {
    $taskNumber++
    $parts = $line.Trim() -split '\s+', 2
    $tasks += [PSCustomObject]@{
        Number   = $taskNumber
        Url      = $parts[0]
        SaveName = if ($parts.Length -ge 2) { $parts[1].Trim() } else { $null }
    }
}

# 使用 ForEach-Object -Parallel 实现并发下载
$tasks | ForEach-Object -ThrottleLimit $concurrentCount -Parallel {
    # 导入需要的模块
    $mp = $USING:modulePath
    Import-Module (Join-Path $mp "modules/UI.psm1") -Force
    Import-Module (Join-Path $mp "modules/Logger.psm1") -Force

    # 调用下载函数
    Invoke-DownloadTask `
        -Url $_.Url `
        -SaveName $_.SaveName `
        -TaskNumber $_.Number `
        -TotalTasks $USING:totalTasks `
        -SharedState $USING:sharedState `
        -Config $USING:config `
        -OutputDir $USING:outputDir `
        -Downloader $USING:downloader
}

# 停止后台UI更新作业
if ($uiUpdateJob) {
    $uiUpdateJob | Stop-Job
    $uiUpdateJob | Remove-Job
}

# 完成
Close-UI

# 音效提示
if ($uiConfig.beepOnComplete) {
    if ($script:state.FailureCount -gt 0) {
        Invoke-Beep -Type Error
    } else {
        Invoke-Beep -Type Success
    }
}

exit $(if ($script:state.FailureCount -gt 0) { 1 } else { 0 })
