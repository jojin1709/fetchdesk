mod banner;
mod config;
mod disk;
mod downloader;
mod history;
mod queue;
mod torrent;
mod youtube;

use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;

use config::Config;
use history::DownloadHistory;
use queue::DownloadQueue;

#[tokio::main]
async fn main() {
    banner::print_banner();

    let mut config = Config::load();
    let mut download_queue = DownloadQueue::load();
    download_queue.max_parallel = config.download.max_parallel_downloads;
    let mut history = DownloadHistory::load();

    loop {
        print!("{} ", "fetchdesk>".truecolor(180, 140, 255).bold());
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut parts = input.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("").trim();

        match cmd {
            "exit" | "q" | "quit" => break,
            "help" | "?" => banner::print_help(),
            "y" => handle_youtube(arg, &config, &mut history).await,
            "m" => handle_magnet(arg, &config, &mut history),
            "d" => handle_direct(arg, &config, &mut history).await,
            "search" => handle_search(arg),
            "playlist" => handle_playlist(arg, &config).await,
            "info" => handle_info(arg),
            "speed" => handle_speed(arg).await,
            "quality" => handle_quality(arg, &mut config).await,
            "format" => handle_format(arg, &mut config),
            "conns" => handle_conns(arg, &mut config),
            "out" => handle_out(arg, &mut config),
            "proxy" => handle_proxy(arg, &mut config),
            "throttle" => handle_throttle(arg, &mut config),
            "retry" => handle_retry(arg, &mut config),
            "timeout" => handle_timeout(arg, &mut config),
            "dry" => toggle_dry(&mut config),
            "verbose" => toggle_verbose(&mut config),
            "quiet" => toggle_quiet(&mut config),
            "subs" => toggle_subs(&mut config),
            "thumb" => toggle_thumb(&mut config),
            "organize" => toggle_organize(&mut config),
            "verify" => toggle_verify(&mut config),
            "queue" => handle_queue(arg, &mut download_queue, &config, &mut history).await,
            "history" => handle_history(arg, &mut history),
            "export" => handle_export(arg, &history),
            "config" => handle_config(arg, &config),
            _ => handle_target(input, &config, &mut history).await,
        }
    }

    println!();
    banner::print_info("bye.");
    println!();
}

