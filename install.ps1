# ⚡ FetchDesk One-Click Installer for Windows
$ErrorActionPreference = "SilentlyContinue"

Write-Host "`n  ███████╗███████╗████████╗██████╗██╗  ██╗██████╗ ███████╗███████╗██╗  ██╗" -ForegroundColor Cyan
Write-Host "  ██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║██╔══██╗██╔════╝██╔════╝██║ ██╔╝" -ForegroundColor Cyan
Write-Host "  █████╗  █████╗     ██║   ██║     ███████║██║  ██║█████╗  ███████╗█████╔╝ " -ForegroundColor Cyan
Write-Host "  ██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║██║  ██║██╔══╝  ╚════██║██╔═██╗ " -ForegroundColor Cyan
Write-Host "  ██║     ███████╗   ██║   ╚██████╗██║  ██║██████╔╝███████╗███████║██║  ██╗" -ForegroundColor Cyan
Write-Host "  ╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝`n" -ForegroundColor Cyan

Write-Host "  → Installing FetchDesk Download Manager..." -ForegroundColor Yellow

$InstallDir = "$env:LOCALAPPDATA\FetchDesk"
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$ExePath = Join-Path $InstallDir "fetchdesk.exe"
$DownloadUrl = "https://github.com/jojin1709/fetchdesk/releases/latest/download/fetchdesk.exe"

# Add InstallDir to User PATH if not present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "  ✓ Added FetchDesk to User PATH ($InstallDir)" -ForegroundColor Green
}

Write-Host "`n  🎉 FetchDesk ready! Type 'fetchdesk' to launch.`n" -ForegroundColor Green
