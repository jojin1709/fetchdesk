use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub target_type: String,
    pub quality: Option<String>,
    pub file_path: String,
    pub file_size: Option<u64>,
    pub downloaded_at: String,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistory {
    pub entries: Vec<HistoryEntry>,
}

impl DownloadHistory {
    pub fn load() -> Self {
        let path = config::history_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => DownloadHistory::new(),
            }
        } else {
            DownloadHistory::new()
        }
    }

    pub fn new() -> Self {
        DownloadHistory {
            entries: Vec::new(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config::history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(
        &mut self,
        url: &str,
        title: Option<&str>,
        target_type: &str,
        quality: Option<&str>,
        file_path: &str,
        file_size: Option<u64>,
        duration_secs: Option<f64>,
    ) {
        let id = format!(
            "{}",
            chrono::Utc::now().timestamp_millis()
        );
        let entry = HistoryEntry {
            id,
            url: url.to_string(),
            title: title.map(|s| s.to_string()),
            target_type: target_type.to_string(),
            quality: quality.map(|s| s.to_string()),
            file_path: file_path.to_string(),
            file_size,
            downloaded_at: Utc::now().to_rfc3339(),
            duration_secs,
        };
        self.entries.push(entry);
        let _ = self.save();
    }

    pub fn is_duplicate(&self, url: &str) -> bool {
        self.entries.iter().any(|e| e.url == url)
    }

    pub fn list(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&q)
                    || e.title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || e.file_path.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        let _ = self.save();
    }

    pub fn total_size(&self) -> u64 {
        self.entries.iter().filter_map(|e| e.file_size).sum()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn export_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn export_csv(&self) -> Result<String> {
        let mut csv = String::from("id,url,title,type,quality,file_path,size,downloaded_at\n");
        for e in &self.entries {
            let escape = |s: &str| -> String {
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.to_string()
                }
            };
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                escape(&e.id),
                escape(&e.url),
                escape(e.title.as_deref().unwrap_or("")),
                escape(&e.target_type),
                escape(e.quality.as_deref().unwrap_or("")),
                escape(&e.file_path),
                e.file_size.map(|s| s.to_string()).unwrap_or_default(),
                escape(&e.downloaded_at),
            ));
        }
        Ok(csv)
    }
}

impl Default for DownloadHistory {
    fn default() -> Self {
        Self::new()
    }
}
