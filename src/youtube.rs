use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::banner;
use crate::config::Config;
use crate::downloader::human_size;

pub struct VideoFormat {
    #[allow(dead_code)]
    pub id: String,
    pub display: String,
    #[allow(dead_code)]
    pub format: String,
    pub size: String,
    pub single_file: bool,
}

pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub uploader: String,
    pub webpage_url: String,
}

/// Get video info without downloading
pub fn get_video_info(url: &str) -> Result<VideoInfo> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    let output = Command::new(&ytdlp_path)
        .arg(url)
        .arg("--dump-json")
        .arg("--no-warnings")
        .arg("--no-playlist")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get video info"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| anyhow!("Failed to parse video info: {}", e))?;

    let duration_str = if let Some(ds) = json["duration_string"].as_str() {
        ds.to_string()
    } else if let Some(secs) = json["duration"].as_f64() {
        let s = secs as u64;
        format!("{:02}:{:02}", s / 60, s % 60)
    } else if let Some(secs) = json["duration"].as_u64() {
        format!("{:02}:{:02}", secs / 60, secs % 60)
    } else {
        "?".to_string()
    };

    Ok(VideoInfo {
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        duration: duration_str,
        uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
        webpage_url: json["webpage_url"].as_str().unwrap_or(url).to_string(),
    })
}

/// Get available qualities for a YouTube video using yt-dlp JSON output
pub async fn get_available_qualities(url: &str) -> Result<Vec<VideoFormat>> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    banner::print_info("Fetching available formats...");

    let output = Command::new(&ytdlp_path)
        .arg(url)
        .arg("-J")
        .arg("--no-warnings")
        .arg("--no-playlist")
        .output()?;

    let mut formats = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(fmts) = json["formats"].as_array() {
                for fmt_obj in fmts {
                    let format_id = fmt_obj["format_id"].as_str().unwrap_or("").to_string();
                    if format_id.is_empty() {
                        continue;
                    }

                    let vcodec = fmt_obj["vcodec"].as_str().unwrap_or("none");
                    let acodec = fmt_obj["acodec"].as_str().unwrap_or("none");
                    let height = fmt_obj["height"].as_u64();
                    let filesize = fmt_obj["filesize"]
                        .as_u64()
                        .or_else(|| fmt_obj["filesize_approx"].as_u64());

                    let size = filesize
                        .map(human_size)
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (display, single_file, req_format) = if vcodec == "none" && acodec != "none" {
                        ("Audio only".to_string(), true, format_id.clone())
                    } else if let Some(h) = height {
                        let label = match h {
                            2160 => "4K (2160p)".to_string(),
                            1440 => "2K (1440p)".to_string(),
                            1080 => "1080p (Full HD)".to_string(),
                            720 => "720p (HD)".to_string(),
                            480 => "480p (SD)".to_string(),
                            360 => "360p".to_string(),
                            240 => "240p".to_string(),
                            144 => "144p".to_string(),
                            other => format!("{}p", other),
                        };
                        let is_single = acodec != "none";
                        let req_fmt = format!(
                            "best[height<={}]/bestvideo[height<={}] +bestaudio/best",
                            h, h
                        ).replace(" ", "");
                        (label, is_single, req_fmt)
                    } else {
                        continue;
                    };

                    // Avoid duplicates in display resolution list
                    if !formats.iter().any(|f: &VideoFormat| f.display == display) {
                        formats.push(VideoFormat {
                            id: format_id,
                            display,
                            format: req_format,
                            size,
                            single_file,
                        });
                    }
                }
            }
        }
    }

    formats.sort_by(|a, b| {
        let get_height = |s: &str| -> u32 {
            if s.contains("4K") { 2160 }
            else if s.contains("2K") { 1440 }
            else if s.contains("1080") { 1080 }
            else if s.contains("720") { 720 }
            else if s.contains("480") { 480 }
            else if s.contains("360") { 360 }
            else if s.contains("240") { 240 }
            else if s.contains("144") { 144 }
            else if s.contains("Audio") { 0 }
            else { 0 }
        };
        get_height(&b.display).cmp(&get_height(&a.display))
    });

    Ok(formats)
}

