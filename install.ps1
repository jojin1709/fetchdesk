#Requires -Version 5.1
# FetchDesk - Automated Setup Script
# Run: iwr -useb https://raw.githubusercontent.com/jojin1709/fetchdesk/main/install.ps1 | iex

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Write-Header {
    Clear-Host
    Write-Host ""
    Write-Host "   ___ ___ ___ ___ _  _ ___  ___ ___ _  _" -ForegroundColor Cyan
    Write-Host "  | __| __|_ _| __| || |   \| __/ __| |/ /" -ForegroundColor Cyan
    Write-Host "  | _|| _| | || (__| __ | |) | _|__ \   <" -ForegroundColor Cyan
    Write-Host "  |_| |___|___|\___|_||_|___/|___|___/\_/\_" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  ==========================================================================" -ForegroundColor DarkGray
    Write-Host "    v0.2.0  *  Terminal-Based Multi-Connection Download Manager  *  by Jojin John" -ForegroundColor DarkGray
    Write-Host "  ==========================================================================" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step($num, $total, $msg) {
    Write-Host "  [$num/$total] " -ForegroundColor DarkCyan -NoNewline
    Write-Host $msg -ForegroundColor White
}

function Write-OK($msg) {
    Write-Host "        [+] " -ForegroundColor Green -NoNewline
    Write-Host $msg -ForegroundColor Gray
}

function Write-Fail($msg) {
    Write-Host "        [!] " -ForegroundColor Red -NoNewline
    Write-Host $msg -ForegroundColor Red
}

function Write-Info($msg) {
    Write-Host "          -> " -ForegroundColor DarkGray -NoNewline
    Write-Host $msg -ForegroundColor DarkGray
}

function Download-WithProgress($url, $outputFile) {
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
    $request = [System.Net.HttpWebRequest]::Create($url)
    $request.Method = "GET"
    $request.UserAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) FetchDesk/0.2.0"
    $request.AutomaticDecompression = [System.Net.DecompressionMethods]::GZip -bor [System.Net.DecompressionMethods]::Deflate
    
    try {
        $response = $request.GetResponse()
    } catch {
        throw $_
    }
    
    $totalBytes = $response.ContentLength
    $responseStream = $response.GetResponseStream()
    $targetStream = [System.IO.File]::Create($outputFile)
    $buffer = New-Object Byte[] 16384
    $bytesRead = 0
    $totalRead = 0
    $width = 30
    
    do {
        $bytesRead = $responseStream.Read($buffer, 0, $buffer.Length)
        if ($bytesRead -gt 0) {
            $targetStream.Write($buffer, 0, $bytesRead)
            $totalRead += $bytesRead
            
            if ($totalBytes -gt 0) {
                $percent = ($totalRead / $totalBytes) * 100
                $filled = [Math]::Floor(($totalRead / $totalBytes) * $width)
                $empty = $width - $filled
                $bar = ("=" * $filled) + (" " * $empty)
                $mbRead = [Math]::Round($totalRead / 1MB, 1)
                $mbTotal = [Math]::Round($totalBytes / 1MB, 1)
                $statusStr = "`r          [ $bar ] $([Math]::Round($percent, 1))% ($mbRead MB / $mbTotal MB)"
                Write-Host -NoNewline $statusStr
            } else {
                $mbRead = [Math]::Round($totalRead / 1MB, 1)
                Write-Host -NoNewline "`r          Downloading: $mbRead MB"
            }
        }
    } while ($bytesRead -gt 0)
    
    Write-Host ""
    $responseStream.Close()
    $targetStream.Close()
    $response.Close()
}

# -- Paths --------------------------------------------------------------------
$InstallDir = "$env:LOCALAPPDATA\FetchDesk"
$BinDir     = "$InstallDir\bin"
$ExePath    = "$InstallDir\fetchdesk.exe"

foreach ($dir in @($InstallDir, $BinDir)) {
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
}

Write-Header

# -----------------------------------------------------------------------------
#   STEP 1  ─  Pre-flight system check
# -----------------------------------------------------------------------------
Write-Step 1 6 "Checking system requirements..."
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-OK "Windows $([System.Environment]::OSVersion.Version.Major).$([System.Environment]::OSVersion.Version.Minor), $arch"

