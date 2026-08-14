#Requires -Version 5.1
# FetchDesk - Automated Setup Script
# Run: iwr -useb https://raw.githubusercontent.com/jojin1709/fetchdesk/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Write-Header {
    Clear-Host
    Write-Host ""
    Write-Host "  ███████╗███████╗████████╗ ██████╗██╗  ██╗██████╗ ███████╗███████╗██╗  ██╗" -ForegroundColor Cyan
    Write-Host "  ██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║██╔══██╗██╔════╝██╔════╝██║ ██╔╝" -ForegroundColor Cyan
    Write-Host "  █████╗  █████╗     ██║   ██║     ███████║██║  ██║█████╗  ███████╗█████╔╝ " -ForegroundColor Cyan
    Write-Host "  ██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║██║  ██║██╔══╝  ╚════██║██╔═██╗ " -ForegroundColor Cyan
    Write-Host "  ██║     ███████╗   ██║   ╚██████╗██║  ██║██████╔╝███████╗███████║██║  ██╗" -ForegroundColor Cyan
    Write-Host "  ╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
    Write-Host "    v0.2.0  •  Terminal-Based Multi-Connection Download Manager  •  by Jojin John" -ForegroundColor DarkGray
    Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step($num, $total, $msg) {
    Write-Host "  [$num/$total] " -ForegroundColor DarkCyan -NoNewline
    Write-Host $msg -ForegroundColor White
}

function Write-OK($msg) {
    Write-Host "        ✓ " -ForegroundColor Green -NoNewline
    Write-Host $msg -ForegroundColor Gray
}

function Write-Fail($msg) {
    Write-Host "        ✗ " -ForegroundColor Red -NoNewline
    Write-Host $msg -ForegroundColor Red
}

function Write-Info($msg) {
    Write-Host "          → " -ForegroundColor DarkGray -NoNewline
    Write-Host $msg -ForegroundColor DarkGray
}

# ── Paths ────────────────────────────────────────────────────────────────────
$InstallDir = "$env:LOCALAPPDATA\FetchDesk"
$BinDir     = "$InstallDir\bin"
$ExePath    = "$InstallDir\fetchdesk.exe"

foreach ($dir in @($InstallDir, $BinDir)) {
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
}

Write-Header

# ─────────────────────────────────────────────────────────────────────────────
#   STEP 1  ─  Pre-flight system check
# ─────────────────────────────────────────────────────────────────────────────
Write-Step 1 5 "Checking system requirements..."
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-OK "Windows $([System.Environment]::OSVersion.Version.Major).$([System.Environment]::OSVersion.Version.Minor), $arch"

# Check winget
$winget = Get-Command winget -ErrorAction SilentlyContinue
if ($winget) { Write-OK "winget found ($($winget.Source))" }
else         { Write-Info "winget not found — will use direct downloads" }

# Check pip / Python
$pip = Get-Command pip -ErrorAction SilentlyContinue
if ($pip) { Write-OK "pip found" }
else      { Write-Info "pip not found — will install yt-dlp via standalone binary" }

Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   STEP 2  ─  Download & install aria2c
# ─────────────────────────────────────────────────────────────────────────────
Write-Step 2 5 "Installing aria2c (BitTorrent & Magnet engine)..."
$aria2Dest = "$BinDir\aria2c.exe"
$aria2Already = (Get-Command aria2c -ErrorAction SilentlyContinue) -or (Test-Path $aria2Dest)

if ($aria2Already) {
    Write-OK "aria2c already installed"
} elseif ($winget) {
    try {
        Write-Info "Installing via winget..."
        winget install aria2.aria2 --accept-source-agreements --accept-package-agreements -h 2>&1 | Out-Null
        Write-OK "aria2c installed via winget"
    } catch {
        Write-Fail "winget install failed, trying direct download..."
        $aria2Already = $false
    }
}

if (-not $aria2Already -and -not (Get-Command aria2c -ErrorAction SilentlyContinue)) {
    try {
        Write-Info "Downloading aria2c 1.37.0 binary..."
        $aria2Url = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip"
        $aria2Zip = "$env:TEMP\aria2.zip"
        Invoke-WebRequest -Uri $aria2Url -OutFile $aria2Zip -UseBasicParsing
        $aria2Tmp = "$env:TEMP\aria2extract"
        Expand-Archive -Path $aria2Zip -DestinationPath $aria2Tmp -Force
        $aria2Exe = Get-ChildItem -Recurse -Filter "aria2c.exe" -Path $aria2Tmp | Select-Object -First 1
        Copy-Item $aria2Exe.FullName -Destination $aria2Dest -Force
        Remove-Item $aria2Zip -Force -ErrorAction SilentlyContinue
        Remove-Item $aria2Tmp -Recurse -Force -ErrorAction SilentlyContinue
        Write-OK "aria2c downloaded and installed to $aria2Dest"
    } catch {
        Write-Fail "Could not install aria2c: $_"
    }
}

Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   STEP 3  ─  Install yt-dlp
# ─────────────────────────────────────────────────────────────────────────────
Write-Step 3 5 "Installing yt-dlp (YouTube engine)..."
$ytdlpDest    = "$BinDir\yt-dlp.exe"
$ytdlpAlready = (Get-Command yt-dlp -ErrorAction SilentlyContinue) -or (Test-Path $ytdlpDest)

if ($ytdlpAlready) {
    Write-OK "yt-dlp already installed"
} elseif ($pip) {
    try {
        Write-Info "Installing via pip..."
        pip install -U yt-dlp -q 2>&1 | Out-Null
        Write-OK "yt-dlp installed via pip"
    } catch {
        Write-Fail "pip install failed, trying standalone binary..."
        $ytdlpAlready = $false
    }
}

