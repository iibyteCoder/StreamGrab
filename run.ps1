#!/usr/bin/env pwsh
<#
.SYNOPSIS
    M3U8 下载管理器 - 主入口脚本

.DESCRIPTION
    统一的命令行入口，支持交互式菜单和命令行参数两种使用方式。
    整合下载、合并、清理功能于一体。

.PARAMETER Action
    操作类型:
    - download: 下载视频
    - merge: 合并音视频
    - clean: 清理临时文件
    - all: 执行下载 + 合并
    - menu: 显示交互式菜单（默认）

.PARAMETER Config
    配置文件路径

.PARAMETER NoBeep
    禁用音效提示

.EXAMPLE
    ./run.ps1
    ./run.ps1 -Action download
    ./run.ps1 -Action all -NoBeep

.NOTES
    版本: 1.0.0
    要求: PowerShell 7.2+
#>

#Requires -Version 7.2

param(
    [Parameter(Position=0)]
    [ValidateSet('download', 'merge', 'clean', 'all', 'menu', 'config')]
    [string]$Action = 'menu',

    [string]$Config = './config/settings.json',
    [switch]$NoBeep
)

# ============================================================
# 获取脚本目录
# ============================================================
$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { $PWD.Path }

# ============================================================
# 导入模块
# ============================================================
Import-Module (Join-Path $ScriptDir "modules/UI.psm1") -Force
Import-Module (Join-Path $ScriptDir "modules/Config.psm1") -Force

# ============================================================
# 加载配置
# ============================================================
$settings = Get-Config -Path $Config

if ($NoBeep) {
    $settings.ui.beepOnComplete = $false
    $settings.ui.beepOnError = $false
}

# ============================================================
# 获取模块资源
# ============================================================
$theme = Get-Theme
$icons = Get-Icons

# ============================================================
# 脚本路径
# ============================================================
$scripts = @{
    Download = Join-Path $ScriptDir "src/Download-Video.ps1"
    Merge    = Join-Path $ScriptDir "src/Merge-AudioVideo.ps1"
    Clean    = Join-Path $ScriptDir "src/Clear-Workspace.ps1"
}

# ============================================================
# 辅助函数
# ============================================================
function Show-Banner {
    Clear-Host
    Write-Title -Title "M3U8 下载管理器" -Version "1.0"
    Write-Host "$($theme.Info)一个简洁高效的 M3U8 视频批量下载工具$($theme.Reset)"
    Write-Host ""
}

function Show-Menu {
    Show-Banner

    Write-Host "$($theme.Accent)可用操作:$($theme.Reset)"
    Write-Host ""
    Write-Host "  $($theme.Success)1.$($theme.Reset) 下载视频      - 下载并整理到 output 目录"
    Write-Host "  $($theme.Success)2.$($theme.Reset) 整理输出      - 合并/移动视频到 output，清理临时文件"
    Write-Host "  $($theme.Success)3.$($theme.Reset) 清理所有      - 删除 video、output、logs 等目录"
    Write-Host "  $($theme.Success)4.$($theme.Reset) 一键执行      - 下载 + 整理"
    Write-Host "  $($theme.Success)5.$($theme.Reset) 查看配置      - 显示当前配置"
    Write-Host "  $($theme.Failure)0.$($theme.Reset) 退出"
    Write-Host ""
}

function Show-ConfigInfo {
    Show-Banner

    Write-Host "$($theme.Title)当前配置:$($theme.Reset)"
    Write-Host ""
    Write-Host "  $($theme.Info)输入文件:$($theme.Reset)   $($settings.paths.inputFile)"
    Write-Host "  $($theme.Info)临时目录:$($theme.Reset)   $($settings.paths.tempDir)"
    Write-Host "  $($theme.Info)输出目录:$($theme.Reset)   $($settings.paths.outputDir)"
    Write-Host "  $($theme.Info)日志目录:$($theme.Reset)   $($settings.paths.logDir)"
    Write-Host ""
    Write-Host "  $($theme.Info)下载器:$($theme.Reset)     $($settings.tools.downloader)"
    Write-Host "  $($theme.Info)FFmpeg:$($theme.Reset)      $($settings.tools.ffmpeg)"
    Write-Host ""
    Write-Host "  $($theme.Info)重试次数:$($theme.Reset)   $($settings.download.retryCount)"
    Write-Host "  $($theme.Info)并行下载数:$($theme.Reset) $($settings.download.concurrent)"
    Write-Host ""
    Write-Host "  $($theme.Info)配置文件:$($theme.Reset)   $Config"
    Write-Host ""

    Write-Host "$($theme.Dim)按任意键返回菜单...$($theme.Reset)"
    [void][System.Console]::ReadKey($true)
}

