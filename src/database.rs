use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::models::Transaction;

const DB_FILE: &str = "transactions.json";

#[derive(Default, Serialize, Deserialize)]
pub struct Database {
    pub transactions: Vec<Transaction>,
}

impl Database {
    pub fn load() -> Self {
        if Path::new(DB_FILE).exists() {
            let data = fs::read_to_string(DB_FILE).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Database::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(DB_FILE, json)?;
        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }
}