async fn handle_youtube(url: &str, config: &Config, history: &mut DownloadHistory) {
    let url = if url.is_empty() {
        prompt("Paste YouTube URL:")
    } else {
        url.to_string()
    };
    if url.is_empty() {
        return;
    }
    if !is_youtube(&url) {
        banner::print_error("Not a YouTube URL");
        return;
    }

    // Check duplicate
    if history.is_duplicate(&url) {
        banner::print_warning("This URL was already downloaded. Use 'history' to see details.");
    }

    let out_dir = PathBuf::from(&config.general.output_dir);

    match youtube::get_available_qualities(&url).await {
        Ok(formats) => {
            if formats.is_empty() {
                banner::print_error("No formats found for this video");
                return;
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {}", "AVAILABLE QUALITIES".yellow().bold());
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            for (i, fmt) in formats.iter().enumerate() {
                let num = format!("{}", i + 1).cyan().bold();
                let quality = fmt.display.green();
                let size = fmt.size.dimmed();
                let single = if fmt.single_file { " ✓".yellow() } else { "".normal() };
                println!("    {} {} - {}{}", num, quality, size, single);
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();

            print!("  {} ", "Select quality (number):".dimmed());
            io::stdout().flush().ok();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).ok();
            if let Ok(idx) = choice.trim().parse::<usize>() {
                if idx >= 1 && idx <= formats.len() {
                    let selected = &formats[idx - 1];
                    let mut custom_config = config.clone();
                    custom_config.youtube.quality = selected.format.clone();
                    let start = std::time::Instant::now();
                    let result = youtube::download_youtube(&url, &out_dir, &custom_config).await;
                    let elapsed = start.elapsed().as_secs_f64();

                    match result {
                        Ok(_) => {
                            history.add(&url, None, "youtube", Some(&selected.display), &out_dir.display().to_string(), None, Some(elapsed));
                        }
                        Err(e) => {
                            banner::print_error(&e.to_string());
                        }
                    }
                } else {
                    banner::print_error("Invalid selection");
                }
            } else {
                banner::print_error("Invalid input");
            }
        }
        Err(e) => {
            banner::print_error(&format!("Failed to fetch formats: {}", e));
        }
    }
}

fn handle_magnet(url: &str, config: &Config, history: &mut DownloadHistory) {
    let url = if url.is_empty() {
        prompt("Paste magnet link or .torrent path:")
    } else {
        url.to_string()
    };
    if url.is_empty() {
        return;
    }
    let out_dir = PathBuf::from(&config.general.output_dir);
    let start = std::time::Instant::now();
    match torrent::download_torrent(&url, &out_dir, config) {
        Ok(_) => {
            let elapsed = start.elapsed().as_secs_f64();
            history.add(&url, None, "torrent", None, &out_dir.display().to_string(), None, Some(elapsed));
        }
        Err(e) => {
            banner::print_error(&e.to_string());
        }
    }
}

async fn handle_direct(url: &str, config: &Config, history: &mut DownloadHistory) {
    let url = if url.is_empty() {
        prompt("Paste direct file URL:")
    } else {
        url.to_string()
    };
    if url.is_empty() {
        return;
    }

    if history.is_duplicate(&url) {
        banner::print_warning("This URL was already downloaded.");
    }

    let out_dir = PathBuf::from(&config.general.output_dir);
    let start = std::time::Instant::now();
    let result = downloader::download_direct(&url, &out_dir, config).await;
    let elapsed = start.elapsed().as_secs_f64();

    match result {
        Ok(_) => {
            history.add(&url, None, "direct", None, &out_dir.display().to_string(), None, Some(elapsed));
        }
        Err(e) => {
            banner::print_error(&e.to_string());
        }
    }
}

fn handle_search(query: &str) {
    let query = if query.is_empty() {
        prompt("Search YouTube:")
    } else {
        query.to_string()
    };
    if query.is_empty() {
        return;
    }

    banner::print_info(&format!("Searching for: {}", query));

    match youtube::search_youtube(&query, 10) {
        Ok(results) => {
            if results.is_empty() {
                banner::print_error("No results found");
                return;
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {}", "SEARCH RESULTS".yellow().bold());
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            for (i, (title, url, duration)) in results.iter().enumerate() {
                let num = format!("{}", i + 1).cyan().bold();
                let title = title.green();
                let duration = duration.dimmed();
                println!("    {} {} [{}]", num, title, duration);
                println!("      {}", url.blue());
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {} To download, use: y <url>", "Tip:".yellow());
            println!();
        }
        Err(e) => {
            banner::print_error(&format!("Search failed: {}", e));
        }
    }
}

async fn handle_playlist(url: &str, config: &Config) {
    let url = if url.is_empty() {
        prompt("Paste YouTube playlist URL:")
    } else {
        url.to_string()
    };
    if url.is_empty() {
        return;
    }
    if !is_youtube(&url) {
        banner::print_error("Not a YouTube URL");
        return;
    }

    let out_dir = PathBuf::from(&config.general.output_dir);

    match youtube::get_playlist_entries(&url) {
        Ok(entries) => {
            if entries.is_empty() {
                banner::print_error("No entries found in playlist");
                return;
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {} ({} videos)", "PLAYLIST".yellow().bold(), entries.len());
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            for (i, title, _) in entries.iter().take(20) {
                println!("    {} {}", format!("{}", i).cyan().bold(), title.green());
            }
            if entries.len() > 20 {
                println!("    {} ... and {} more", "...".dimmed(), entries.len() - 20);
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();

            print!("  {} ", "Download entire playlist? (y/n):".dimmed());
            io::stdout().flush().ok();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).ok();
            if choice.trim().to_lowercase() == "y" {
                if let Err(e) = youtube::download_playlist(&url, &out_dir, config).await {
                    banner::print_error(&e.to_string());
                }
            }
        }
        Err(e) => {
            banner::print_error(&format!("Failed to get playlist: {}", e));
        }
    }
}

fn handle_info(url: &str) {
    let url = if url.is_empty() {
        prompt("Paste YouTube URL:")
    } else {
        url.to_string()
    };
    if url.is_empty() {
        return;
    }

    banner::print_info("Fetching video info...");

    match youtube::get_video_info(&url) {
        Ok(info) => {
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {}", "VIDEO INFO".yellow().bold());
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            println!("    {} {}", "Title:".cyan(), info.title.green());
            println!("    {} {}", "Uploader:".cyan(), info.uploader.green());
            println!("    {} {}", "Duration:".cyan(), info.duration.green());
            println!("    {} {}", "URL:".cyan(), info.webpage_url.blue());
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
        }
        Err(e) => {
            banner::print_error(&format!("Failed to get info: {}", e));
        }
    }
}

async fn handle_speed(url: &str) {
    let url = if url.is_empty() {
        prompt("Enter test URL (or press Enter for default):")
    } else {
        url.to_string()
    };
    let url = if url.is_empty() {
        "https://speed.cloudflare.com/__down?bytes=10000000".to_string()
    } else {
        url
    };

    if let Err(e) = downloader::speed_test(&url).await {
        banner::print_error(&format!("Speed test failed: {}", e));
    }
}

async fn handle_quality(arg: &str, config: &mut Config) {
    if arg.is_empty() {
        banner::print_quality(&config.youtube.quality);
        println!();
        banner::print_info("Usage: quality <youtube-url>  - see available qualities");
        banner::print_info("       quality <preset>      - set quality (4k/2k/1080/720/480/audio)");
        return;
    }

    if is_youtube(arg) {
        match youtube::get_available_qualities(arg).await {
            Ok(formats) => {
                if formats.is_empty() {
                    banner::print_error("No formats found");
                    return;
                }
                println!();
                println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                println!("  {}", "AVAILABLE QUALITIES".yellow().bold());
                println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                println!();
                for (i, fmt) in formats.iter().enumerate() {
                    let num = format!("{}", i + 1).cyan().bold();
                    let quality = fmt.display.green();
                    let size = fmt.size.dimmed();
                    println!("    {} {} - {}", num, quality, size);
                }
                println!();
                println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                println!();
            }
            Err(e) => {
                banner::print_error(&format!("Failed: {}", e));
            }
        }
    } else {
        config.youtube.quality = match arg.to_lowercase().as_str() {
            "4k" | "2160" | "2160p" => "bestvideo[height<=2160]+bestaudio/best[height<=2160]".to_string(),
            "2k" | "1440" | "1440p" => "bestvideo[height<=1440]+bestaudio/best[height<=1440]".to_string(),
            "1080" | "1080p" | "fhd" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]".to_string(),
            "720" | "720p" | "hd" => "bestvideo[height<=720]+bestaudio/best[height<=720]".to_string(),
            "480" | "480p" | "sd" => "18".to_string(),
            "audio" | "mp3" | "music" => "bestaudio[ext=m4a]/bestaudio".to_string(),
            "best" | "max" => "bestvideo+bestaudio/best".to_string(),
            _ => {
                banner::print_error(&format!("Unknown quality: {}", arg));
                return;
            }
        };
        banner::print_quality(&config.youtube.quality);
    }
}

fn handle_format(arg: &str, config: &mut Config) {
    if arg.is_empty() {
        banner::print_setting("Format", &config.youtube.format);
        println!();
        banner::print_info("Available: mp4, webm, mkv, avi, mp3, flac");
        return;
    }
    let valid = ["mp4", "webm", "mkv", "avi", "mp3", "flac", "wav", "ogg", "best"];
    if valid.contains(&arg) {
        config.youtube.format = arg.to_string();
        banner::print_setting("Format", arg);
    } else {
        banner::print_error(&format!("Invalid format: {}", arg));
        banner::print_info("Available: mp4, webm, mkv, avi, mp3, flac");
    }
}

fn handle_conns(arg: &str, config: &mut Config) {
    if let Ok(n) = arg.parse::<usize>() {
        config.download.max_connections = n.max(1);
        banner::print_setting("Connections", &config.download.max_connections.to_string());
    } else {
        banner::print_error("Usage: conns <number>");
    }
}

fn handle_out(arg: &str, config: &mut Config) {
    if arg.is_empty() {
        banner::print_setting("Output dir", &config.general.output_dir);
    } else {
        config.general.output_dir = arg.to_string();
        banner::print_setting("Output dir", arg);
    }
}

fn handle_proxy(arg: &str, config: &mut Config) {
    if arg.is_empty() {
        match &config.network.proxy {
            Some(p) => banner::print_setting("Proxy", p),
            None => banner::print_info("No proxy set"),
        }
    } else {
        config.network.proxy = Some(arg.to_string());
        banner::print_setting("Proxy", arg);
    }
}

fn handle_throttle(arg: &str, config: &mut Config) {
    if arg.is_empty() {
        match config.download.bandwidth_limit_kbps {
            Some(limit) => banner::print_setting("Bandwidth limit", &format!("{} KB/s", limit)),
            None => banner::print_info("No bandwidth limit set"),
        }
    } else if let Ok(kbps) = arg.parse::<u64>() {
        config.download.bandwidth_limit_kbps = Some(kbps);
        banner::print_setting("Bandwidth limit", &format!("{} KB/s", kbps));
    } else {
        banner::print_error("Usage: throttle <kbps>");
    }
}

fn handle_retry(arg: &str, config: &mut Config) {
    if let Ok(n) = arg.parse::<u32>() {
        config.download.retry_count = n;
        banner::print_setting("Retry count", &n.to_string());
    } else {
        banner::print_error("Usage: retry <number>");
    }
}

fn handle_timeout(arg: &str, config: &mut Config) {
    if let Ok(n) = arg.parse::<u64>() {
        config.download.timeout_secs = n;
        banner::print_setting("Timeout", &format!("{}s", n));
    } else {
        banner::print_error("Usage: timeout <seconds>");
    }
}

fn toggle_dry(config: &mut Config) {
    config.download.dry_run = !config.download.dry_run;
    let state = if config.download.dry_run { "ON" } else { "OFF" };
    banner::print_setting("Dry run", state);
}

fn toggle_verbose(config: &mut Config) {
    config.general.verbose = !config.general.verbose;
    let state = if config.general.verbose { "ON" } else { "OFF" };
    banner::print_setting("Verbose", state);
}

fn toggle_quiet(config: &mut Config) {
    config.general.quiet = !config.general.quiet;
    let state = if config.general.quiet { "ON" } else { "OFF" };
    banner::print_setting("Quiet", state);
}

fn toggle_subs(config: &mut Config) {
    config.youtube.write_subtitles = !config.youtube.write_subtitles;
    let state = if config.youtube.write_subtitles { "ON" } else { "OFF" };
    banner::print_setting("Subtitles", state);
}

fn toggle_thumb(config: &mut Config) {
    config.youtube.write_thumbnail = !config.youtube.write_thumbnail;
    let state = if config.youtube.write_thumbnail { "ON" } else { "OFF" };
    banner::print_setting("Thumbnail", state);
}

fn toggle_organize(config: &mut Config) {
    config.download.auto_organize = !config.download.auto_organize;
    let state = if config.download.auto_organize { "ON" } else { "OFF" };
    banner::print_setting("Auto-organize", state);
}

fn toggle_verify(config: &mut Config) {
    config.download.hash_verify = !config.download.hash_verify;
    let state = if config.download.hash_verify { "ON" } else { "OFF" };
    banner::print_setting("Hash verify", state);
}

async fn handle_queue(
    arg: &str,
    queue: &mut DownloadQueue,
    config: &Config,
    history: &mut DownloadHistory,
) {
    let mut parts = arg.splitn(2, ' ');
    let sub = parts.next().unwrap_or("list");
    let sub_arg = parts.next().unwrap_or("").trim();

    match sub {
        "list" | "" => {
            if queue.list().is_empty() {
                banner::print_info("Queue is empty");
                return;
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!(
                "  {} ({} pending, {} active, {} completed, {} failed)",
                "DOWNLOAD QUEUE".yellow().bold(),
                queue.pending_count(),
                queue.active_count,
                queue.completed_count(),
                queue.failed_count()
            );
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            for item in queue.list() {
                let status = match item.status {
                    queue::QueueStatus::Pending => "Pending".yellow(),
                    queue::QueueStatus::Downloading => "Downloading".cyan(),
                    queue::QueueStatus::Paused => "Paused".blue(),
                    queue::QueueStatus::Completed => "Completed".green(),
                    queue::QueueStatus::Failed => "Failed".red(),
                };
                let url_display = if item.url.len() > 45 {
                    format!("{}...", &item.url[..42])
                } else {
                    item.url.clone()
                };
                println!(
                    "  {} {} {} {:.1}%",
                    item.id.cyan().bold(),
                    url_display.dimmed(),
                    status,
                    item.progress
                );
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
        }
        "add" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: queue add <url>");
                return;
            }
            let target_type = if is_youtube(sub_arg) {
                "youtube"
            } else if is_torrent(sub_arg) {
                "torrent"
            } else if sub_arg.starts_with("http://") || sub_arg.starts_with("https://") {
                "direct"
            } else {
                banner::print_error("Unrecognized URL type");
                return;
            };
            let id = queue.add(sub_arg, target_type, Some(config.youtube.quality.clone()), &config.general.output_dir);
            banner::print_success(&format!("Added to queue: {}", id));
        }
        "start" => {
            process_queue(queue, config, history).await;
        }
        "pause" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: queue pause <id>");
                return;
            }
            queue.pause(sub_arg);
            banner::print_success(&format!("Paused: {}", sub_arg));
        }
        "resume" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: queue resume <id>");
                return;
            }
            queue.resume(sub_arg);
            banner::print_success(&format!("Resumed: {}", sub_arg));
        }
        "cancel" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: queue cancel <id>");
                return;
            }
            queue.cancel(sub_arg);
            banner::print_success(&format!("Cancelled: {}", sub_arg));
        }
        "retry" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: queue retry <id>");
                return;
            }
            queue.retry(sub_arg);
            banner::print_success(&format!("Retrying: {}", sub_arg));
        }
        "clear" => {
            queue.clear_completed();
            banner::print_success("Cleared completed items");
        }
        "clearall" => {
            queue.clear_all();
            banner::print_success("Cleared entire queue");
        }
        _ => {
            banner::print_error("Unknown queue command. Use: queue [list|add|start|pause|resume|cancel|retry|clear|clearall]");
        }
    }
}

