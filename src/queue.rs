use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub url: String,
    pub target_type: String,
    pub quality: Option<String>,
    pub output_dir: String,
    pub status: QueueStatus,
    pub added_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub progress: f64,
    pub file_path: Option<String>,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadQueue {
    pub items: Vec<QueueItem>,
    pub max_parallel: usize,
    pub active_count: usize,
}

impl DownloadQueue {
    pub fn load() -> Self {
        let path = config::queue_path();
        let mut queue: DownloadQueue = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => DownloadQueue::new(),
            }
        } else {
            DownloadQueue::new()
        };
        queue.active_count = 0;
        for item in &mut queue.items {
            if item.status == QueueStatus::Downloading {
                item.status = QueueStatus::Pending;
            }
        }
        queue
    }

    pub fn new() -> Self {
        DownloadQueue {
            items: Vec::new(),
            max_parallel: 3,
            active_count: 0,
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config::queue_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(&mut self, url: &str, target_type: &str, quality: Option<String>, output_dir: &str) -> String {
        let id = Uuid::new_v4().to_string()[..8].to_string();
        let item = QueueItem {
            id: id.clone(),
            url: url.to_string(),
            target_type: target_type.to_string(),
            quality,
            output_dir: output_dir.to_string(),
            status: QueueStatus::Pending,
            added_at: Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            file_path: None,
            error: None,
            retry_count: 0,
        };
        self.items.push(item);
        let _ = self.save();
        id
    }

    pub fn start_next(&mut self) -> Option<&mut QueueItem> {
        if self.active_count >= self.max_parallel {
            return None;
        }
        let pos = self.items.iter().position(|i| i.status == QueueStatus::Pending)?;
        self.items[pos].status = QueueStatus::Downloading;
        self.items[pos].started_at = Some(Utc::now().to_rfc3339());
        self.active_count += 1;
        let _ = self.save();
        Some(&mut self.items[pos])
    }

    pub fn complete(&mut self, id: &str, file_path: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = QueueStatus::Completed;
            item.completed_at = Some(Utc::now().to_rfc3339());
            item.progress = 100.0;
            item.file_path = Some(file_path.to_string());
            self.active_count = self.active_count.saturating_sub(1);
            let _ = self.save();
        }
    }

    pub fn fail(&mut self, id: &str, error: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = QueueStatus::Failed;
            item.error = Some(error.to_string());
            self.active_count = self.active_count.saturating_sub(1);
            let _ = self.save();
        }
    }

    pub fn pause(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = QueueStatus::Paused;
            self.active_count = self.active_count.saturating_sub(1);
            let _ = self.save();
        }
    }

    pub fn resume(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if item.status == QueueStatus::Paused {
                item.status = QueueStatus::Pending;
                let _ = self.save();
            }
        }
    }

    pub fn cancel(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if item.status == QueueStatus::Downloading {
                self.active_count = self.active_count.saturating_sub(1);
            }
            item.status = QueueStatus::Failed;
            item.error = Some("Cancelled by user".to_string());
            let _ = self.save();
        }
    }

    pub fn retry(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if item.status == QueueStatus::Failed {
                item.status = QueueStatus::Pending;
                item.error = None;
                item.retry_count += 1;
                let _ = self.save();
            }
        }
    }

    pub fn clear_completed(&mut self) {
        self.items.retain(|i| i.status != QueueStatus::Completed);
        let _ = self.save();
    }

    pub fn clear_all(&mut self) {
        self.items.clear();
        self.active_count = 0;
        let _ = self.save();
    }

    pub fn list(&self) -> &[QueueItem] {
        &self.items
    }

    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == QueueStatus::Pending).count()
    }

    pub fn failed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == QueueStatus::Failed).count()
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == QueueStatus::Completed).count()
    }
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self::new()
    }
}
