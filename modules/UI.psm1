<#
.SYNOPSIS
    UI 模块 - 统一的控制台 UI 组件
.DESCRIPTION
    提供颜色主题、图标、进度条、日志输出等 UI 功能
.NOTES
    版本: 1.0.0
    要求: PowerShell 7.2+
#>

#Requires -Version 7.2

# ============================================================
# 颜色主题
# ============================================================
$script:Theme = @{
    Title   = $PSStyle.Foreground.FromRgb(0, 220, 255)    # 亮天蓝色
    Accent  = $PSStyle.Foreground.FromRgb(255, 0, 255)    # 洋红色
    Success = $PSStyle.Foreground.FromRgb(0, 255, 128)    # 亮绿色
    Failure = $PSStyle.Foreground.FromRgb(255, 80, 80)    # 亮红色
    Warning = $PSStyle.Foreground.FromRgb(255, 200, 0)    # 橙黄色
    Info    = $PSStyle.Foreground.FromRgb(150, 160, 170)  # 浅灰色
    Dim     = $PSStyle.Foreground.FromRgb(80, 90, 100)    # 暗灰色
    Reset   = $PSStyle.Reset
}

# ============================================================
# 图标
# ============================================================
$script:Icons = @{
    Download = "📥"
    Success  = "✓"
    Failure  = "✗"
    Info     = "·"
    Warning  = "!"
    Rocket   = "🚀"
    Task     = "🎬"
    Merge    = "🎶"
    Clean    = "🧹"
    Folder   = "📁"
    Done     = "✨"
    Spinner  = @('⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏')
}

# ============================================================
# ANSI 转义序列
# ============================================================
$script:Ansi = @{
    SaveCursor    = "$([char]0x1b)[s"
    RestoreCursor = "$([char]0x1b)[u"
    MoveUp        = "$([char]0x1b)[{0}A"
    MoveDown      = "$([char]0x1b)[{0}B"
    MoveTo        = "$([char]0x1b)[{0};{1}H"
    ClearLine     = "$([char]0x1b)[2K"
    ClearBelow    = "$([char]0x1b)[0J"
    ClearScreen   = "$([char]0x1b)[2J"
    HideCursor    = "$([char]0x1b)[?25l"
    ShowCursor    = "$([char]0x1b)[?25h"
    AltScreen     = "$([char]0x1b)[?1049h"
    NormalScreen  = "$([char]0x1b)[?1049l"
}

# ============================================================
# 导出函数
# ============================================================

function Get-Theme {
    <#
    .SYNOPSIS
        获取颜色主题
    #>
    return $script:Theme
}

function Get-Icons {
    <#
    .SYNOPSIS
        获取图标集合
    #>
    return $script:Icons
}

function Get-Ansi {
    <#
    .SYNOPSIS
        获取 ANSI 转义序列
    #>
    return $script:Ansi
}

function Write-Status {
    <#
    .SYNOPSIS
        输出带时间戳和图标的状态消息
    .PARAMETER Message
        消息内容
    .PARAMETER Type
        消息类型: Success, Failure, Warning, Info, Title, Accent
    .PARAMETER NoTimestamp
        不显示时间戳
    .PARAMETER Indent
        缩进级别
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Message,

        [ValidateSet('Success', 'Failure', 'Warning', 'Info', 'Title', 'Accent')]
        [string]$Type = 'Info',

        [switch]$NoTimestamp,

        [int]$Indent = 0
    )

    $theme = Get-Theme

    # 根据类型选择颜色和图标
    $color = $theme.$Type
    $bgColor = switch ($Type) {
        'Success' { $PSStyle.Background.FromRgb(0, 80, 40) }
        'Failure' { $PSStyle.Background.FromRgb(100, 20, 20) }
        'Warning' { $PSStyle.Background.FromRgb(100, 80, 0) }
        'Title'   { $PSStyle.Background.FromRgb(0, 60, 100) }
        'Accent'  { $PSStyle.Background.FromRgb(80, 0, 80) }
        default   { $PSStyle.Background.FromRgb(40, 45, 50) }
    }

    $icon = switch ($Type) {
        'Success' { ' ✔ ' }
        'Failure' { ' ✖ ' }
        'Warning' { ' ⚠ ' }
        'Title'   { ' ▶ ' }
        'Accent'  { ' ★ ' }
        default   { ' ○ ' }
    }

    $indentStr = '  ' * $Indent
    $timestamp = if ($NoTimestamp) { '' } else { "$($theme.Dim)$(Get-Date -Format 'HH:mm:ss')$($theme.Reset) " }

    # 美化的输出格式
    Write-Host "${indentStr}${timestamp}${bgColor}${color}${icon}${theme.Reset} ${color}${Message}${theme.Reset}"
}

function Write-Title {
    <#
    .SYNOPSIS
        输出标题横幅
    .PARAMETER Title
        标题文本
    .PARAMETER Version
        版本号（可选）
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Title,

        [string]$Version
    )

    $theme = Get-Theme
    $titleText = if ($Version) { "$Title v$Version" } else { $Title }
    $padding = [math]::Max(0, 60 - $titleText.Length)

    Write-Host ""
    Write-Host "$($theme.Title)  ╭──────────────────────────────────────────────────────────────────────╮$($theme.Reset)"
    Write-Host "$($theme.Title)  │$($theme.Reset)  $($theme.Accent)$titleText$($theme.Reset)$(' ' * $padding)$($theme.Title)  │$($theme.Reset)"
    Write-Host "$($theme.Title)  ╰──────────────────────────────────────────────────────────────────────╯$($theme.Reset)"
    Write-Host ""
}

