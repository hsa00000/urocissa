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

$cargoToml = Get-Content "gallery-backend/Cargo.toml"
$versionLine = $cargoToml | Select-String -Pattern '^version\s*=\s*"(.*)"'
if ($versionLine) {
    $version = $versionLine.Matches.Groups[1].Value
} else {
    $version = "0.0.0"
    Write-Warning "Could not determine version from Cargo.toml. Using default 0.0.0"
}

Write-Host "Detected version: $version"

Write-Host "Compiling Rust Backend (Static Linking + Embedded Frontend)..."
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --manifest-path "gallery-backend/Cargo.toml" --profile $profile --features "embed-frontend auto-open-browser"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit 1
}

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
