param (
    [switch]$dev
)

Write-Host "Building Urocissa Installer..."

$nsisPath = "${env:ProgramFiles(x86)}\NSIS\makensis.exe"
if (-not (Test-Path $nsisPath)) {
    Write-Warning "NSIS not found at default location: $nsisPath"
    Write-Host "Attempting to find makensis in PATH..."
    $nsisPath = "makensis.exe"
}

$profile = "static-release"
if ($dev) {
    $profile = "static-dev"
    Write-Host "Dev mode enabled. Using profile: $profile"
} else {
    Write-Host "Release mode. Using profile: $profile"
}

# Determine version based on GitHub Actions environment
if ($env:GITHUB_REF_TYPE -eq "tag") {
    $version = $env:GITHUB_REF_NAME
    Write-Host "Tag detected. Using version: $version"
} else {
    $version = "0.0.0"
    Write-Host "No tag detected. Using default version: $version"
}

Write-Host "Detected version: $version"

Write-Host "Compiling Rust Backend (Static Linking + Embedded Frontend)..."
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --manifest-path "gallery-backend/Cargo.toml" --profile $profile --features "embed-frontend auto-open-browser"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit 1
}

# ---- Embed exe icon (Windows release only) ----
$repoRoot = Resolve-Path $PSScriptRoot | Select-Object -ExpandProperty Path

$exePath  = Join-Path $repoRoot "gallery-backend\target\$profile\urocissa.exe"
$iconPath = Join-Path $repoRoot "gallery-backend\assets\logo.ico"

if (-not (Test-Path $exePath))  { throw "exe not found: $exePath" }
if (-not (Test-Path $iconPath)) { throw "icon not found: $iconPath" }

# Ensure rcedit exists (via Chocolatey)
if (-not (Get-Command rcedit.exe -ErrorAction SilentlyContinue)) {
  choco install rcedit -y --no-progress
  if ($LASTEXITCODE -ne 0) { throw "choco install rcedit failed." }
}

# Set icon
rcedit "$exePath" --set-icon "$iconPath"
if ($LASTEXITCODE -ne 0) { throw "rcedit failed." }

Write-Host "Creating Installer with NSIS ($nsisPath)..."

$exeSource = Resolve-Path "gallery-backend/target/$profile/urocissa.exe"
$iconSource = Resolve-Path "gallery-backend\assets\logo.ico"
$installerName = "urocissa-windows-installer-${version}.exe"

Write-Host "Using Executable: $exeSource"
Write-Host "Using Icon: $iconSource"
Write-Host "Installer Name: $installerName"

& $nsisPath "/DPRODUCT_ICON=$iconSource" "/DEXE_SOURCE=$exeSource" "/DINSTALLER_NAME=$installerName" gallery-backend/installer.nsi

if ($LASTEXITCODE -ne 0) {
    Write-Error "NSIS build failed!"
    exit 1
}

Write-Host "Build Complete! Installer located at: gallery-backend/$installerName"
