param (
    [string]$Profile = "release",
    [string]$Features = "embed-frontend auto-open-browser",
    [switch]$SkipFrontend
)

Write-Host "Building Linux Urocissa binary with Docker..."

# Ensure docker is available
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "Docker is not installed or not in PATH. Please install Docker Desktop and try again."
    exit 1
}

# Resolve repo root (script directory)
$repoRoot = Resolve-Path $PSScriptRoot | Select-Object -ExpandProperty Path

# Determine Cargo target directory name
$targetDir = if ($Profile -eq "debug") { "debug" } elseif ($Profile -eq "release") { "release" } else { $Profile }

# Extract version from Cargo.toml
$cargoTomlPath = Join-Path $repoRoot "gallery-backend\Cargo.toml"
$cargoToml = Get-Content $cargoTomlPath
$versionLine = $cargoToml | Select-String -Pattern '^version\s*=\s*"(.*)"'
if ($versionLine) {
    $version = $versionLine.Matches.Groups[1].Value
} else {
    $version = "0.0.0"
    Write-Warning "Could not determine version from Cargo.toml. Using default 0.0.0"
}

Write-Host "Detected version: $version"
Write-Host "Profile: $Profile"
Write-Host "Features: $Features"

$featuresArg = $Features.Trim()
if ($featuresArg) {
    $featuresArg = ($featuresArg -replace "\s+", ",")
}

# 1) Build frontend (in Docker) unless skipped
if (-not $SkipFrontend) {
    Write-Host "Building frontend in Docker (node:lts)..."
    $frontendArgs = @(
        "run",
        "--rm",
        "-v", "${repoRoot}:/app",
        "-w", "/app/gallery-frontend",
        "node:lts",
        "bash", "-lc",
        "npm ci; npm run build"
    )
    Write-Host ("docker " + ($frontendArgs -join " "))
    & docker @frontendArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Frontend build failed!"
        exit 1
    }
} else {
    Write-Host "Skipping frontend build as requested."
}

# 2) Build backend (in Docker)
Write-Host "Building backend in Docker (rust:bookworm)..."

$cargoProfileArg = if ($Profile -eq "release") { "--release" } elseif ($Profile -eq "debug") { "" } else { "--profile $Profile" }

$featuresClause = if ($featuresArg) { "--features $featuresArg" } else { "" }
$backendShell = "export PATH=/usr/local/cargo/bin:/usr/local/rustup/bin:`$PATH; apt-get update; apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev; rm -rf /var/lib/apt/lists/*; cargo build $cargoProfileArg $featuresClause"
$backendArgs = @(
    "run",
    "--rm",
    "-v", "${repoRoot}:/app",
    "-w", "/app/gallery-backend",
    "rust:bookworm",
    "bash", "-lc",
    $backendShell
)
Write-Host ("docker " + ($backendArgs -join " "))
& docker @backendArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Backend build failed!"
    exit 1
}

# 3) Copy output binary to dist folder
$binaryPath = Join-Path $repoRoot "gallery-backend\target\$targetDir\urocissa"
if (-not (Test-Path $binaryPath)) {
    Write-Error "Linux binary not found at $binaryPath"
    exit 1
}

$outDir = Join-Path $repoRoot "dist"
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$outFile = Join-Path $outDir "urocissa-linux-$version-$targetDir"
Copy-Item -Path $binaryPath -Destination $outFile -Force

Write-Host "Build complete!"
Write-Host "Linux binary: $outFile"