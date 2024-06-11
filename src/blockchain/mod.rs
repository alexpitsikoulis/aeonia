mod block;
mod transaction;

use std::sync::{Arc, Mutex};

use block::Block;
use chrono::Utc;
use transaction::Transaction;

#[derive(Debug)]
pub enum Error {
    MutexPoisonError(String),
    SerializeJSONError(String),
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::MutexPoisonError(e) => Self::new(std::io::ErrorKind::Other, e),
            Error::SerializeJSONError(e) => Self::new(std::io::ErrorKind::InvalidData, e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Blockchain {
    address: String,
    chain: Arc<Mutex<Vec<Arc<Block>>>>,
    transaction_pool: Arc<Mutex<Vec<Transaction>>>,
    mining_difficulty: u8,
    mining_reward: f64,
}

impl Blockchain {
    pub fn new(address: String, mining_difficulty: u8, mining_reward: f64) -> Result<Self> {
        let mut blockchain = Blockchain {
            address,
            chain: Arc::new(Mutex::new(vec![])),
            transaction_pool: Arc::new(Mutex::new(vec![])),
            mining_difficulty,
            mining_reward,
        };
        blockchain.add_block(0)?;
        Ok(blockchain)
    }

    pub fn last_block(&self) -> Option<Arc<Block>> {
        match self.chain.lock() {
            Ok(chain) => chain.get(chain.len().saturating_sub(1)).cloned(),
            Err(_) => None,
        }
    }

    fn add_block(&mut self, nonce: i32) -> Result<Arc<Block>> {
        let previous_block = self.last_block().unwrap_or_default();
        let previous_hash = previous_block
            .hash()
            .map_err(|e| Error::SerializeJSONError(e.to_string()))?;
        let mut transactions: Vec<Transaction> = vec![];
        let mut transaction_pool_lock = self
            .transaction_pool
            .lock()
            .map_err(|e| Error::MutexPoisonError(e.to_string()))?;
        while transaction_pool_lock.iter().len() > 0 {
            let transaction = match transaction_pool_lock.pop() {
                Some(transaction) => transaction,
                None => break,
            };
            transactions.push(transaction.clone());
        }
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
        let b = Arc::new(Block::new(nonce, previous_hash, transactions, timestamp, self.address.clone()));
        let mut chain_lock = self
            .chain
            .lock()
            .map_err(|e| Error::MutexPoisonError(e.to_string()))?;
        chain_lock.push(b.clone());
        Ok(b)
    }

    pub fn add_transation_to_pool(
        &mut self,
        sender: String,
        recipient: String,
        amount: f64,
    ) -> Result<Transaction> {
        let transaction = Transaction::new(sender, recipient, amount);
        let mut transaction_pool_lock = self
            .transaction_pool
            .lock()
            .map_err(|e| Error::MutexPoisonError(e.to_string()))?;
        transaction_pool_lock.push(transaction.clone());
        Ok(transaction)
    }

    fn valid_proof(
        &self,
        nonce: i32,
        previous_hash: String,
        transactions: Vec<Transaction>,
    ) -> bool {
        let zeros = vec!["0"; self.mining_difficulty as usize].join("");
        let guess_block = Block::new(nonce, previous_hash, transactions, 0, "none".into());
        if let Ok(guess_json) = serde_json::to_string(&guess_block) {
            let guess_hash = sha256::digest(guess_json);
            guess_hash.starts_with(&zeros)
        } else {
            false
        }
    }

    fn proof_of_work(&mut self) -> Result<i32> {
        let transaction_pool_lock = self
            .transaction_pool
            .lock()
            .map_err(|e| Error::MutexPoisonError(e.to_string()))?;
        let last_block = self.last_block().unwrap();
        let previous_hash = last_block
            .hash()
            .map_err(|e| Error::SerializeJSONError(e.to_string()))?;
        let mut nonce = 0;
        while !self.valid_proof(nonce, previous_hash.clone(), transaction_pool_lock.clone()) {
            nonce += 1;
        }
        Ok(nonce)
    }

    pub fn mining(&mut self) -> bool {
        if self.add_transation_to_pool("THE BLOCKCHAIN".into(), self.address.clone(), self.mining_reward).is_ok() {
            if let Ok(nonce) = self.proof_of_work() {
                self.add_block(nonce).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        match Blockchain::new("0".into(), 3, 1.0) {
            Ok(blockchain) => blockchain,
            Err(e) => {
                let mut retries = 3;
                while retries >= 0 {
                    if let Ok(blockchain) = Blockchain::new("0".into(), 3, 1.0) {
                        return blockchain;
                    } else {
                        retries -= 1;
                    }
                }
                panic!("failed to create default blockchain: {:?}", e);
            }
        }
    }
}

impl std::fmt::Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain_lock = self.chain.lock().unwrap();
        for block in chain_lock.iter() {
            writeln!(f, "{}", vec!["="; 100].join(""))?;
            writeln!(f, "\tnonce: {}", block.nonce())?;
            writeln!(f, "\tprevious_hash: {}", block.previous_hash())?;
            writeln!(f, "\ttimestamp: {}", block.timestamp())?;
            writeln!(f, "\ttransactions: {:?}", block.transactions())?;
            writeln!(f, "\tminer: {:?}", block.miner())?;
            writeln!(f, "{}", vec!["="; 100].join(""))?;
        }
        writeln!(f)?;
        if let Ok(transaction_pool) = self.transaction_pool.lock() {
            writeln!(f, "transaction pool")?;
            for transaction in transaction_pool.iter() {
                writeln!(f, "{}", vec!["-"; 50].join(""))?;
                writeln!(f, "\tsender: {}", transaction.sender)?;
                writeln!(f, "\trecipient: {}", transaction.recipient)?;
                writeln!(f, "\tamount: {}", transaction.amount)?;
                writeln!(f, "{}", vec!["-"; 50].join(""))?;
            }
        };
        writeln!(f, "end\n")?;
        Ok(())
    }
}