function Write-SectionHeader {
    <#
    .SYNOPSIS
        输出分隔线
    .PARAMETER Title
        分隔线标题（可选）
    #>
    param(
        [string]$Title
    )

    $theme = Get-Theme

    if ($Title) {
        $padding = [math]::Max(0, 56 - $Title.Length)
        Write-Host ""
        Write-Host "$($theme.Title)  ──$($theme.Accent) $Title $($theme.Title)$('─' * $padding)─$($theme.Reset)"
    } else {
        Write-Host "$($theme.Dim)  ─────────────────────────────────────────────────────────────────────$($theme.Reset)"
    }
}

function Write-SummaryBox {
    <#
    .SYNOPSIS
        输出统计信息框
    .PARAMETER Title
        框标题
    .PARAMETER Items
        键值对 hashtable
    #>
    param(
        [Parameter(Mandatory)]
        [string]$Title,

        [Parameter(Mandatory)]
        [hashtable]$Items
    )

    $theme = Get-Theme
    $titlePadding = [math]::Max(0, 56 - $Title.Length)

    Write-Host ""
    Write-Host "$($theme.Title)  ╭──────────────────────────────────────────────────────────────────────╮$($theme.Reset)"
    Write-Host "$($theme.Title)  │$($theme.Reset)  $($theme.Accent)$Title$($theme.Reset)$(' ' * $titlePadding)$($theme.Title)  │$($theme.Reset)"
    Write-Host "$($theme.Title)  ├──────────────────────────────────────────────────────────────────────┤$($theme.Reset)"

    foreach ($key in $Items.Keys) {
        $value = $Items[$key]
        $line = "  $($theme.Info)$key$($theme.Reset): $value"
        $linePadding = [math]::Max(0, 68 - $line.Length)
        Write-Host "$($theme.Title)  │$($theme.Reset)$line$(' ' * $linePadding)$($theme.Title)  │$($theme.Reset)"
    }

    Write-Host "$($theme.Title)  ╰──────────────────────────────────────────────────────────────────────╯$($theme.Reset)"
    Write-Host ""
}

function Show-ProgressBar {
    <#
    .SYNOPSIS
        生成进度条字符串数组
    .PARAMETER Current
        当前进度
    .PARAMETER Total
        总数
    .PARAMETER Success
        成功数
    .PARAMETER Failure
        失败数
    .PARAMETER Width
        进度条宽度
    .PARAMETER Elapsed
        已用时间
    #>
    param(
        [int]$Current,
        [int]$Total,
        [int]$Success,
        [int]$Failure,
        [int]$Width = 40,
        [TimeSpan]$Elapsed
    )

    $theme = Get-Theme

    $percentage = if ($Total -gt 0) { [math]::Round($Current / $Total * 100) } else { 0 }
    $filledWidth = [math]::Round($percentage / 100 * $Width)
    $emptyWidth = $Width - $filledWidth

    # 构建渐变进度条
    $progressBar = ""
    for ($i = 0; $i -lt $filledWidth; $i++) {
        $ratio = $i / $Width
        $r = [int](100 + (155 * $ratio))
        $g = [int](200 - (100 * $ratio))
        $b = [int](100 + (50 * $ratio))
        $color = $PSStyle.Foreground.FromRgb($r, $g, $b)
        $progressBar += "${color}●"
    }
    $progressBar += "$($theme.Dim)" + ('○' * $emptyWidth) + "$($theme.Reset)"

    # 格式化耗时
    $elapsedStr = if ($Elapsed) { $Elapsed.ToString('hh\:mm\:ss') } else { "00:00:00" }

    # 状态图标
    $statusIcon = if ($Failure -gt 0) { "$($theme.Failure)⚠$($theme.Reset)" }
                  elseif ($percentage -eq 100) { "$($theme.Success)✓$($theme.Reset)" }
                  else { "$($theme.Accent)▶$($theme.Reset)" }

    # 构建进度条区域
    $lines = @(
        "",
        "  $($theme.Title)╭──────────────────────────────────────────────────────────────────────╮$($theme.Reset)",
        "  $($theme.Title)│$($theme.Reset)  $($theme.Info)进度$($theme.Reset)  $progressBar  $($theme.Warning)$($percentage.ToString().PadLeft(3))%$($theme.Reset)   $statusIcon  $($theme.Title)│$($theme.Reset)",
        "  $($theme.Title)│$($theme.Reset)  $($theme.Info)任务$($theme.Reset)  $($theme.Success)$($Current.ToString().PadLeft($Total.ToString().Length))$($theme.Reset)$($theme.Dim)/$($theme.Reset)$($Total)   $($theme.Success)✓$Success$($theme.Reset)   $($theme.Failure)✗$Failure$($theme.Reset)   ⏱ $elapsedStr   $($theme.Title)│$($theme.Reset)",
        "  $($theme.Title)╰──────────────────────────────────────────────────────────────────────╯$($theme.Reset)"
    )

    return $lines
}

function Invoke-Beep {
    <#
    .SYNOPSIS
        播放提示音
    .PARAMETER Type
        音效类型: Success, Error, Notify
    #>
    param(
        [ValidateSet('Success', 'Error', 'Notify')]
        [string]$Type = 'Notify'
    )

    try {
        switch ($Type) {
            'Success' {
                [console]::beep(1000, 200)
                [console]::beep(1200, 300)
            }
            'Error' {
                [console]::beep(500, 500)
            }
            'Notify' {
                [console]::beep(800, 100)
            }
        }
    } catch {
        # 忽略蜂鸣错误（某些环境不支持）
    }
}

# 导出模块成员
Export-ModuleMember -Function @(
    'Get-Theme',
    'Get-Icons',
    'Get-Ansi',
    'Write-Status',
    'Write-Title',
    'Write-SectionHeader',
    'Write-SummaryBox',
    'Show-ProgressBar',
    'Invoke-Beep'
)
