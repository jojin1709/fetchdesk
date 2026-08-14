use anyhow::{anyhow, Result};
use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT_RANGES, CONTENT_LENGTH, RANGE};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::banner;
use crate::config::Config;

/// Download a direct HTTP(S) file with full features
pub async fn download_direct(url: &str, out_dir: &PathBuf, config: &Config) -> Result<()> {
    let mut builder = reqwest::Client::builder()
        .user_agent(&config.network.user_agent)
        .connect_timeout(Duration::from_secs(30));

    if config.download.timeout_secs > 0 {
        builder = builder.timeout(Duration::from_secs(config.download.timeout_secs));
    }

    // Proxy
    if let Some(ref proxy) = config.network.proxy {
        let proxy = reqwest::Proxy::all(proxy)?;
        builder = builder.proxy(proxy);
    }

    // Cookies file
    if let Some(ref cookies_file) = config.network.cookies_file {
        let cookie_jar = reqwest::cookie::Jar::default();
        if let Ok(content) = std::fs::read_to_string(cookies_file) {
            for line in content.lines() {
                if !line.starts_with('#') && !line.is_empty() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 7 {
                        let domain = parts[0];
                        let path = parts[2];
                        let name = parts[5];
                        let value = parts[6];
                        let url_str = format!("https://{}{}", domain, path);
                        if let Ok(url) = url_str.parse() {
                            cookie_jar.add_cookie_str(
                                &format!("{}={}", name, value),
                                &url,
                            );
                        }
                    }
                }
            }
        }
        builder = builder.cookie_provider(Arc::new(cookie_jar));
    }

    // Custom headers
    let mut header_map = HeaderMap::new();
    for (key, value) in &config.network.headers {
        if let (Ok(k), Ok(v)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            header_map.insert(k, v);
        }
    }
    builder = builder.default_headers(header_map);

    let client = builder.build()?;

    let mut req = client.head(url);
    for (key, value) in &config.network.headers {
        req = req.header(key.as_str(), value.as_str());
    }
    let head = req.send().await;

    let (supports_ranges, total_size, filename_header) = match &head {
        Ok(resp) if resp.status().is_success() => {
            let ranges_ok = resp
                .headers()
                .get(ACCEPT_RANGES)
                .map(|v| v != "none")
                .unwrap_or(false);
            let len = resp
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            let fname = resp
                .headers()
                .get(reqwest::header::CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok())
                .and_then(filename_from_content_disposition);
            (ranges_ok, len, fname)
        }
        _ => (false, None, None),
    };

    let filename = filename_header.unwrap_or_else(|| filename_from_url(url));
    std::fs::create_dir_all(out_dir)?;

    // Auto-organize by file type
    let actual_out_dir = if config.download.auto_organize {
        organize_dir(out_dir, &filename)
    } else {
        out_dir.clone()
    };
    std::fs::create_dir_all(&actual_out_dir)?;

    let out_path = actual_out_dir.join(&filename);

    // Duplicate detection: only skip if file exists AND matches expected total size
    if out_path.exists() {
        if let Some(size) = total_size {
            if let Ok(meta) = std::fs::metadata(&out_path) {
                if meta.len() == size {
                    banner::print_info("File already exists and size matches, skipping...");
                    banner::print_success(&format!("Saved to: {}", out_path.display()));
                    return Ok(());
                }
            }
        }
    }

    if config.download.dry_run {
        banner::print_info("DRY RUN - would download:");
        banner::print_info(&format!("URL: {}", url));
        banner::print_info(&format!("Output: {}", out_path.display()));
        if let Some(size) = total_size {
            banner::print_info(&format!("Size: {}", human_size(size)));
        }
        return Ok(());
    }

    let out_path = if let Some(size) = total_size {
        let parent = out_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| out_dir.clone());
        let new_parent = crate::disk::ensure_sufficient_space(&parent, size);
        new_parent.join(&filename)
    } else {
        out_path
    };

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Resume check
    let resume_from = if config.download.auto_resume && out_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&out_path) {
            if supports_ranges && total_size.is_some() {
                let current = metadata.len();
                let total = total_size.unwrap();
                if current > 0 && current < total {
                    banner::print_info(&format!(
                        "Resuming from {} / {}",
                        human_size(current),
                        human_size(total)
                    ));
                    Some(current)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if supports_ranges && total_size.is_some() {
        let size = total_size.unwrap();
        let connections = config.download.max_connections.max(1);
        banner::print_info(&format!(
            "Downloading: {} ({}) using {} connections",
            filename,
            human_size(size),
            connections
        ));
        download_chunked(
            &client,
            url,
            &out_path,
            size,
            connections,
            config.download.bandwidth_limit_kbps,
            resume_from,
        )
        .await?;
    } else {
        banner::print_info(&format!("Downloading: {} (single stream)", filename));
        download_single(
            &client,
            url,
            &out_path,
            config.download.bandwidth_limit_kbps,
            resume_from,
        )
        .await?;
    }

    // Hash verification
    if config.download.hash_verify && out_path.exists() {
        verify_hash(&out_path)?;
    }

    banner::print_success(&format!("Saved to: {}", out_path.display()));
    Ok(())
}

async fn download_chunked(
    client: &reqwest::Client,
    url: &str,
    out_path: &PathBuf,
    size: u64,
    connections: usize,
    bandwidth_limit_kbps: Option<u64>,
    resume_from: Option<u64>,
) -> Result<()> {
    let start_offset = resume_from.unwrap_or(0);
    let remaining = size.saturating_sub(start_offset);
    if remaining == 0 {
        return Ok(());
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = if resume_from.is_some() && out_path.exists() {
        OpenOptions::new().write(true).open(out_path)?
    } else {
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(out_path)?;
        f.set_len(size)?;
        f
    };
    drop(file);

    let conn_count = connections.min(remaining as usize).max(1);
    let chunk_size = remaining / conn_count as u64;
    
    let pb = ProgressBar::new(remaining);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {percent}% | {bytes}/{total_bytes} | {speed} | ETA {eta}",
        )?
        .progress_chars("█▓░ "),
    );

    let mut handles = Vec::new();

    for i in 0..conn_count {
        let start = start_offset + i as u64 * chunk_size;
        let end = if i == conn_count - 1 {
            size - 1
        } else {
            start + chunk_size - 1
        };

        if start > end {
            continue;
        }

        let pb_clone = pb.clone();
        let client = client.clone();
        let url = url.to_string();
        let out_path = out_path.clone();
        let limit = bandwidth_limit_kbps.map(|k| k / conn_count as u64);

        handles.push(tokio::spawn(async move {
            let req = client.get(&url).header(RANGE, format!("bytes={}-{}", start, end));
            let resp = req.send().await?.error_for_status()?;

            let mut stream = resp.bytes_stream();
            let mut offset = start;
            let mut bytes_since_limit_check = 0u64;
            let limit_bytes_per_sec = limit.map(|k| k * 1024);

            let mut thread_file = OpenOptions::new().write(true).open(&out_path)?;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                thread_file.seek(SeekFrom::Start(offset))?;
                thread_file.write_all(&chunk)?;
                offset += chunk.len() as u64;
                pb_clone.inc(chunk.len() as u64);

                // Bandwidth throttle
                if let Some(limit_bps) = limit_bytes_per_sec {
                    bytes_since_limit_check += chunk.len() as u64;
                    if limit_bps > 0 && bytes_since_limit_check >= limit_bps {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        bytes_since_limit_check = 0;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        }));
    }

    for h in handles {
        h.await??;
    }

    pb.finish_and_clear();
    Ok(())
}

async fn download_single(
    client: &reqwest::Client,
    url: &str,
    out_path: &PathBuf,
    bandwidth_limit_kbps: Option<u64>,
    resume_from: Option<u64>,
) -> Result<()> {
    let mut req = client.get(url);
    if let Some(offset) = resume_from {
        req = req.header(RANGE, format!("bytes={}-", offset));
    }
    let resp = req.send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {percent}% | {bytes}/{total_bytes} | {speed} | ETA {eta}",
        )?
        .progress_chars("█▓░ "),
    );

    let mut file = if resume_from.is_some() {
        OpenOptions::new().write(true).open(out_path)?
    } else {
        std::fs::File::create(out_path)?
    };

    if resume_from.is_some() {
        file.seek(SeekFrom::End(0))?;
    }

    let mut stream = resp.bytes_stream();
    let mut bytes_since_limit_check = 0u64;
    let limit_bytes_per_sec = bandwidth_limit_kbps.map(|k| k * 1024);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);

        if let Some(limit_bps) = limit_bytes_per_sec {
            bytes_since_limit_check += chunk.len() as u64;
            if limit_bps > 0 && bytes_since_limit_check >= limit_bps {
                tokio::time::sleep(Duration::from_secs(1)).await;
                bytes_since_limit_check = 0;
            }
        }
    }
    pb.finish_and_clear();
    Ok(())
}