function Invoke-Download {
    Write-Status -Message "开始执行下载任务..." -Type Title
    Write-Host ""

    $result = & $scripts.Download -ConfigPath $Config
    return $LASTEXITCODE
}

function Invoke-Merge {
    Write-Status -Message "开始执行合并任务..." -Type Title
    Write-Host ""

    $result = & $scripts.Merge -ConfigPath $Config
    return $LASTEXITCODE
}

function Invoke-Clean {
    Write-Status -Message "开始执行清理任务..." -Type Title
    Write-Host ""

    $result = & $scripts.Clean -ConfigPath $Config -Force
    return $LASTEXITCODE
}

function Invoke-All {
    $downloadResult = Invoke-Download
    Write-Host ""
    Write-SectionHeader
    Write-Host ""

    if ($downloadResult -ne 0) {
        Write-Status -Message "部分下载任务失败，但仍继续合并..." -Type Warning
        Write-Host ""
    }

    $mergeResult = Invoke-Merge

    Write-Host ""
    Write-SectionHeader -Title "全部完成"

    if ($downloadResult -eq 0 -and $mergeResult -eq 0) {
        Write-Status -Message "所有任务成功完成!" -Type Success
    } else {
        Write-Status -Message "部分任务存在问题，请检查日志" -Type Warning
    }

    return $mergeResult
}

# ============================================================
# 主逻辑
# ============================================================

# 检查脚本是否存在
foreach ($key in $scripts.Keys) {
    if (-not (Test-Path $scripts[$key])) {
        Write-Status -Message "脚本文件不存在: $($scripts[$key])" -Type Failure
        exit 1
    }
}

# 根据操作类型执行
switch ($Action) {
    'download' {
        $exitCode = Invoke-Download
    }
    'merge' {
        $exitCode = Invoke-Merge
    }
    'clean' {
        $exitCode = Invoke-Clean
    }
    'all' {
        $exitCode = Invoke-All
    }
    'config' {
        Show-ConfigInfo
        $exitCode = 0
    }
    'menu' {
        # 交互式菜单
        do {
            Show-Menu
            $choice = Read-Host "请选择操作"

            switch ($choice) {
                '1' {
                    Invoke-Download
                    Write-Host "`n$($theme.Dim)按任意键返回菜单...$($theme.Reset)"
                    [void][System.Console]::ReadKey($true)
                }
                '2' {
                    Invoke-Merge
                    Write-Host "`n$($theme.Dim)按任意键返回菜单...$($theme.Reset)"
                    [void][System.Console]::ReadKey($true)
                }
                '3' {
                    Invoke-Clean
                    Write-Host "`n$($theme.Dim)按任意键返回菜单...$($theme.Reset)"
                    [void][System.Console]::ReadKey($true)
                }
                '4' {
                    Invoke-All
                    Write-Host "`n$($theme.Dim)按任意键返回菜单...$($theme.Reset)"
                    [void][System.Console]::ReadKey($true)
                }
                '5' {
                    Show-ConfigInfo
                }
                '0' {
                    Write-Status -Message "再见!" -Type Info
                    $exitCode = 0
                    break
                }
                default {
                    Write-Status -Message "无效选择，请重试" -Type Warning
                    Start-Sleep -Seconds 1
                }
            }
        } while ($choice -ne '0')
    }
}

# 等待用户按键（非菜单模式下）
if ($Action -ne 'menu' -and $Action -ne 'config') {
    Write-Host ""
    Write-Status -Message "按任意键退出..." -Type Info
    [void][System.Console]::ReadKey($true)
}

exit $exitCode
