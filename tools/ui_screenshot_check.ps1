param(
    [string]$BinaryPath = "target/debug/century_journey.exe",
    [string]$OutputDirectory = "target/ui-screenshots",
    [int]$CaptureTimeoutSeconds = 90
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

function Test-Screenshot([string]$Path, [int]$ExpectedWidth, [int]$ExpectedHeight) {
    $bitmap = [Drawing.Bitmap]::new($Path)
    try {
        if ($bitmap.Width -ne $ExpectedWidth -or $bitmap.Height -ne $ExpectedHeight) {
            throw "Expected $($ExpectedWidth)x$($ExpectedHeight), got $($bitmap.Width)x$($bitmap.Height)."
        }
        $colors = [Collections.Generic.HashSet[int]]::new()
        $brightness = 0L
        $samples = 0
        for ($y = 0; $y -lt $bitmap.Height; $y += 64) {
            for ($x = 0; $x -lt $bitmap.Width; $x += 64) {
                $color = $bitmap.GetPixel($x, $y)
                [void]$colors.Add($color.ToArgb())
                $brightness += $color.R + $color.G + $color.B
                $samples += 1
            }
        }
        $average = $brightness / (3 * $samples)
        if ($colors.Count -lt 8 -or $average -lt 2) {
            throw "Screenshot appears blank (colors=$($colors.Count), average=$average)."
        }
        return [pscustomobject]@{
            Width = $bitmap.Width
            Height = $bitmap.Height
            SampledColors = $colors.Count
            AverageBrightness = [Math]::Round($average, 1)
        }
    } finally {
        $bitmap.Dispose()
    }
}

function Invoke-Capture(
    [string]$Mode,
    [int]$Width,
    [int]$Height,
    [string]$OutputPath,
    [string]$Executable,
    [int]$TimeoutSeconds
) {
    Remove-Item -LiteralPath $OutputPath -Force -ErrorAction SilentlyContinue
    $env:CJ_UI_SCREENSHOT = $OutputPath
    $env:CJ_UI_SCREENSHOT_MODE = $Mode
    $env:CJ_UI_SCREENSHOT_WIDTH = $Width.ToString()
    $env:CJ_UI_SCREENSHOT_HEIGHT = $Height.ToString()
    $process = Start-Process -FilePath $Executable -WorkingDirectory (Get-Location).Path -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        while ([DateTime]::UtcNow -lt $deadline) {
            $process.Refresh()
            if (Test-Path -LiteralPath $OutputPath) {
                $lastLength = -1L
                $stableChecks = 0
                while ($stableChecks -lt 3) {
                    Start-Sleep -Milliseconds 500
                    $length = (Get-Item -LiteralPath $OutputPath).Length
                    if ($length -gt 0 -and $length -eq $lastLength) {
                        $stableChecks += 1
                    } else {
                        $stableChecks = 0
                        $lastLength = $length
                    }
                }
                return Test-Screenshot $OutputPath $Width $Height
            }
            if ($process.HasExited) {
                throw "Game exited before writing $OutputPath."
            }
            Start-Sleep -Milliseconds 200
        }
        throw "Timed out waiting for $OutputPath."
    } finally {
        if (-not $process.HasExited) {
            Stop-Process -Id $process.Id
        }
        [void]$process.WaitForExit(10000)
        Start-Sleep -Seconds 2
    }
}

$resolvedBinary = (Resolve-Path $BinaryPath).Path
$resolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
[IO.Directory]::CreateDirectory($resolvedOutput) | Out-Null
$resolutions = @(
    @{ Width = 1280; Height = 720 },
    @{ Width = 1920; Height = 1080 },
    @{ Width = 2560; Height = 1440 }
)
$originalEnvironment = @{
    Screenshot = $env:CJ_UI_SCREENSHOT
    Mode = $env:CJ_UI_SCREENSHOT_MODE
    Width = $env:CJ_UI_SCREENSHOT_WIDTH
    Height = $env:CJ_UI_SCREENSHOT_HEIGHT
}

$results = @()
try {
    foreach ($mode in @("survival", "creative")) {
        foreach ($resolution in $resolutions) {
            $fileName = "inventory-$mode-$($resolution.Width)x$($resolution.Height).png"
            $outputPath = Join-Path $resolvedOutput $fileName
            $result = Invoke-Capture $mode $resolution.Width $resolution.Height $outputPath $resolvedBinary $CaptureTimeoutSeconds
            $results += [pscustomobject]@{
                Name = $fileName
                Size = "$($result.Width)x$($result.Height)"
                Colors = $result.SampledColors
                Brightness = $result.AverageBrightness
            }
        }
    }
} finally {
    $env:CJ_UI_SCREENSHOT = $originalEnvironment.Screenshot
    $env:CJ_UI_SCREENSHOT_MODE = $originalEnvironment.Mode
    $env:CJ_UI_SCREENSHOT_WIDTH = $originalEnvironment.Width
    $env:CJ_UI_SCREENSHOT_HEIGHT = $originalEnvironment.Height
}

$results | Format-Table -AutoSize
