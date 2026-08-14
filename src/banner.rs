use colored::*;

pub fn print_banner() {
    let art = r#"
 ███████╗███████╗████████╗ ██████╗██╗  ██╗██████╗ ███████╗███████╗██╗  ██╗
 ██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║██╔══██╗██╔════╝██╔════╝██║ ██╔╝
 █████╗  █████╗     ██║   ██║     ███████║██║  ██║█████╗  ███████╗█████╔╝
 ██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║██║  ██║██╔══╝  ╚════██║██╔═██╗
 ██║     ███████╗   ██║   ╚██████╗██║  ██║██████╔╝███████╗███████║██║  ██╗
 ╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝
"#;
    println!("{}", art.truecolor(180, 140, 255));
    println!(
        "  {} {} {}",
        "v0.2.0".yellow().bold(),
        "•".dimmed(),
        "Developed by JOJIN JOHN".yellow().bold()
    );
    println!();
    println!(
        "  {}",
        "Terminal-based multi-connection download manager.".dimmed()
    );
    println!(
        "  {}",
        "Paste a YouTube link, magnet link, .torrent path, or direct file URL.".dimmed()
    );
    println!();
    println!(
        "  {}  {}    {}  {}    {}  {}    {}  {}    {}  {}",
        "y".cyan().bold(),
        "YouTube".dimmed(),
        "m".cyan().bold(),
        "Magnet".dimmed(),
        "d".cyan().bold(),
        "Direct".dimmed(),
        "q".cyan().bold(),
        "Quit".dimmed(),
        "?".cyan().bold(),
        "Help".dimmed()
    );
    println!();
}

pub fn print_help() {
    println!();
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  {}", "DOWNLOAD COMMANDS".yellow().bold());
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} {:<30} {}",
        "y".cyan().bold(),
        "[url]",
        "Download YouTube video"
    );
    println!(
        "  {} {:<30} {}",
        "m".cyan().bold(),
        "[url]",
        "Download magnet/torrent link"
    );
    println!(
        "  {} {:<30} {}",
        "d".cyan().bold(),
        "[url]",
        "Download direct HTTP(S) link"
    );
    println!(
        "  {} {:<30} {}",
        "<url>".cyan().bold(),
        "",
        "Auto-detect and download"
    );
    println!(
        "  {} {:<30} {}",
        "search".cyan().bold(),
        "<query>",
        "Search YouTube"
    );
    println!(
        "  {} {:<30} {}",
        "playlist".cyan().bold(),
        "<url>",
        "Download entire YouTube playlist"
    );
    println!();

    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  {}", "QUEUE MANAGEMENT".yellow().bold());
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} {:<30} {}",
        "queue".cyan().bold(),
        "",
        "Show download queue"
    );
    println!(
        "  {} {:<30} {}",
        "queue add".cyan().bold(),
        "<url>",
        "Add URL to queue"
    );
    println!(
        "  {} {:<30} {}",
        "queue start".cyan().bold(),
        "",
        "Start processing queue"
    );
    println!(
        "  {} {:<30} {}",
        "queue pause".cyan().bold(),
        "<id>",
        "Pause a download"
    );
    println!(
        "  {} {:<30} {}",
        "queue resume".cyan().bold(),
        "<id>",
        "Resume a paused download"
    );
    println!(
        "  {} {:<30} {}",
        "queue cancel".cyan().bold(),
        "<id>",
        "Cancel a download"
    );
    println!(
        "  {} {:<30} {}",
        "queue retry".cyan().bold(),
        "<id>",
        "Retry a failed download"
    );
    println!(
        "  {} {:<30} {}",
        "queue clear".cyan().bold(),
        "",
        "Clear completed items"
    );
    println!(
        "  {} {:<30} {}",
        "queue clearall".cyan().bold(),
        "",
        "Clear entire queue"
    );
    println!();

    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  {}", "SETTINGS".yellow().bold());
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} {:<30} {}",
        "quality".cyan().bold(),
        "[url]",
        "Show available qualities"
    );
    println!(
        "  {} {:<30} {}",
        "quality <preset>".cyan().bold(),
        "",
        "Set quality (4k/2k/1080/720/480/audio)"
    );
    println!(
        "  {} {:<30} {}",
        "format".cyan().bold(),
        "<fmt>",
        "Set output format (mp4/webm/mkv/avi)"
    );
    println!(
        "  {} {:<30} {}",
        "conns".cyan().bold(),
        "<n>",
        "Set parallel connections (default: 8)"
    );
    println!(
        "  {} {:<30} {}",
        "out".cyan().bold(),
        "<dir>",
        "Set output directory"
    );
    println!(
        "  {} {:<30} {}",
        "proxy".cyan().bold(),
        "<url>",
        "Set proxy (http/socks5)"
    );
    println!(
        "  {} {:<30} {}",
        "throttle".cyan().bold(),
        "<kbps>",
        "Set bandwidth limit in KB/s"
    );
    println!(
        "  {} {:<30} {}",
        "retry".cyan().bold(),
        "<n>",
        "Set retry count (default: 3)"
    );
    println!(
        "  {} {:<30} {}",
        "timeout".cyan().bold(),
        "<secs>",
        "Set timeout in seconds (default: 300)"
    );
    println!(
        "  {} {:<30} {}",
        "dry".cyan().bold(),
        "",
        "Toggle dry run mode"
    );
    println!(
        "  {} {:<30} {}",
        "verbose".cyan().bold(),
        "",
        "Toggle verbose mode"
    );
    println!(
        "  {} {:<30} {}",
        "quiet".cyan().bold(),
        "",
        "Toggle quiet mode"
    );
    println!(
        "  {} {:<30} {}",
        "subs".cyan().bold(),
        "",
        "Toggle subtitle download"
    );
    println!(
        "  {} {:<30} {}",
        "thumb".cyan().bold(),
        "",
        "Toggle thumbnail download"
    );
    println!(
        "  {} {:<30} {}",
        "organize".cyan().bold(),
        "",
        "Toggle auto-organize by file type"
    );
    println!(
        "  {} {:<30} {}",
        "verify".cyan().bold(),
        "",
        "Toggle hash verification"
    );
    println!();

    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  {}", "INFO & HISTORY".yellow().bold());
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} {:<30} {}",
        "history".cyan().bold(),
        "",
        "Show download history"
    );
    println!(
        "  {} {:<30} {}",
        "history search".cyan().bold(),
        "<query>",
        "Search download history"
    );
    println!(
        "  {} {:<30} {}",
        "history clear".cyan().bold(),
        "",
        "Clear download history"
    );
    println!(
        "  {} {:<30} {}",
        "export".cyan().bold(),
        "<format>",
        "Export history (json/csv)"
    );
    println!(
        "  {} {:<30} {}",
        "config".cyan().bold(),
        "",
        "Show current config"
    );
    println!(
        "  {} {:<30} {}",
        "config save".cyan().bold(),
        "",
        "Save current settings to config"
    );
    println!(
        "  {} {:<30} {}",
        "config reset".cyan().bold(),
        "",
        "Reset config to defaults"
    );
    println!(
        "  {} {:<30} {}",
        "info".cyan().bold(),
        "<url>",
        "Get video info without downloading"
    );
    println!(
        "  {} {:<30} {}",
        "speed".cyan().bold(),
        "[url]",
        "Run speed test"
    );
    println!(
        "  {} {:<30} {}",
        "help".cyan().bold(),
        "/ ?",
        "Show this help"
    );
    println!(
        "  {} {:<30} {}",
        "exit".cyan().bold(),
        "/ q",
        "Quit"
    );
    println!();

    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("  {}", "QUALITY PRESETS".yellow().bold());
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} {:<30} {}",
        "1".cyan().bold(),
        "4k",
        "Ultra HD (2160p)"
    );
    println!(
        "  {} {:<30} {}",
        "2".cyan().bold(),
        "2k",
        "Quad HD (1440p)"
    );
    println!(
        "  {} {:<30} {}",
        "3".cyan().bold(),
        "1080",
        "Full HD (recommended)"
    );
    println!(
        "  {} {:<30} {}",
        "4".cyan().bold(),
        "720",
        "HD"
    );
    println!(
        "  {} {:<30} {}",
        "5".cyan().bold(),
        "480",
        "SD (single file, no merge needed)"
    );
    println!(
        "  {} {:<30} {}",
        "6".cyan().bold(),
        "audio",
        "Audio only (MP3)"
    );
    println!(
        "  {} {:<30} {}",
        "7".cyan().bold(),
        "best",
        "Best available"
    );
    println!();
}

