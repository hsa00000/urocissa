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

Write-Host "Embedding icon into executable using rcedit..."
if (!(Test-Path "rcedit.exe")) {
    Write-Host "Downloading rcedit.exe..."
    Invoke-WebRequest -Uri "https://github.com/electron/rcedit/releases/latest/download/rcedit-x64.exe" -OutFile "rcedit.exe"
}
$exeSource = Resolve-Path "gallery-backend/target/$profile/urocissa.exe"
$iconSource = Resolve-Path "gallery-backend\assets\logo.ico"
& .\rcedit.exe $exeSource --set-icon $iconSource

Write-Host "Creating Installer with NSIS ($nsisPath)..."
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
