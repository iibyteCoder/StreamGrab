<#
.SYNOPSIS
    视频处理脚本 - 合并与整理

.DESCRIPTION
    本脚本自动处理视频文件：
    - 有对应音频文件的视频：合并后删除临时文件
    - 没有对应音频文件的视频：直接移动到输出目录
    最终所有视频都会在同一个输出目录中。

.PARAMETER VideoPath
    视频文件来源目录

.PARAMETER AudioPath
    音频文件来源目录（默认与视频目录相同）

.PARAMETER OutputPath
    输出目录

.PARAMETER ConfigPath
    配置文件路径

.PARAMETER KeepTemp
    保留临时文件（不删除源文件）

.EXAMPLE
    ./Merge-AudioVideo.ps1
    ./Merge-AudioVideo.ps1 -KeepTemp

.NOTES
    版本: 3.1
    要求: PowerShell 7.2+, ffmpeg.exe
#>

#Requires -Version 7.2

# ============================================================
# 参数定义
# ============================================================
param(
    [string]$VideoPath,
    [string]$AudioPath,
    [string]$OutputPath,
    [string]$ConfigPath = "./config/settings.json",
    [switch]$KeepTemp
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
if ($VideoPath) { $config.paths.tempDir = $VideoPath }
if ($AudioPath) { $config.merge.audioPath = $AudioPath }
if ($OutputPath) { $config.paths.outputDir = $OutputPath }

# 简化变量访问
$VideoPath = $config.paths.tempDir
$AudioPath = if ($config.merge.audioPath) { $config.merge.audioPath } else { $VideoPath }
$OutputPath = $config.paths.outputDir
$FfmpegPath = $config.tools.ffmpeg
$VideoExt = $config.merge.videoExt
$AudioExt = $config.merge.audioExt

# ============================================================
# 获取模块资源
# ============================================================
$theme = Get-Theme
$icons = Get-Icons

# ============================================================
# 初始化日志
# ============================================================
Initialize-Logger -LogDir $config.paths.logDir -EnableFileLog $true

# ============================================================
# 主程序
# ============================================================
Write-Title -Title "视频处理工具" -Version "3.1"

# --- 1. 环境检查 ---
if (-not (Test-Path $VideoPath)) {
    Write-Status -Message "视频目录 '$VideoPath' 不存在" -Type Warning
    Write-Status -Message "没有需要处理的文件" -Type Info
    exit 0
}

if (-not (Test-Path $OutputPath)) {
    Write-Status -Message "创建输出目录 '$OutputPath'" -Type Info
    New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
}

# --- 2. 查找并处理文件 ---
$videoFilter = "*$VideoExt"
$videoFiles = Get-ChildItem -Path $VideoPath -Filter $videoFilter -File
$totalFiles = $videoFiles.Count
$processedFiles = 0
$mergedCount = 0
$movedCount = 0
$skippedCount = 0
$failedCount = 0
$cleanedFiles = @()

if ($totalFiles -eq 0) {
    Write-Status -Message "在 '$VideoPath' 中未找到 $VideoExt 文件" -Type Info
    exit 0
}

Write-Status -Message "在 '$VideoPath' 中发现 $totalFiles 个 $VideoExt 文件" -Type Info
Write-Host ""

foreach ($video in $videoFiles) {
    $processedFiles++
    $baseName = $video.BaseName
    $audioFile = Get-ChildItem -Path $AudioPath -Filter "$baseName$AudioExt" -File | Select-Object -First 1

    Write-Host "$($icons.Task) $($theme.Title)处理 [$processedFiles/$totalFiles]:$($theme.Reset) $($video.Name)"

    $outputFile = Join-Path $OutputPath $video.Name

    # 检查输出文件是否已存在
    if (Test-Path $outputFile) {
        if ($config.merge.overwrite) {
            Write-Status -Message "输出文件已存在，将覆盖" -Type Warning
            Remove-Item $outputFile -Force
        } else {
            Write-Status -Message "输出文件已存在，跳过" -Type Warning
            $skippedCount++
            Write-Host ""
            continue
        }
    }

    if ($audioFile) {
        # --- 情况1: 有对应的音频文件，执行合并 ---
        Write-Status -Message "找到音频文件，执行合并..." -Type Info

        # 构建 FFmpeg 参数
        $ffmpegArgs = @(
            "-i", $video.FullName,
            "-i", $audioFile.FullName,
            "-c", $config.merge.codec,
            "-y",
            "-nostdin",
            "-loglevel", "error",
            $outputFile
        )

        try {
            # 检查 FFmpeg 是否存在
            if (-not (Test-Path $FfmpegPath)) {
                Write-Status -Message "FFmpeg 未找到，改为直接移动视频文件" -Type Warning
                Move-Item -Path $video.FullName -Destination $outputFile -Force
                $movedCount++
                Write-Status -Message "已移动到输出目录" -Type Success
            } else {
                # 执行合并
                & $FfmpegPath $ffmpegArgs 2>&1 | Out-Null

                if ($LASTEXITCODE -eq 0 -and (Test-Path $outputFile)) {
                    # 验证输出文件
                    $fileInfo = Get-Item $outputFile
                    if ($fileInfo.Length -gt 1KB) {
                        $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
                        Write-Status -Message "合并成功! 大小: $sizeMB MB" -Type Success
                        $mergedCount++
                        Write-Log -Message "合并成功: $($video.Name) ($sizeMB MB)" -Level Info -LogName "merge"

                        # 标记临时文件待清理
                        if (-not $KeepTemp) {
                            $cleanedFiles += $video.FullName
                            $cleanedFiles += $audioFile.FullName
                        }
                    } else {
                        Write-Status -Message "输出文件过小，可能损坏" -Type Failure
                        Remove-Item $outputFile -Force -ErrorAction SilentlyContinue
                        $failedCount++
                    }
                } else {
                    Write-Status -Message "合并失败 (退出码: $LASTEXITCODE)" -Type Failure
                    if (Test-Path $outputFile) {
                        Remove-Item $outputFile -Force -ErrorAction SilentlyContinue
                    }
                    $failedCount++
                }
            }
        } catch {
            Write-Status -Message "处理错误: $($_.Exception.Message)" -Type Failure
            $failedCount++
        }
    } else {
        # --- 情况2: 没有对应的音频文件，直接移动 ---
        Write-Status -Message "无需合并，直接移动到输出目录" -Type Info

        try {
            Move-Item -Path $video.FullName -Destination $outputFile -Force
            $sizeMB = [math]::Round((Get-Item $outputFile).Length / 1MB, 2)
            Write-Status -Message "移动成功! 大小: $sizeMB MB" -Type Success
            $movedCount++
            Write-Log -Message "移动视频: $($video.Name) ($sizeMB MB)" -Level Info -LogName "merge"
        } catch {
            Write-Status -Message "移动失败: $($_.Exception.Message)" -Type Failure
            $failedCount++
        }
    }

    Write-Host ""
}

# --- 3. 清理临时文件 ---
if (-not $KeepTemp -and $cleanedFiles.Count -gt 0) {
    Write-Status -Message "清理临时文件..." -Type Info
    $cleanedCount = 0

    foreach ($file in $cleanedFiles) {
        if (Test-Path $file) {
            try {
                Remove-Item $file -Force -ErrorAction Stop
                $cleanedCount++
            } catch {
                Write-Status -Message "无法删除: $file" -Type Warning
            }
        }
    }

    if ($cleanedCount -gt 0) {
        Write-Status -Message "已清理 $cleanedCount 个临时文件" -Type Success
    }
    Write-Host ""
}

# --- 4. 结束统计 ---
Write-SectionHeader -Title "完成"

$summaryItems = @{}

if ($mergedCount -gt 0) {
    $summaryItems["已合并"] = "$mergedCount 个文件"
}

if ($movedCount -gt 0) {
    $summaryItems["已移动"] = "$movedCount 个文件"
}

if ($skippedCount -gt 0) {
    $summaryItems["已跳过"] = "$skippedCount 个文件"
}

if ($failedCount -gt 0) {
    $summaryItems["失败"] = "$failedCount 个文件"
}

$summaryItems["输出目录"] = $OutputPath

Write-SummaryBox -Title "处理完成" -Items $summaryItems

# 音效提示
if ($config.ui.beepOnComplete) {
    if ($failedCount -gt 0) {
        Invoke-Beep -Type Error
    } elseif ($mergedCount -gt 0 -or $movedCount -gt 0) {
        Invoke-Beep -Type Success
    }
}

Write-Host ""

exit $(if ($failedCount -gt 0) { 1 } else { 0 })