/// Download YouTube video using yt-dlp with anti-403 configuration and animated progress bar
pub async fn download_youtube(
    url: &str,
    out_dir: &PathBuf,
    config: &Config,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    silent: bool,
) -> Result<()> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    std::fs::create_dir_all(out_dir)?;
    let out_tmpl = out_dir.join("%(title)s.%(ext)s");

    let mut cmd = Command::new(&ytdlp_path);
    cmd.arg(url)
        .arg("-o")
        .arg(out_tmpl.to_string_lossy().to_string())
        .arg("--newline")
        .arg("--no-part")
        .arg("--no-warnings")
        .arg("--progress")
        .arg("--extractor-args")
        .arg("youtube:player_client=mweb,default")
        .arg("--user-agent")
        .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .arg("--no-cache-dir")
        .arg("--concurrent-fragments")
        .arg("16")
        .arg("--buffer-size")
        .arg("1024k")
        .arg("--http-chunk-size")
        .arg("10M");

    // Quality
    let quality = config.youtube.quality.as_str();
    if quality == "bestaudio" || quality == "ba" {
        let audio_fmt = if config.youtube.format == "mp4" || config.youtube.format == "best" {
            "m4a"
        } else {
            &config.youtube.format
        };
        cmd.arg("-x")
            .arg("--audio-format")
            .arg(audio_fmt)
            .arg("-f")
            .arg(quality);
    } else if quality == "best" {
        // Let yt-dlp choose
    } else {
        cmd.arg("-f").arg(quality);
    }

    // Format
    if config.youtube.format != "mp4" && config.youtube.format != "best" {
        cmd.arg("--merge-output-format").arg(&config.youtube.format);
    }

    // Subtitles
    if config.youtube.write_subtitles {
        cmd.arg("--write-subs");
        cmd.arg("--sub-langs");
        cmd.arg(config.youtube.subtitle_langs.join(","));
        if !config.youtube.write_info_json {
            cmd.arg("--no-write-info-json");
        }
    }

    // Thumbnail
    if config.youtube.write_thumbnail {
        cmd.arg("--write-thumbnail");
    }

    // Info JSON
    if config.youtube.write_info_json {
        cmd.arg("--write-info-json");
    }

    // FFmpeg
    if let Some(ffmpeg_path) = which("ffmpeg") {
        cmd.arg("--ffmpeg-location").arg(&ffmpeg_path);
    }

    // Proxy
    if let Some(ref proxy) = config.network.proxy {
        cmd.arg("--proxy").arg(proxy);
    }

    // Cookies
    if let Some(ref cookies) = config.network.cookies_file {
        cmd.arg("--cookies").arg(cookies);
    }

    if config.download.dry_run {
        if !silent {
            banner::print_info("DRY RUN - would download:");
            banner::print_info(&format!("URL: {}", url));
            banner::print_info(&format!("Quality: {}", quality));
            banner::print_info(&format!("Output: {}", out_dir.display()));
        }
        return Ok(());
    }

    if !silent {
        banner::print_info("Downloading...");
    }

    run_ytdlp_with_progress(cmd, progress_callback, silent)?;

    if !silent {
        banner::print_success(&format!("Saved to: {}", out_dir.display()));
    }
    Ok(())
}