pub fn print_quality(current: &str) {
    let display = match current {
        s if s.contains("2160") => "4K (2160p)",
        s if s.contains("1440") => "2K (1440p)",
        s if s.contains("1080") => "1080p (Full HD)",
        s if s.contains("720") => "720p (HD)",
        _ if current == "18" => "480p (SD, single file)",
        s if s.contains("audio") || s.contains("bestaudio") => "Audio only",
        _ => "Best available",
    };
    println!("  {} {}", "✓".green(), format!("Quality: {}", display).cyan());
}

pub fn print_setting(name: &str, value: &str) {
    println!("  {} {}", "✓".green(), format!("{}: {}", name, value).cyan());
}

pub fn print_error(msg: &str) {
    println!("  {} {}", "✗".red().bold(), msg.red());
}

pub fn print_info(msg: &str) {
    println!("  {} {}", "→".green(), msg.dimmed());
}

pub fn print_success(msg: &str) {
    println!("  {} {}", "✓".green().bold(), msg.green());
}

pub fn print_warning(msg: &str) {
    println!("  {} {}", "⚠".yellow(), msg.yellow());
}

#[allow(dead_code)]
pub fn print_queue_item(
    id: &str,
    url: &str,
    status: &str,
    progress: f64,
) {
    let status_colored = match status {
        "Pending" => status.yellow(),
        "Downloading" => status.cyan(),
        "Paused" => status.blue(),
        "Completed" => status.green(),
        "Failed" => status.red(),
        _ => status.normal(),
    };

    let truncated_url = if url.len() > 50 {
        format!("{}...", &url[..47])
    } else {
        url.to_string()
    };

    println!(
        "  {} {} {} {:.1}% {}",
        id.cyan().bold(),
        truncated_url.dimmed(),
        status_colored,
        progress,
        "│".dimmed()
    );
}
