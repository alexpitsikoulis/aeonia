mod block;
mod transaction;

use std::sync::{Arc, Mutex};

use block::Block;
use transaction::Transaction;

#[derive(Debug)]
pub enum Error {
    MutexPoisonError(String),
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::MutexPoisonError(e) => Self::new(std::io::ErrorKind::Other, e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Blockchain {
    chain: Arc<Mutex<Vec<Arc<Block>>>>,
    transaction_pool: Arc<Mutex<Vec<Arc<Transaction>>>>,
}

impl Blockchain {
    pub fn new() -> Result<Self> {
        let mut blockchain = Blockchain {
            chain: Arc::new(Mutex::new(vec![])),
            transaction_pool: Arc::new(Mutex::new(vec![])),
        };
        blockchain.add_block(0)?;
        Ok(blockchain)
    }

    pub fn last_block(&mut self) -> Option<Arc<Block>> {
        match self.chain.lock() {
            Ok(chain) => chain.get(chain.len().saturating_sub(1)).cloned(),
            Err(_) => None,
        }
    }

    pub fn add_block(&mut self, nonce: i32) -> Result<Arc<Block>> {
        let previous_block = self.last_block().unwrap_or_default();
        let previous_block_json = serde_json::to_string(previous_block.as_ref()).unwrap();
        let previous_hash = sha256::digest(previous_block_json);
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
            transactions.push(transaction.as_ref().clone());
        }
        let b = Arc::new(Block::new(nonce, previous_hash, transactions));
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
    ) -> Result<Arc<Transaction>> {
        let transaction = Arc::new(Transaction::new(sender, recipient, amount));
        let mut transaction_pool_lock = self
            .transaction_pool
            .lock()
            .map_err(|e| Error::MutexPoisonError(e.to_string()))?;
        transaction_pool_lock.push(transaction.clone());
        Ok(transaction)
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