/// Download YouTube playlist
pub async fn download_playlist(
    url: &str,
    out_dir: &PathBuf,
    config: &Config,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    silent: bool,
) -> Result<()> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    std::fs::create_dir_all(out_dir)?;
    let out_tmpl = out_dir.join("%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s");

    let mut cmd = Command::new(&ytdlp_path);
    cmd.arg(url)
        .arg("-o")
        .arg(out_tmpl.to_string_lossy().to_string())
        .arg("--newline")
        .arg("--no-part")
        .arg("--no-warnings")
        .arg("--progress")
        .arg("--yes-playlist")
        .arg("--extractor-args")
        .arg("youtube:player_client=mweb,default")
        .arg("--user-agent")
        .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .arg("--no-cache-dir")
        .arg("--concurrent-fragments")
        .arg("16")
        .arg("--buffer-size")
        .arg("1024k")
        .arg("--http-chunk-size")
        .arg("10M");

    // Quality
    let quality = config.youtube.quality.as_str();
    if quality == "bestaudio" || quality == "ba" {
        let audio_fmt = if config.youtube.format == "mp4" || config.youtube.format == "best" {
            "m4a"
        } else {
            &config.youtube.format
        };
        cmd.arg("-x")
            .arg("--audio-format")
            .arg(audio_fmt)
            .arg("-f")
            .arg(quality);
    } else if quality != "best" {
        cmd.arg("-f").arg(quality);
    }

    if config.youtube.format != "mp4" && config.youtube.format != "best" {
        cmd.arg("--merge-output-format").arg(&config.youtube.format);
    }

    if config.youtube.write_subtitles {
        cmd.arg("--write-subs");
        cmd.arg("--sub-langs");
        cmd.arg(config.youtube.subtitle_langs.join(","));
    }

    if config.youtube.write_thumbnail {
        cmd.arg("--write-thumbnail");
    }

    if let Some(ffmpeg_path) = which("ffmpeg") {
        cmd.arg("--ffmpeg-location").arg(&ffmpeg_path);
    }

    if let Some(ref proxy) = config.network.proxy {
        cmd.arg("--proxy").arg(proxy);
    }

    // Playlist range
    if let Some(start) = config.youtube.playlist_start {
        cmd.arg("--playlist-start").arg(start.to_string());
    }
    if let Some(end) = config.youtube.playlist_end {
        cmd.arg("--playlist-end").arg(end.to_string());
    }

    if config.download.dry_run {
        if !silent {
            banner::print_info("DRY RUN - would download playlist:");
            banner::print_info(&format!("URL: {}", url));
        }
        return Ok(());
    }

    if !silent {
        banner::print_info("Downloading playlist...");
    }

    run_ytdlp_with_progress(cmd, progress_callback, silent)?;

    if !silent {
        banner::print_success(&format!("Playlist saved to: {}", out_dir.display()));
    }
    Ok(())
}

fn run_ytdlp_with_progress(
    mut cmd: Command,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    silent: bool,
) -> Result<()> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let pb = if silent {
        ProgressBar::hidden()
    } else {
        let p = ProgressBar::new(1000);
        p.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} [{bar:40.cyan/blue}] {percent}% | {msg}",
            )?
            .progress_chars("█▓░ "),
        );
        p
    };

    let pb_clone = pb.clone();
    let progress_cb = progress_callback.clone();
    let stdout_handle = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().flatten() {
            if line.contains("[download]") && line.contains('%') {
                if let Some(pct_idx) = line.find('%') {
                    let start = line[..pct_idx].rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
                    if let Ok(pct) = line[start..pct_idx].trim().parse::<f64>() {
                        pb_clone.set_position((pct * 10.0) as u64);
                        if let Some(ref cb) = progress_cb {
                            cb(pct);
                        }
                    }
                }
                if let Some(at_idx) = line.find(" at ") {
                    let sub = &line[at_idx + 4..];
                    pb_clone.set_message(sub.trim().to_string());
                } else {
                    pb_clone.set_message(line.replace("[download]", "").trim().to_string());
                }
            } else if line.contains("[Merger]") || line.contains("Merging") {
                pb_clone.set_message("Merging audio and video...".to_string());
            } else if line.contains("Extracting") || line.contains("Downloading webpage") {
                pb_clone.set_message("Fetching video metadata...".to_string());
            }
        }
    });

    let err_msgs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let err_msgs_clone = std::sync::Arc::clone(&err_msgs);
    let stderr_handle = std::thread::spawn(move || {
        let err_reader = std::io::BufReader::new(stderr);
        for line in err_reader.lines().flatten() {
            if line.contains("ERROR:") || line.contains("error:") {
                err_msgs_clone.lock().unwrap().push(line);
            }
        }
    });

    let status = child.wait()?;
    stdout_handle.join().ok();
    stderr_handle.join().ok();

    pb.finish_and_clear();

    if !status.success() {
        let errors = err_msgs.lock().unwrap();
        if !errors.is_empty() {
            return Err(anyhow!(errors.join("\n")));
        }
        return Err(anyhow!("yt-dlp process failed (exit status: {})", status));
    }

    Ok(())
}

