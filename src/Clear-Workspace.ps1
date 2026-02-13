<#
.SYNOPSIS
    工作目录清理脚本

.DESCRIPTION
    清理所有临时文件和输出目录：
    - video/   临时下载文件
    - output/  最终输出文件（用户已复制走）
    - Logs/    日志文件
    - .work/   工作目录
    - 清除空文件夹

.PARAMETER Force
    跳过确认提示

.PARAMETER WhatIf
    预览模式，只显示将要删除的内容

.EXAMPLE
    ./Clear-Workspace.ps1
    ./Clear-Workspace.ps1 -Force
    ./Clear-Workspace.ps1 -WhatIf

.NOTES
    版本: 3.1
    要求: PowerShell 7.2+
#>

#Requires -Version 7.2

param(
    [switch]$Force,
    [switch]$WhatIf,
    [string]$ConfigPath = "./config/settings.json"
)

# ============================================================
# 导入模块
# ============================================================
$modulePath = Split-Path $PSScriptRoot -Parent
Import-Module (Join-Path $modulePath "modules/UI.psm1") -Force
Import-Module (Join-Path $modulePath "modules/Config.psm1") -Force

# ============================================================
# 加载配置
# ============================================================
$config = Get-Config -Path $ConfigPath
$cleanTargets = $config.clean.targets

# ============================================================
# 主程序
# ============================================================
Clear-Host
Write-Title -Title "清理工具" -Version "3.1"

if ($WhatIf) {
    Write-Status -Message "预览模式 - 不会实际删除文件" -Type Warning
    Write-Host ""
}

Write-Status -Message "将要清理的目录: $($cleanTargets -join ', ')" -Type Info
Write-Host ""

# --- 确认提示 ---
if (-not $Force -and -not $WhatIf) {
    $confirmation = Read-Host "确认清理? (y/N)"
    if ($confirmation -ne 'y' -and $confirmation -ne 'Y') {
        Write-Status -Message "操作已取消" -Type Warning
        exit 0
    }
    Write-Host ""
}

# --- 执行清理 ---
$cleanedCount = 0
$cleanedDirs = 0
$skippedCount = 0
$failedCount = 0
$totalSize = 0

foreach ($target in $cleanTargets) {
    if (Test-Path $target) {
        try {
            # 计算目录大小
            $dirInfo = Get-ChildItem $target -Recurse -File -ErrorAction SilentlyContinue
            $dirSize = ($dirInfo | Measure-Object -Property Length -Sum).Sum
            $dirSizeMB = [math]::Round($dirSize / 1MB, 2)
            $fileCount = @($dirInfo).Count

            if ($WhatIf) {
                Write-Status -Message "$target - $fileCount 个文件, $dirSizeMB MB" -Type Warning
                $skippedCount++
            } else {
                Remove-Item $target -Recurse -Force -ErrorAction Stop
                Write-Status -Message "$target - 已删除 $fileCount 个文件, $dirSizeMB MB" -Type Success
                $cleanedCount++
                $cleanedDirs++
                $totalSize += $dirSize
            }
        } catch {
            Write-Status -Message "$target - 删除失败: $($_.Exception.Message)" -Type Failure
            $failedCount++
        }
    } else {
        Write-Status -Message "$target - 不存在，跳过" -Type Info
        $skippedCount++
    }
}

# --- 清除空文件夹 ---
if (-not $WhatIf) {
    Write-Host ""
    Write-Status -Message "检查空文件夹..." -Type Info

    $emptyDirs = Get-ChildItem -Directory -Recurse -ErrorAction SilentlyContinue |
                 Where-Object { (Get-ChildItem $_.FullName -File -Recurse -ErrorAction SilentlyContinue).Count -eq 0 }

    foreach ($dir in $emptyDirs) {
        try {
            Remove-Item $dir.FullName -Force -Recurse -ErrorAction Stop
            Write-Status -Message "删除空文件夹: $($dir.Name)" -Type Info
            $cleanedDirs++
        } catch {
            # 忽略删除失败的空文件夹
        }
    }
}

# --- 结束统计 ---
Write-SectionHeader -Title "完成"

$totalSizeMB = [math]::Round($totalSize / 1MB, 2)

$summaryItems = @{
    "已清理" = "$cleanedCount 个目录"
    "释放空间" = "$totalSizeMB MB"
}

if ($skippedCount -gt 0) {
    $summaryItems["跳过"] = "$skippedCount 个目录"
}

if ($failedCount -gt 0) {
    $summaryItems["失败"] = "$failedCount 个目录"
}

Write-SummaryBox -Title "清理完成" -Items $summaryItems

# 音效提示
if (-not $WhatIf -and $config.ui.beepOnComplete) {
    if ($failedCount -gt 0) {
        Invoke-Beep -Type Error
    } elseif ($cleanedCount -gt 0) {
        Invoke-Beep -Type Success
    }
}

Write-Host ""
Write-Status -Message "按任意键退出..." -Type Info
[void][System.Console]::ReadKey($true)

exit $(if ($failedCount -gt 0) { 1 } else { 0 })