# Check winget
$winget = Get-Command winget -ErrorAction SilentlyContinue
if ($winget) { Write-OK "winget found ($($winget.Source))" }
else         { Write-Info "winget not found - will use direct downloads" }

# Check pip / Python
$pip = Get-Command pip -ErrorAction SilentlyContinue
if ($pip) { Write-OK "pip found" }
else      { Write-Info "pip not found - will install yt-dlp via standalone binary" }

Write-Host ""

# -----------------------------------------------------------------------------
#   STEP 2  ─  Download & install aria2c
# -----------------------------------------------------------------------------
Write-Step 2 6 "Installing aria2c (BitTorrent & Magnet engine)..."
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
        Download-WithProgress $aria2Url $aria2Zip
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

# -----------------------------------------------------------------------------
#   STEP 3  ─  Install yt-dlp
# -----------------------------------------------------------------------------
Write-Step 3 6 "Installing yt-dlp (YouTube engine)..."
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
        Download-WithProgress $ytdlpUrl $ytdlpDest
        Write-OK "yt-dlp downloaded to $ytdlpDest"
    } catch {
        Write-Fail "Could not install yt-dlp: $_"
    }
}

Write-Host ""

# -----------------------------------------------------------------------------
#   STEP 4  ─  Install ffmpeg
# -----------------------------------------------------------------------------
Write-Step 4 6 "Installing ffmpeg (Video/Audio merger)..."
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
        Download-WithProgress $ffmpegUrl $ffmpegZip
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

# -----------------------------------------------------------------------------
#   STEP 5  ─  Download & install fetchdesk.exe
# -----------------------------------------------------------------------------
Write-Step 5 6 "Installing FetchDesk..."
try {
    Write-Info "Downloading fetchdesk.exe from GitHub Releases..."
    $relUrl = "https://github.com/jojin1709/fetchdesk/releases/latest/download/fetchdesk.exe"
    Download-WithProgress $relUrl $ExePath
    Write-OK "fetchdesk.exe installed to $ExePath"
} catch {
    try {
        Write-Info "Release binary not available on GitHub Releases - trying direct download from repository..."
        $binUrl = "https://raw.githubusercontent.com/jojin1709/fetchdesk/main/bin/fetchdesk.exe"
        Download-WithProgress $binUrl $ExePath
        Write-OK "fetchdesk.exe downloaded from repository bin folder"
    } catch {
        Write-Info "Repository binary not available - checking for local project build..."
        $localBuild = "target\release\fetchdesk.exe"
        if ($PSScriptRoot) {
            $localBuild = Join-Path $PSScriptRoot "target\release\fetchdesk.exe"
        }
        if (Test-Path $localBuild) {
            Write-Info "Found local build at target\release\fetchdesk.exe"
            Copy-Item $localBuild -Destination $ExePath -Force
            Write-OK "Installed local build to $ExePath"
        } elseif (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Info "cargo found! Building FetchDesk from source locally..."
            $oldPath = Get-Location
            try {
                if ($PSScriptRoot) {
                    if (Test-Path (Join-Path $PSScriptRoot "Cargo.toml")) {
                        Set-Location $PSScriptRoot
                    }
                }
                cargo build --release
                if (Test-Path $localBuild) {
                    Copy-Item $localBuild -Destination $ExePath -Force
                    Write-OK "Built and installed FetchDesk to $ExePath"
                } else {
                    throw "Output binary target\release\fetchdesk.exe not found after build."
                }
            } catch {
                Write-Fail "Local build failed: $_"
                Write-Fail "Please run: cargo build --release inside the source folder manually."
            } finally {
                Set-Location $oldPath
            }
        } else {
            Write-Fail "Release binary not available and cargo is not installed."
            Write-Fail "Please build or copy fetchdesk.exe manually to $ExePath"
        }
    }
}

Write-Host ""

