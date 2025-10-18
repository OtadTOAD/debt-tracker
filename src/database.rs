use std::{fs, path::Path};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::models::Transaction;

const DB_FILE: &str = "transactions.json";
const BACKUP_DIR: &str = "backups";
const ATTACHMENTS_DIR: &str = "attachments";
const MAX_BACKUPS: usize = 50;

#[derive(Default, Serialize, Deserialize)]
pub struct Database {
    pub transactions: Vec<Transaction>,
}

impl Database {
    pub fn load() -> Self {
        let _ = fs::create_dir_all(ATTACHMENTS_DIR);

        if Path::new(DB_FILE).exists() {
            if let Ok(data) = fs::read_to_string(DB_FILE) {
                if let Ok(db) = serde_json::from_str(&data) {
                    return db;
                }
            }
        }

        if let Some(backup) = Self::get_most_recent_backup() {
            eprintln!(
                "Main database corrupted, attempting to restore from backup: {}",
                backup
            );
            if let Ok(data) = fs::read_to_string(&backup) {
                if let Ok(db) = serde_json::from_str(&data) {
                    let _ = fs::copy(&backup, DB_FILE);
                    return db;
                }
            }
        }

        Database::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(BACKUP_DIR)?;

        if Path::new(DB_FILE).exists() {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let backup_file = format!("{}/transactions_backup_{}.json", BACKUP_DIR, timestamp);
            fs::copy(DB_FILE, &backup_file)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(DB_FILE, &json)?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_file = format!("{}/transactions_backup_{}.json", BACKUP_DIR, timestamp);
        fs::write(&backup_file, &json)?;

        Self::cleanup_old_backups()?;

        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn copy_attachment_to_storage(
        source_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        fs::create_dir_all(ATTACHMENTS_DIR)?;

        let source = Path::new(source_path);
        let filename = source
            .file_name()
            .ok_or("Invalid filename")?
            .to_string_lossy();

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let new_filename = format!("{}_{}.{}", timestamp, filename, extension);
        let dest_path = format!("{}/{}", ATTACHMENTS_DIR, new_filename);

        fs::copy(source_path, &dest_path)?;

        Ok(dest_path)
    }

    fn get_most_recent_backup() -> Option<String> {
        if !Path::new(BACKUP_DIR).exists() {
            return None;
        }

        let mut backups: Vec<_> = fs::read_dir(BACKUP_DIR)
            .ok()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "json" {
                    Some(path.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        backups.sort();
        backups.reverse();
        backups.first().cloned()
    }

    fn cleanup_old_backups() -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(BACKUP_DIR).exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = fs::read_dir(BACKUP_DIR)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "json" {
                    let metadata = fs::metadata(&path).ok()?;
                    let modified = metadata.modified().ok()?;
                    Some((path, modified))
                } else {
                    None
                }
            })
            .collect();

        backups.sort_by(|a, b| b.1.cmp(&a.1));

        for (path, _) in backups.iter().skip(MAX_BACKUPS) {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }
}