fn verify_hash(path: &PathBuf) -> Result<()> {
    banner::print_info("Verifying file integrity...");
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hex::encode(hasher.finalize());
    banner::print_info(&format!("SHA256: {}", hash));
    Ok(())
}

fn organize_dir(base_dir: &PathBuf, filename: &str) -> PathBuf {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let subfolder = match ext.as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" => "videos",
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => "audio",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => "images",
        "pdf" | "doc" | "docx" | "txt" | "epub" | "mobi" => "documents",
        "zip" | "rar" | "7z" | "tar" | "gz" => "archives",
        "exe" | "msi" | "dmg" | "deb" | "rpm" => "installers",
        "torrent" => "torrents",
        _ => "other",
    };

    base_dir.join(subfolder)
}

fn filename_from_content_disposition(header: &str) -> Option<String> {
    for part in header.split(';') {
        let trimmed = part.trim();
        if trimmed.starts_with("filename=") {
            let val = trimmed.trim_start_matches("filename=").trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn filename_from_url(url: &str) -> String {
    url.split('/')
        .last()
        .filter(|s| !s.is_empty())
        .unwrap_or("download.bin")
        .split('?')
        .next()
        .unwrap_or("download.bin")
        .to_string()
}

pub fn human_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, units[unit])
}

/// Run a speed test against a URL
pub async fn speed_test(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("FetchDesk/0.2")
        .build()?;

    banner::print_info("Running speed test...");

    let start = std::time::Instant::now();
    let resp = client.get(url).send().await?.error_for_status()?;
    let mut total_bytes = 0u64;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total_bytes += chunk.len() as u64;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let speed = total_bytes as f64 / elapsed / 1024.0 / 1024.0;

    println!();
    println!(
        "  {} {:.2} MiB/s ({} in {:.2}s)",
        "Speed:".green().bold(),
        speed,
        human_size(total_bytes),
        elapsed
    );
    println!();

    Ok(())
}

#[allow(dead_code)]
pub fn require_ok(cond: bool, msg: &str) -> Result<()> {
    if cond {
        Ok(())
    } else {
        Err(anyhow!(msg.to_string()))
    }
}