/// Search YouTube and return results
pub fn search_youtube(query: &str, max_results: usize) -> Result<Vec<(String, String, String)>> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    let search_url = format!("ytsearch{}:{}", max_results, query);

    let output = Command::new(&ytdlp_path)
        .arg(&search_url)
        .arg("--flat-playlist")
        .arg("--dump-json")
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Search failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let title = json["title"].as_str().unwrap_or("Unknown").to_string();
            let url = json["url"]
                .as_str()
                .or_else(|| json["webpage_url"].as_str())
                .unwrap_or("")
                .to_string();
            let duration = if let Some(ds) = json["duration_string"].as_str() {
                ds.to_string()
            } else if let Some(secs) = json["duration"].as_u64() {
                format!("{:02}:{:02}", secs / 60, secs % 60)
            } else {
                "?".to_string()
            };

            if !url.is_empty() {
                results.push((title, url, duration));
            }
        }
    }

    Ok(results)
}

/// Get playlist entries
pub fn get_playlist_entries(url: &str) -> Result<Vec<(usize, String, String)>> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    let output = Command::new(&ytdlp_path)
        .arg(url)
        .arg("--flat-playlist")
        .arg("--dump-json")
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get playlist entries"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for (i, line) in stdout.lines().enumerate() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let title = json["title"].as_str().unwrap_or("Unknown").to_string();
            let url = json["url"]
                .as_str()
                .or_else(|| json["webpage_url"].as_str())
                .unwrap_or("")
                .to_string();

            if !url.is_empty() {
                entries.push((i + 1, title, url));
            }
        }
    }

    Ok(entries)
}

#[allow(dead_code)]
pub fn list_formats(url: &str) -> Result<()> {
    let ytdlp_path = which("yt-dlp").ok_or_else(|| {
        anyhow!("yt-dlp not found. Install: pip install -U yt-dlp")
    })?;

    banner::print_info("Fetching available formats...");

    let status = Command::new(&ytdlp_path)
        .arg(url)
        .arg("-F")
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to list formats"));
    }

    Ok(())
}

pub fn which(bin: &str) -> Option<PathBuf> {
    let exts = if cfg!(windows) {
        vec!["", ".exe", ".cmd", ".bat"]
    } else {
        vec![""]
    };

    // Check local FetchDesk bin directory first
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let fetch_bin_dir = PathBuf::from(local_app_data).join("FetchDesk").join("bin");
        for ext in &exts {
            let candidate = fetch_bin_dir.join(format!("{}{}", bin, ext));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if let Some(path_var) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&path_var) {
            for ext in &exts {
                let candidate = p.join(format!("{}{}", bin, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    let home_opt = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    if let Some(home) = home_opt {
        let app_data = PathBuf::from(&home).join("AppData");
        let python_dir = app_data.join("Roaming").join("Python");
        if python_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&python_dir) {
                for entry in entries.flatten() {
                    let scripts = entry.path().join("Scripts");
                    if scripts.is_dir() {
                        for ext in &exts {
                            let candidate = scripts.join(format!("{}{}", bin, ext));
                            if candidate.is_file() {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }

        if bin == "ffmpeg" || bin == "ffmpeg.exe" {
            if python_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&python_dir) {
                    for entry in entries.flatten() {
                        let binaries = entry
                            .path()
                            .join("site-packages")
                            .join("imageio_ffmpeg")
                            .join("binaries");
                        if binaries.is_dir() {
                            if let Ok(b_entries) = std::fs::read_dir(&binaries) {
                                for b_entry in b_entries.flatten() {
                                    let path = b_entry.path();
                                    if path.is_file() && path.to_string_lossy().contains("ffmpeg") {
                                        return Some(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