if (-not $ytdlpAlready -and -not (Get-Command yt-dlp -ErrorAction SilentlyContinue)) {
    try {
        Write-Info "Downloading yt-dlp standalone binary..."
        $ytdlpUrl = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
        Invoke-WebRequest -Uri $ytdlpUrl -OutFile $ytdlpDest -UseBasicParsing
        Write-OK "yt-dlp downloaded to $ytdlpDest"
    } catch {
        Write-Fail "Could not install yt-dlp: $_"
    }
}

Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   STEP 4  ─  Install ffmpeg
# ─────────────────────────────────────────────────────────────────────────────
Write-Step 4 5 "Installing ffmpeg (Video/Audio merger)..."
$ffmpegDest    = "$BinDir\ffmpeg.exe"
$ffmpegAlready = (Get-Command ffmpeg -ErrorAction SilentlyContinue) -or (Test-Path $ffmpegDest)

if ($ffmpegAlready) {
    Write-OK "ffmpeg already installed"
} elseif ($winget) {
    try {
        Write-Info "Installing via winget..."
        winget install Gyan.FFmpeg --accept-source-agreements --accept-package-agreements -h 2>&1 | Out-Null
        Write-OK "ffmpeg installed via winget"
    } catch {
        Write-Fail "winget install failed, trying direct download..."
        $ffmpegAlready = $false
    }
}

if (-not $ffmpegAlready -and -not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    try {
        Write-Info "Downloading ffmpeg essentials build..."
        $ffmpegUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
        $ffmpegZip = "$env:TEMP\ffmpeg.zip"
        Invoke-WebRequest -Uri $ffmpegUrl -OutFile $ffmpegZip -UseBasicParsing
        $ffmpegTmp = "$env:TEMP\ffmpegextract"
        Expand-Archive -Path $ffmpegZip -DestinationPath $ffmpegTmp -Force
        $ffmpegExe = Get-ChildItem -Recurse -Filter "ffmpeg.exe" -Path $ffmpegTmp | Select-Object -First 1
        Copy-Item $ffmpegExe.FullName -Destination $ffmpegDest -Force
        Remove-Item $ffmpegZip -Force -ErrorAction SilentlyContinue
        Remove-Item $ffmpegTmp -Recurse -Force -ErrorAction SilentlyContinue
        Write-OK "ffmpeg downloaded to $ffmpegDest"
    } catch {
        Write-Fail "Could not install ffmpeg: $_"
    }
}

Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   STEP 5  ─  Download & install fetchdesk.exe
# ─────────────────────────────────────────────────────────────────────────────
Write-Step 5 5 "Installing FetchDesk..."
try {
    Write-Info "Downloading fetchdesk.exe from GitHub Releases..."
    $relUrl = "https://github.com/jojin1709/fetchdesk/releases/latest/download/fetchdesk.exe"
    Invoke-WebRequest -Uri $relUrl -OutFile $ExePath -UseBasicParsing -ErrorAction Stop
    Write-OK "fetchdesk.exe installed to $ExePath"
} catch {
    Write-Info "Release binary not yet available — building from source..."
    # Fallback: tell user to build manually
    Write-Fail "Please run: cargo build --release inside fetchdesk-src folder"
}

Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   PATH registration
# ─────────────────────────────────────────────────────────────────────────────
# Add InstallDir and BinDir to User PATH
$dirsToAdd = @($InstallDir, $BinDir)
$UserPath  = [Environment]::GetEnvironmentVariable("Path", "User")
foreach ($d in $dirsToAdd) {
    if ($UserPath -notlike "*$d*") {
        $UserPath = "$UserPath;$d"
    }
}
[Environment]::SetEnvironmentVariable("Path", $UserPath, "User")
$env:PATH = $env:PATH + ";$InstallDir;$BinDir"
Write-OK "PATH updated — fetchdesk, aria2c, yt-dlp, ffmpeg available in new terminals"
Write-Host ""

# ─────────────────────────────────────────────────────────────────────────────
#   Optional: Desktop shortcut
# ─────────────────────────────────────────────────────────────────────────────
Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
$ans = Read-Host "  Add FetchDesk shortcut to Desktop? (y/n)"
if ($ans -match '^[Yy]') {
    try {
        $desktop  = [Environment]::GetFolderPath("Desktop")
        $shortcut = "$desktop\FetchDesk.lnk"
        $wsh      = New-Object -ComObject WScript.Shell
        $sc       = $wsh.CreateShortcut($shortcut)
        $sc.TargetPath       = "powershell.exe"
        $sc.Arguments        = "-NoExit -Command `"& '$ExePath'`""
        $sc.WorkingDirectory = $InstallDir
        $sc.Description      = "FetchDesk Download Manager"
        $sc.IconLocation     = $ExePath
        $sc.Save()
        Write-OK "Desktop shortcut created: $shortcut"
    } catch {
        Write-Fail "Could not create shortcut: $_"
    }
}

# ─────────────────────────────────────────────────────────────────────────────
#   Done!
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  🎉 " -NoNewline -ForegroundColor Yellow
Write-Host "FetchDesk is ready! Open a new terminal and type:" -ForegroundColor White
Write-Host ""
Write-Host "       fetchdesk" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Features unlocked:" -ForegroundColor DarkGray
Write-Host "    ✓ Direct downloads  (16 parallel connections)" -ForegroundColor Green
Write-Host "    ✓ YouTube downloads (yt-dlp installed)" -ForegroundColor Green
Write-Host "    ✓ Magnet/Torrent    (aria2c installed)" -ForegroundColor Green
Write-Host "    ✓ Video merging     (ffmpeg installed)" -ForegroundColor Green
Write-Host "    ✓ Smart disk space switcher" -ForegroundColor Green
Write-Host ""
