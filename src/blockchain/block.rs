use super::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    nonce: i32,
    previous_hash: String,
    timestamp: i64,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(nonce: i32, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
        Block {
            nonce,
            previous_hash,
            timestamp,
            transactions,
        }
    }

    pub fn nonce(&self) -> i32 {
        self.nonce
    }

    pub fn previous_hash(&self) -> &String {
        &self.previous_hash
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}

impl Default for Block {
    fn default() -> Self {
        let mut b = Block::new(0, String::new(), vec![]);
        let json = serde_json::to_string(&b).unwrap();
        b.previous_hash = sha256::digest(json);
        b
    }
}
