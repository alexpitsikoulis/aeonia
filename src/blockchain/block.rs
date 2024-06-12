use super::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    nonce: i32,
    previous_hash: String,
    timestamp: i64,
    transactions: Vec<Transaction>,
    miner: String,
}

impl Block {
    pub fn new(
        nonce: i32,
        previous_hash: String,
        transactions: Vec<Transaction>,
        timestamp: i64,
        miner: String,
    ) -> Self {
        Block {
            nonce,
            previous_hash,
            timestamp,
            transactions,
            miner,
        }
    }

    pub fn hash(&self) -> String {
        sha256::digest(self.to_string())
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

    pub fn miner(&self) -> &String {
        &self.miner
    }
}

impl Default for Block {
    fn default() -> Self {
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
        let mut b = Block::new(0, String::new(), vec![], timestamp, "none".into());
        let json = serde_json::to_string(&b).unwrap();
        b.previous_hash = sha256::digest(json);
        b
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        let transactions: Vec<String> = self.transactions.iter().map(|t| t.to_string()).collect();
        format!(
            r#"
        {{
            "nonce": {},
            "previous_hash": "{}",
            "timestamp": {},
            "transactions": [{}],
            "miner": "{}",
        }}
        "#,
            self.nonce,
            self.previous_hash,
            self.timestamp,
            transactions.join(","),
            self.miner
        )
    }
}
