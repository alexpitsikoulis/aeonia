use super::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub enum Error {
    SerializeJSONError(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::SerializeJSONError(e) => e.clone(),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    nonce: i32,
    previous_hash: String,
    timestamp: i64,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        nonce: i32,
        previous_hash: String,
        transactions: Vec<Transaction>,
        timestamp: i64,
    ) -> Self {
        Block {
            nonce,
            previous_hash,
            timestamp,
            transactions,
        }
    }

    pub fn hash(&self) -> Result<String> {
        let json =
            serde_json::to_string(self).map_err(|e| Error::SerializeJSONError(e.to_string()))?;
        Ok(sha256::digest(json))
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
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
        let mut b = Block::new(0, String::new(), vec![], timestamp);
        let json = serde_json::to_string(&b).unwrap();
        b.previous_hash = sha256::digest(json);
        b
    }
}
