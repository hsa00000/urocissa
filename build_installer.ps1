param (
    [switch]$dev
)

Write-Host "Building Urocissa Installer..."

# Locate NSIS
$nsisPath = "${env:ProgramFiles(x86)}\NSIS\makensis.exe"
if (-not (Test-Path $nsisPath)) {
    Write-Warning "NSIS not found at default location: $nsisPath"
    Write-Host "Attempting to find makensis in PATH..."
    $nsisPath = "makensis.exe"
}

# Determine profile based on flag
$profile = "static-release"
if ($dev) {
    $profile = "static-dev"
    Write-Host "Dev mode enabled. Using profile: $profile"
} else {
    Write-Host "Release mode. Using profile: $profile"
}

# 1. Build Backend with Static Linking AND Embedded Frontend
Write-Host "Compiling Rust Backend (Static Linking + Embedded Frontend)..."
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --manifest-path "gallery-backend/Cargo.toml" --profile $profile --features "embed-frontend auto-open-browser"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit 1
}

# 2. Create Installer with NSIS
Write-Host "Creating Installer with NSIS ($nsisPath)..."

# Define the source executable path based on profile
# Note: Paths should be relative to the NSIS script if not absolute.
# Since we are running makensis on "gallery-backend/installer.nsi", 
# the script's directory (gallery-backend/) is the context for relative paths in the script.
#
# However, we are passing EXE_SOURCE from outside.
# Let's use an absolute path to avoid ambiguity.
$exeSource = Resolve-Path "gallery-backend/target/$profile/urocissa.exe"
Write-Host "Using Executable: $exeSource"

& $nsisPath "/DPRODUCT_ICON=c:\Users\User\Documents\GitHub\Urocissa\gallery-backend\assets\logo.ico" "/DEXE_SOURCE=$exeSource" gallery-backend/installer.nsi

if ($LASTEXITCODE -ne 0) {
    Write-Error "NSIS build failed!"
    exit 1
}

Write-Host "Build Complete! Installer located at: gallery-backend/urocissa-install-1.0.exe"
