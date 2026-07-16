$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$runRoot = Join-Path $repoRoot "target\perf-run"
$output = Join-Path $repoRoot "target\perf\survival_spawn_v1.json"
$manifest = Join-Path $repoRoot "Cargo.toml"
$assetRoot = Join-Path $repoRoot "assets"

if (Test-Path -LiteralPath $runRoot) {
    $resolvedRunRoot = [System.IO.Path]::GetFullPath($runRoot)
    $resolvedTarget = [System.IO.Path]::GetFullPath((Join-Path $repoRoot "target"))
    if (-not $resolvedRunRoot.StartsWith($resolvedTarget, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to clean performance run directory outside target: $resolvedRunRoot"
    }
    Remove-Item -LiteralPath $resolvedRunRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
New-Item -ItemType Junction -Path (Join-Path $runRoot "assets") -Target $assetRoot | Out-Null

$previousAssetRoot = $env:CJ_ASSET_ROOT
$previousScenario = $env:CJ_PERF_SCENARIO
$previousOutput = $env:CJ_PERF_OUTPUT
try {
    $env:CJ_ASSET_ROOT = $assetRoot
    $env:CJ_PERF_SCENARIO = "survival_spawn_v1"
    $env:CJ_PERF_OUTPUT = $output
    Push-Location $runRoot
    try {
        cargo run --release --manifest-path $manifest
        if ($LASTEXITCODE -ne 0) {
            throw "Performance scenario exited with code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}
finally {
    $env:CJ_ASSET_ROOT = $previousAssetRoot
    $env:CJ_PERF_SCENARIO = $previousScenario
    $env:CJ_PERF_OUTPUT = $previousOutput
}

Get-Content -Raw -LiteralPath $output
