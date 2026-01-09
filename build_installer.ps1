Write-Host "Building Urocissa Installer..."

# Locate NSIS
$nsisPath = "${env:ProgramFiles(x86)}\NSIS\makensis.exe"
if (-not (Test-Path $nsisPath)) {
    Write-Warning "NSIS not found at default location: $nsisPath"
    Write-Host "Attempting to find makensis in PATH..."
    $nsisPath = "makensis.exe"
}

# 1. Build Backend with Static Linking AND Embedded Frontend
Write-Host "Compiling Rust Backend (Static Linking + Embedded Frontend)..."
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --manifest-path "gallery-backend/Cargo.toml" --profile static-release --features embed-frontend

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit 1
}

# 2. Create Installer with NSIS
Write-Host "Creating Installer with NSIS ($nsisPath)..."

& $nsisPath "/DPRODUCT_ICON=c:\Users\User\Documents\GitHub\Urocissa\gallery-backend\assets\logo.ico" gallery-backend/installer.nsi

if ($LASTEXITCODE -ne 0) {
    Write-Error "NSIS build failed!"
    exit 1
}

Write-Host "Build Complete! Installer located at: gallery-backend/urocissa-install-1.0.exe"
