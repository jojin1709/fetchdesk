use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::banner;
use crate::config::Config;
use crate::youtube::which;

fn resolve_aria2c() -> Result<PathBuf> {
    if let Some(p) = which("aria2c") {
        return Ok(p);
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let fetch_bin = PathBuf::from(local_app_data).join("FetchDesk").join("bin").join("aria2c.exe");
        if fetch_bin.exists() {
            return Ok(fetch_bin);
        }
    }
    if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(PathBuf::from)) {
        let py_script = home.join("AppData").join("Roaming").join("Python").join("Python314").join("Scripts").join("aria2c.exe");
        if py_script.exists() {
            return Ok(py_script);
        }
    }
    Err(anyhow!("aria2c not found. Please run the install.ps1 setup script."))
}

/// Download torrent/magnet with full options and sleek animated progress bar
pub fn download_torrent(
    target: &str,
    out_dir: &PathBuf,
    config: &Config,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    silent: bool,
) -> Result<()> {
    let aria2c_path = resolve_aria2c()?;

    std::fs::create_dir_all(out_dir)?;

    let mut cmd = Command::new(&aria2c_path);
    cmd.arg(target)
        .arg(format!("--dir={}", out_dir.display()))
        .arg("--follow-torrent=mem")
        .arg("--follow-metalink=mem")
        .arg(format!("--seed-time={}", config.torrent.seed_time_mins))
        .arg(format!("--seed-ratio={}", config.torrent.seed_ratio))
        .arg(format!("--max-connection-per-server={}", config.torrent.max_connections_per_server))
        .arg(format!("--split={}", config.torrent.max_connections_per_server))
        .arg("--summary-interval=1")
        .arg(format!("--bt-enable-lpd={}", config.torrent.enable_lpd))
        .arg(format!("--enable-dht={}", config.torrent.enable_dht))
        .arg("--enable-peer-exchange=true")
        .arg("--peer-id-prefix=-TR3000-")
        .arg("--peer-agent=Transmission/3.00")
        .arg("--bt-min-crypto-level=plain")
        .arg("--bt-require-crypto=false")
        .arg("--max-overall-upload-limit=50K")
        .arg("--bt-max-peers=256")
        .arg("--bt-tracker-connect-timeout=5")
        .arg("--bt-tracker-timeout=5")
        .arg("--bt-stop-timeout=600")
        .arg("--file-allocation=none")
        .arg("--console-log-level=warn");

    // Upload speed limit
    if let Some(limit) = config.torrent.upload_limit_kbps {
        cmd.arg(format!("--max-upload-limit={}", limit * 1024));
    }

    // High performance default trackers
    let default_trackers = [
        "udp://tracker.opentrackr.org:1337/announce",
        "udp://open.stealth.si:80/announce",
        "udp://tracker.torrent.eu.org:451/announce",
        "udp://open.demonii.com:1337/announce",
        "udp://tracker.dler.org:6969/announce",
        "udp://tracker.opentorrent.top:6969/announce",
    ];
    for tr in &default_trackers {
        cmd.arg(format!("--bt-tracker={}", tr));
    }

    // Custom trackers from config
    for tracker in &config.torrent.trackers {
        cmd.arg(format!("--bt-tracker={}", tracker));
    }

    // Dry run
    if config.download.dry_run {
        if !silent {
            banner::print_info("DRY RUN - would download torrent:");
            banner::print_info(&format!("Target: {}", target));
            banner::print_info(&format!("Output: {}", out_dir.display()));
        }
        return Ok(());
    }

    if !silent {
        banner::print_info("Connecting to BitTorrent network...");
    }

    run_aria2c_with_progress(cmd, progress_callback, silent)?;

    if !silent {
        banner::print_success(&format!("Saved to: {}", out_dir.display()));
    }
    Ok(())
}

fn run_aria2c_with_progress(
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
            if line.contains("[METADATA]") || line.contains("METADATA") {
                if !silent {
                    pb_clone.set_message("Resolving torrent metadata & searching peers...".to_string());
                }
            } else if line.contains('[') && line.contains(']') && line.contains('%') {
                if let Some(pct_start) = line.find('(') {
                    if let Some(pct_end) = line[pct_start..].find("%)") {
                        let pct_str = &line[pct_start + 1..pct_start + pct_end];
                        if let Ok(pct) = pct_str.parse::<f64>() {
                            pb_clone.set_position((pct * 10.0) as u64);
                            if let Some(ref cb) = progress_cb {
                                cb(pct);
                            }
                        }
                    }
                }
                let clean_line = line.trim();
                if !silent {
                    if let Some(start_idx) = clean_line.find('[') {
                        if let Some(end_idx) = clean_line.rfind(']') {
                            let msg = &clean_line[start_idx + 1..end_idx];
                            pb_clone.set_message(msg.to_string());
                        }
                    }
                }
            } else if line.contains("DL:") || line.contains("CN:") {
                if !silent {
                    pb_clone.set_message(line.trim().to_string());
                }
            }
        }
    });

    let err_msgs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let err_msgs_clone = std::sync::Arc::clone(&err_msgs);
    let stderr_handle = std::thread::spawn(move || {
        let err_reader = std::io::BufReader::new(stderr);
        for line in err_reader.lines().flatten() {
            if (line.contains("ERROR") || line.contains("Exception")) && !line.contains("dht.dat") {
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
        return Err(anyhow!("Torrent download ended"));
    }

    Ok(())
}

/// List files in a torrent
#[allow(dead_code)]
pub fn list_torrent_files(target: &str) -> Result<Vec<(usize, String, String)>> {
    let aria2c_path = which("aria2c").ok_or_else(|| anyhow!("aria2c not found"))?;

    let output = Command::new(&aria2c_path)
        .arg(target)
        .arg("--show-files")
        .arg("--dry-run")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        if line.contains("#") && line.contains("|") {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let num_str = parts[0].trim().split('#').last().unwrap_or("0");
                if let Ok(num) = num_str.parse::<usize>() {
                    let path = parts[1].trim();
                    let size = parts.get(2).map(|s| s.trim()).unwrap_or("?");
                    files.push((num, path.to_string(), size.to_string()));
                }
            }
        }
    }

    Ok(files)
}

/// Get torrent info
#[allow(dead_code)]
pub fn get_torrent_info(target: &str) -> Result<(String, usize, String)> {
    let aria2c_path = which("aria2c").ok_or_else(|| anyhow!("aria2c not found"))?;

    let output = Command::new(&aria2c_path)
        .arg(target)
        .arg("--show-files")
        .arg("--dry-run")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut name = "Unknown".to_string();
    let mut file_count = 0;
    let mut total_size = "Unknown".to_string();

    for line in stdout.lines() {
        if line.contains("DL: ") || line.contains("Size: ") {
            if let Some(idx) = line.find("Size: ") {
                total_size = line[idx + 6..].trim().to_string();
            }
        }
        if line.contains("#") {
            file_count += 1;
        }
    }

    // Try to extract name from output
    for line in stdout.lines() {
        if line.contains("BITFIELD") || line.contains("index") {
            // Skip
        } else if !line.starts_with(' ') && !line.is_empty() && !line.contains('#') {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.len() > 2 {
                name = trimmed.to_string();
                break;
            }
        }
    }

    Ok((name, file_count, total_size))
}