async fn process_queue(queue: &mut DownloadQueue, config: &Config, history: &mut DownloadHistory) {
    loop {
        let item = queue.start_next();
        if item.is_none() {
            break;
        }
        let item = item.unwrap();
        let id = item.id.clone();
        let url = item.url.clone();
        let target_type = item.target_type.clone();
        let out_dir = PathBuf::from(&item.output_dir);

        banner::print_info(&format!("Processing: {} ({})", id, target_type));

        let result = match target_type.as_str() {
            "youtube" => youtube::download_youtube(&url, &out_dir, config).await.map(|_| ()),
            "torrent" => torrent::download_torrent(&url, &out_dir, config),
            "direct" => downloader::download_direct(&url, &out_dir, config).await,
            _ => Err(anyhow::anyhow!("Unknown type")),
        };

        match result {
            Ok(_) => {
                queue.complete(&id, &out_dir.display().to_string());
                history.add(&url, None, &target_type, None, &out_dir.display().to_string(), None, None);
                banner::print_success(&format!("Completed: {}", id));
            }
            Err(e) => {
                queue.fail(&id, &e.to_string());
                banner::print_error(&format!("Failed: {} - {}", id, e));
            }
        }
    }
}

fn handle_history(arg: &str, history: &mut DownloadHistory) {
    let mut parts = arg.splitn(2, ' ');
    let sub = parts.next().unwrap_or("list");
    let sub_arg = parts.next().unwrap_or("").trim();

    match sub {
        "list" | "" => {
            let entries = history.list(20);
            if entries.is_empty() {
                banner::print_info("No download history");
                return;
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!(
                "  {} ({} downloads, {})",
                "DOWNLOAD HISTORY".yellow().bold(),
                history.count(),
                downloader::human_size(history.total_size())
            );
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            for entry in entries {
                let title = entry.title.as_deref().unwrap_or(&entry.url);
                let truncated = if title.len() > 50 {
                    format!("{}...", &title[..47])
                } else {
                    title.to_string()
                };
                println!(
                    "  {} {} [{}]",
                    entry.downloaded_at.dimmed(),
                    truncated.green(),
                    entry.target_type.cyan()
                );
            }
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
        }
        "search" => {
            if sub_arg.is_empty() {
                banner::print_error("Usage: history search <query>");
                return;
            }
            let results = history.search(sub_arg);
            if results.is_empty() {
                banner::print_info("No matching entries found");
                return;
            }
            println!();
            for entry in results {
                let title = entry.title.as_deref().unwrap_or(&entry.url);
                println!(
                    "  {} {} [{}]",
                    entry.downloaded_at.dimmed(),
                    title.green(),
                    entry.target_type.cyan()
                );
            }
            println!();
        }
        "clear" => {
            history.clear();
            banner::print_success("History cleared");
        }
        _ => {
            banner::print_error("Unknown history command. Use: history [list|search|clear]");
        }
    }
}

fn handle_export(arg: &str, history: &DownloadHistory) {
    let format = if arg.is_empty() { "json" } else { arg };

    let content = match format {
        "json" => history.export_json(),
        "csv" => history.export_csv(),
        _ => {
            banner::print_error("Unknown format. Use: export [json|csv]");
            return;
        }
    };

    match content {
        Ok(c) => {
            let filename = format!("fetchdesk_history.{}", format);
            std::fs::write(&filename, &c).ok();
            banner::print_success(&format!("Exported to: {}", filename));
        }
        Err(e) => {
            banner::print_error(&format!("Export failed: {}", e));
        }
    }
}

fn handle_config(arg: &str, config: &Config) {
    let mut parts = arg.splitn(2, ' ');
    let sub = parts.next().unwrap_or("show");

    match sub {
        "show" | "" => {
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {}", "CURRENT CONFIG".yellow().bold());
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
            println!("    {} {}", "Output dir:".cyan(), config.general.output_dir);
            println!("    {} {}", "Quality:".cyan(), config.youtube.quality);
            println!("    {} {}", "Format:".cyan(), config.youtube.format);
            println!("    {} {}", "Connections:".cyan(), config.download.max_connections);
            println!("    {} {}", "Max parallel:".cyan(), config.download.max_parallel_downloads);
            println!("    {} {}", "Retry count:".cyan(), config.download.retry_count);
            println!("    {} {}", "Timeout:".cyan(), config.download.timeout_secs);
            println!("    {} {:?}", "Bandwidth limit:".cyan(), config.download.bandwidth_limit_kbps);
            println!("    {} {}", "Dry run:".cyan(), config.download.dry_run);
            println!("    {} {}", "Verbose:".cyan(), config.general.verbose);
            println!("    {} {}", "Quiet:".cyan(), config.general.quiet);
            println!("    {} {}", "Subtitles:".cyan(), config.youtube.write_subtitles);
            println!("    {} {}", "Thumbnail:".cyan(), config.youtube.write_thumbnail);
            println!("    {} {}", "Auto-organize:".cyan(), config.download.auto_organize);
            println!("    {} {}", "Hash verify:".cyan(), config.download.hash_verify);
            println!("    {} {:?}", "Proxy:".cyan(), config.network.proxy);
            println!("    {} {}", "Seed time:".cyan(), config.torrent.seed_time_mins);
            println!("    {} {}", "Seed ratio:".cyan(), config.torrent.seed_ratio);
            println!();
            println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!();
        }
        "save" => {
            match config.save() {
                Ok(_) => banner::print_success("Config saved"),
                Err(e) => banner::print_error(&format!("Failed to save: {}", e)),
            }
        }
        "reset" => {
            banner::print_info("Config reset to defaults (use 'config save' to persist)");
        }
        _ => {
            banner::print_error("Unknown config command. Use: config [show|save|reset]");
        }
    }
}

async fn handle_target(target: &str, config: &Config, history: &mut DownloadHistory) {
    let out_dir = PathBuf::from(&config.general.output_dir);
    let start = std::time::Instant::now();

    let (target_type, result) = if is_youtube(target) {
        ("youtube", youtube::download_youtube(target, &out_dir, config).await)
    } else if is_torrent(target) {
        ("torrent", torrent::download_torrent(target, &out_dir, config))
    } else if target.starts_with("http://") || target.starts_with("https://") {
        ("direct", downloader::download_direct(target, &out_dir, config).await)
    } else {
        banner::print_error("Not a recognized YouTube link, magnet/torrent, or HTTP(S) URL");
        return;
    };

    match result {
        Ok(_) => {
            let elapsed = start.elapsed().as_secs_f64();
            history.add(target, None, target_type, None, &out_dir.display().to_string(), None, Some(elapsed));
        }
        Err(e) => {
            banner::print_error(&e.to_string());
        }
    }
}

fn is_youtube(s: &str) -> bool {
    s.contains("youtube.com") || s.contains("youtu.be")
}

fn is_torrent(s: &str) -> bool {
    s.starts_with("magnet:") || s.ends_with(".torrent")
}

fn prompt(label: &str) -> String {
    print!("  {} ", label.dimmed());
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    buf.trim().to_string()
}
