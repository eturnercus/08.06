# Downloads llama.cpp Windows binaries (+ all required DLLs) for NeuroForge subprocess inference.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\download-llama-win.ps1 [-Variant cuda|vulkan|cpu]

param(
    [ValidateSet("cpu", "cuda", "vulkan")]
    [string]$Variant = "cpu"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "src-tauri/bin/llama"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

$Api = "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest"
Write-Host "[INFO] Fetching latest llama.cpp release..."
$Release = Invoke-RestMethod -Uri $Api -Headers @{ "User-Agent" = "NeuroForge" }

function Pick-Asset($assets, $variant) {
    switch ($variant) {
        "cuda" {
            return $assets | Where-Object { $_.name -match "bin-win-cuda-.*-x64\.zip$" } | Select-Object -First 1
        }
        "vulkan" {
            return $assets | Where-Object { $_.name -match "bin-win-vulkan-x64\.zip$" } | Select-Object -First 1
        }
        default {
            $cpu = $assets | Where-Object { $_.name -match "bin-win-cpu-x64\.zip$" } | Select-Object -First 1
            if ($cpu) { return $cpu }
            return $assets | Where-Object {
                $_.name -match "bin-win-.*x64\.zip$" -and
                $_.name -notmatch "arm|hip|opencl|cuda|vulkan"
            } | Select-Object -First 1
        }
    }
}

$Asset = Pick-Asset $Release.assets $Variant
if (-not $Asset) {
    Write-Error "No Windows llama.cpp zip found for variant '$Variant'."
}

$ZipPath = Join-Path $env:TEMP $Asset.name
Write-Host "[INFO] Downloading $($Asset.name)..."
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath -UseBasicParsing

$Extract = Join-Path $env:TEMP "llama-cpp-extract"
if (Test-Path $Extract) { Remove-Item -Recurse -Force $Extract }
Expand-Archive -Path $ZipPath -DestinationPath $Extract -Force

$Cli = Get-ChildItem -Path $Extract -Recurse -Filter "llama-cli.exe" | Select-Object -First 1
if (-not $Cli) {
    Write-Error "llama-cli.exe not found inside archive."
}

$CliDir = $Cli.Directory.FullName
Write-Host "[INFO] llama-cli directory: $CliDir"

Copy-Item $Cli.FullName (Join-Path $Dest "llama-cli.exe") -Force
Write-Host "[ OK ] $(Join-Path $Dest 'llama-cli.exe')"

# Copy every DLL next to llama-cli (llama.dll, ggml.dll, ggml-base.dll, ggml-cpu.dll, …)
$DllDirs = @($CliDir)
$parent = Split-Path $CliDir -Parent
if ($parent) { $DllDirs += $parent }
$DllDirs += $Extract

$copied = @{}
foreach ($dir in ($DllDirs | Select-Object -Unique)) {
    if (-not (Test-Path $dir)) { continue }
    Get-ChildItem -Path $dir -Filter "*.dll" -File -ErrorAction SilentlyContinue | ForEach-Object {
        if (-not $copied.ContainsKey($_.Name)) {
            Copy-Item $_.FullName (Join-Path $Dest $_.Name) -Force
            $copied[$_.Name] = $true
            Write-Host "[ OK ] DLL: $($_.Name)"
        }
    }
}

if (-not $copied.ContainsKey("llama.dll")) {
    Write-Warning "llama.dll not found in archive — llama-cli may fail at runtime."
    Write-Warning "Try another variant: -Variant cpu | cuda | vulkan"
}

Write-Host "[INFO] $($copied.Count) DLL(s) staged in $Dest"
Write-Host "[INFO] Rebuild NeuroForge (build-windows.bat) so the installer bundles these files."
