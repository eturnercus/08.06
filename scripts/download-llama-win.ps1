# Downloads llama.cpp Windows binaries for Silenium subprocess inference.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\download-llama-win.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "src-tauri\bin/llama"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

$Api = "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest"
Write-Host "[INFO] Fetching latest llama.cpp release..."
$Release = Invoke-RestMethod -Uri $Api -Headers @{ "User-Agent" = "Silenium" }

$Asset = $Release.assets | Where-Object {
    $_.name -match "bin-win.*(cuda|vulkan|avx2).*\.zip$" -and $_.name -notmatch "arm"
} | Sort-Object { if ($_.name -match "cuda") { 0 } elseif ($_.name -match "vulkan") { 1 } else { 2 } } | Select-Object -First 1

if (-not $Asset) {
    Write-Error "No Windows llama.cpp zip found in latest release."
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

Copy-Item $Cli.FullName (Join-Path $Dest "llama-cli.exe") -Force
Write-Host "[ OK ] $($Dest)\llama-cli.exe ($((Get-Item (Join-Path $Dest 'llama-cli.exe')).Length) bytes)"
Write-Host "[INFO] Rebuild Silenium or run from project with embedded llama disabled."
