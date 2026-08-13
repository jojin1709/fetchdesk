use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    pub download: Download,
    pub youtube: YouTube,
    pub torrent: Torrent,
    pub network: Network,
    pub ui: UI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    pub output_dir: String,
    pub log_file: Option<String>,
    pub verbose: bool,
    pub quiet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub max_connections: usize,
    pub max_parallel_downloads: usize,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub bandwidth_limit_kbps: Option<u64>,
    pub timeout_secs: u64,
    pub auto_resume: bool,
    pub auto_organize: bool,
    pub auto_extract: bool,
    pub hash_verify: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTube {
    pub quality: String,
    pub format: String,
    pub write_subtitles: bool,
    pub write_thumbnail: bool,
    pub write_info_json: bool,
    pub subtitle_langs: Vec<String>,
    pub playlist_start: Option<usize>,
    pub playlist_end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub seed_time_mins: u64,
    pub seed_ratio: f64,
    pub upload_limit_kbps: Option<u64>,
    pub max_connections_per_server: usize,
    pub trackers: Vec<String>,
    pub enable_dht: bool,
    pub enable_lpd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub proxy: Option<String>,
    pub user_agent: String,
    pub cookies_file: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UI {
    pub color_theme: String,
    pub show_progress: bool,
    pub notify_on_complete: bool,
}

impl Default for Config {
    fn default() -> Self {
        let output_dir = dirs_download_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "./downloads".to_string());

        Config {
            general: General {
                output_dir,
                log_file: None,
                verbose: false,
                quiet: false,
            },
            download: Download {
                max_connections: 16,
                max_parallel_downloads: 3,
                retry_count: 3,
                retry_delay_ms: 1000,
                bandwidth_limit_kbps: None,
                timeout_secs: 300,
                auto_resume: true,
                auto_organize: true,
                auto_extract: false,
                hash_verify: true,
                dry_run: false,
            },
            youtube: YouTube {
                quality: "best".to_string(),
                format: "mp4".to_string(),
                write_subtitles: false,
                write_thumbnail: false,
                write_info_json: false,
                subtitle_langs: vec!["en".to_string()],
                playlist_start: None,
                playlist_end: None,
            },
            torrent: Torrent {
                seed_time_mins: 0,
                seed_ratio: 0.0,
                upload_limit_kbps: None,
                max_connections_per_server: 16,
                trackers: Vec::new(),
                enable_dht: true,
                enable_lpd: true,
            },
            network: Network {
                proxy: None,
                user_agent: "FetchDesk/0.2".to_string(),
                cookies_file: None,
                headers: std::collections::HashMap::new(),
            },
            ui: UI {
                color_theme: "default".to_string(),
                show_progress: true,
                notify_on_complete: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = config_path();
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("  Warning: Failed to parse config: {}", e);
                        eprintln!("  Using default config");
                    }
                },
                Err(e) => {
                    eprintln!("  Warning: Failed to read config: {}", e);
                    eprintln!("  Using default config");
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<()> {
        let config_path = config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

fn base_app_dir() -> PathBuf {
    let home_opt = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    if let Some(home) = home_opt {
        if cfg!(windows) {
            PathBuf::from(&home).join("AppData").join("Local").join("fetchdesk")
        } else {
            PathBuf::from(&home).join(".config").join("fetchdesk")
        }
    } else {
        PathBuf::from(".")
    }
}

pub fn config_path() -> PathBuf {
    base_app_dir().join("config.toml")
}

pub fn history_path() -> PathBuf {
    base_app_dir().join("history.json")
}

pub fn queue_path() -> PathBuf {
    base_app_dir().join("queue.json")
}

fn dirs_download_dir() -> Option<PathBuf> {
    let home_opt = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    if let Some(home) = home_opt {
        let downloads = PathBuf::from(&home).join("Downloads");
        if downloads.is_dir() {
            return Some(downloads);
        }
    }
    None
}