# -----------------------------------------------------------------------------
#   STEP 6  ─  Download Chrome Extension files
# -----------------------------------------------------------------------------
Write-Step 6 6 "Downloading Chrome Extension files..."
try {
    $ExtDir = "$InstallDir\extension"
    if (!(Test-Path $ExtDir)) { New-Item -ItemType Directory -Force -Path $ExtDir | Out-Null }
    
    $baseUrl = "https://raw.githubusercontent.com/jojin1709/fetchdesk/main/extension"
    $files = @("manifest.json", "background.js", "popup.html", "popup.js", "icon16.png", "icon48.png", "icon128.png")
    
    foreach ($file in $files) {
        Write-Info "Downloading $file..."
        Invoke-WebRequest -Uri "$baseUrl/$file" -OutFile "$ExtDir\$file" -UseBasicParsing -ErrorAction Stop
    }
    Write-OK "Chrome Extension downloaded to $ExtDir"
} catch {
    Write-Fail "Could not download Chrome Extension: $_"
}

Write-Host ""

# -----------------------------------------------------------------------------
#   PATH registration
# -----------------------------------------------------------------------------
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
Write-OK "PATH updated - fetchdesk, aria2c, yt-dlp, ffmpeg available in new terminals"
Write-Host ""

# -----------------------------------------------------------------------------
#   Generate Shortcut Icon
# -----------------------------------------------------------------------------
$icoPath = "$InstallDir\fetchdesk.ico"
$pngPath = "$InstallDir\extension\icon128.png"
if (Test-Path $pngPath) {
    try {
        [System.Reflection.Assembly]::LoadWithPartialName("System.Drawing") | Out-Null
        $img = New-Object System.Drawing.Bitmap($pngPath)
        $hIcon = $img.GetHicon()
        $icon = [System.Drawing.Icon]::FromHandle($hIcon)
        $stream = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
        $icon.Save($stream)
        $stream.Close()
        $img.Dispose()
        Write-OK "Created application icon at $icoPath"
    } catch {
        Write-Info "Could not generate .ico file: $_"
    }
}

# -----------------------------------------------------------------------------
#   Optional: Desktop shortcut
# -----------------------------------------------------------------------------
Write-Host "  ==========================================================================" -ForegroundColor DarkGray
$ans = Read-Host "  Add FetchDesk shortcut to Desktop? (y/n)"
if ($ans -match "^[Yy]") {
    try {
        $desktop  = [Environment]::GetFolderPath("Desktop")
        $shortcut = "$desktop\FetchDesk.lnk"
        $wsh      = New-Object -ComObject WScript.Shell
        $sc       = $wsh.CreateShortcut($shortcut)
        $sc.TargetPath       = "powershell.exe"
        $sc.Arguments        = "-NoExit -Command " + [char]34 + $ExePath + [char]34
        $sc.WorkingDirectory = $InstallDir
        $sc.Description      = "FetchDesk Download Manager"
        $sc.IconLocation     = if (Test-Path $icoPath) { $icoPath } else { $ExePath }
        $sc.Save()
        Write-OK "Desktop shortcut created: $shortcut"
    } catch {
        Write-Fail "Could not create shortcut: $_"
    }
}

# -----------------------------------------------------------------------------
#   Done!
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "  ==========================================================================" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  [Done] " -NoNewline -ForegroundColor Yellow
Write-Host "FetchDesk is ready! Open a new terminal and type:" -ForegroundColor White
Write-Host ""
Write-Host "       fetchdesk" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Features unlocked:" -ForegroundColor DarkGray
Write-Host "    [+] Direct downloads  (16 parallel connections)" -ForegroundColor Green
Write-Host "    [+] YouTube downloads (yt-dlp installed)" -ForegroundColor Green
Write-Host "    [+] Magnet/Torrent    (aria2c installed)" -ForegroundColor Green
Write-Host "    [+] Video merging     (ffmpeg installed)" -ForegroundColor Green
Write-Host "    [+] Smart disk space switcher" -ForegroundColor Green
Write-Host "    [+] Chrome Extension  (Downloaded to $InstallDir\extension)" -ForegroundColor Green
Write-Host ""
Write-Host "  To enable Chrome/Edge integration:" -ForegroundColor White
Write-Host "    1. Open chrome://extensions in your browser"
Write-Host "    2. Turn on Developer Mode (toggle in the top-right corner)"
Write-Host "    3. Click Load Unpacked (top-left) and select this folder:"
Write-Host "       $InstallDir\extension"
Write-Host ""
