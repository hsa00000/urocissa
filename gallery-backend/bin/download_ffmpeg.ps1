param(
    [string]$Url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
)

$ErrorActionPreference = "Stop"
$scriptPath = $PSScriptRoot

Write-Host "Downloading FFmpeg from $Url..."
$zipPath = Join-Path $scriptPath "ffmpeg.zip"
# Use Tls12 for secure download if needed (defaults usually work in newer PS, but good practice for legacy)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Invoke-WebRequest -Uri $Url -OutFile $zipPath

Write-Host "Extracting FFmpeg..."
$extractPath = Join-Path $scriptPath "ffmpeg_temp"
Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force

Write-Host "Locating binaries..."
$ffmpeg = Get-ChildItem -Path $extractPath -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
$ffprobe = Get-ChildItem -Path $extractPath -Recurse -Filter "ffprobe.exe" | Select-Object -First 1

if ($ffmpeg -and $ffprobe) {
    Write-Host "Moving binaries to $scriptPath..."
    Move-Item -Path $ffmpeg.FullName -Destination $scriptPath -Force
    Move-Item -Path $ffprobe.FullName -Destination $scriptPath -Force
    Write-Host "FFmpeg and FFprobe installed successfully."
} else {
    Write-Error "Could not find ffmpeg.exe or ffprobe.exe in the downloaded archive."
}

Write-Host "Cleaning up..."
Remove-Item -Path $zipPath -Force
Remove-Item -Path $extractPath -Recurse -Force
