mod block;
mod transaction;

use std::sync::{Arc, Mutex};

use block::Block;
use chrono::Utc;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

pub use transaction::Transaction;

use crate::wallet::Wallet;

const MINING_DIFFICULTY: u8 = 3;
const MINING_REWARD: f64 = 1.0;

#[derive(Debug)]
pub enum Error {
    MutexPoison(String),
    Json(String),
    Ecdsa(String),
    InvalidSignature(String),
    AvailableBalanceExceeded(String),
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::MutexPoison(e) => Self::new(std::io::ErrorKind::Other, e),
            Error::Json(e) => Self::new(std::io::ErrorKind::InvalidData, e),
            Error::Ecdsa(e) => Self::new(std::io::ErrorKind::Other, e),
            Error::InvalidSignature(e) => Self::new(std::io::ErrorKind::InvalidData, e),
            Error::AvailableBalanceExceeded(sender) => Self::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "transaction exceeds available balance for sender {}",
                    sender
                ),
            ),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Blockchain {
    wallet: Wallet,
    chain: Arc<Mutex<Vec<Arc<Block>>>>,
    transaction_pool: Arc<Mutex<Vec<Transaction>>>,
}

impl Blockchain {
    pub fn new(version: u8) -> Result<Self> {
        let mut blockchain = Blockchain {
            wallet: Wallet::new(version).map_err(|e| Error::Ecdsa(e.to_string()))?,
            chain: Arc::new(Mutex::new(vec![])),
            transaction_pool: Arc::new(Mutex::new(vec![])),
        };
        let address = blockchain.wallet.address().clone();
        blockchain.add_block(0, &address)?;
        Ok(blockchain)
    }

    pub fn last_block(&self) -> Option<Arc<Block>> {
        match self.chain.lock() {
            Ok(chain) => chain.get(chain.len().saturating_sub(1)).cloned(),
            Err(_) => None,
        }
    }

    fn add_block(&mut self, nonce: i32, miner: &String) -> Result<Arc<Block>> {
        let previous_block = self.last_block().unwrap_or_default();
        let previous_hash = previous_block.hash();
        let mut transactions: Vec<Transaction> = vec![];
        let mut transaction_pool_lock = self
            .transaction_pool
            .lock()
            .map_err(|e| Error::MutexPoison(e.to_string()))?;
        while transaction_pool_lock.iter().len() > 0 {
            let transaction = match transaction_pool_lock.pop() {
                Some(transaction) => transaction,
                None => break,
            };
            transactions.push(transaction.clone());
        }
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
        let b = Arc::new(Block::new(
            nonce,
            previous_hash,
            transactions,
            timestamp,
            miner.clone(),
        ));
        let mut chain_lock = self
            .chain
            .lock()
            .map_err(|e| Error::MutexPoison(e.to_string()))?;
        chain_lock.push(b.clone());
        Ok(b)
    }

    pub fn add_transation_to_pool(
        &mut self,
        transaction: Transaction,
        signature: Signature,
        verifying_key: VerifyingKey,
    ) -> Result<Transaction> {
        if let Err(e) = verifying_key.verify(transaction.to_string().as_bytes(), &signature) {
            Err(Error::InvalidSignature(e.to_string()))
        } else {
            let sender = transaction.clone().sender;
            if &sender.clone() != self.wallet.address() {
                let sender_balance = self.calculate_transactions_total(sender.clone())?;
                if sender_balance < transaction.amount {
                    return Err(Error::AvailableBalanceExceeded(sender));
                }
            }
            let mut transaction_pool_lock = self
                .transaction_pool
                .lock()
                .map_err(|e| Error::MutexPoison(e.to_string()))?;
            transaction_pool_lock.push(transaction.clone());
            Ok(transaction)
        }
    }

    pub fn deposit_to_wallet(&mut self, recipient: &String, amount: f64) -> Result<Transaction> {
        let (transaction, signature, v_key) = self.wallet.sign_transaction(recipient, amount).map_err(|e| Error::Ecdsa(e.to_string()))?;
        self.add_transation_to_pool(transaction, signature, v_key)
    }

    fn valid_proof(
        &self,
        nonce: i32,
        previous_hash: String,
        transactions: Vec<Transaction>,
    ) -> bool {
        let zeros = vec!["0"; MINING_DIFFICULTY as usize].join("");
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
            .map_err(|e| Error::MutexPoison(e.to_string()))?;
        let last_block = self.last_block().unwrap();
        let previous_hash = last_block.hash();
        let mut nonce = 0;
        while !self.valid_proof(nonce, previous_hash.clone(), transaction_pool_lock.clone()) {
            nonce += 1;
        }
        Ok(nonce)
    }

    pub fn mining(&mut self, miner: &String) -> bool {
        if let Ok((transaction, signature, v_key)) =
            self.wallet.sign_transaction(miner, MINING_REWARD)
        {
            if self
                .add_transation_to_pool(transaction, signature, v_key)
                .is_ok()
            {
                if let Ok(nonce) = self.proof_of_work() {
                    self.add_block(nonce, miner).is_ok()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn calculate_transactions_total(&mut self, address: String) -> Result<f64> {
        let mut total_amount = 0.0;
        let chain_lock = self
            .chain
            .lock()
            .map_err(|e| Error::MutexPoison(e.to_string()))?;
        for block in chain_lock.iter() {
            for transaction in block.transactions() {
                if transaction.recipient == address {
                    total_amount += transaction.amount;
                }
                if transaction.sender == address {
                    total_amount -= transaction.amount;
                }
            }
        }
        let transaction_pool_lock = self.transaction_pool.lock().map_err(|e| Error::MutexPoison(e.to_string()))?;
        for transaction in transaction_pool_lock.iter() {
            if transaction.recipient == address {
                total_amount += transaction.amount;
            }
            if transaction.sender == address {
                total_amount -= transaction.amount;
            }
        }
        Ok(total_amount)
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        match Blockchain::new(0x00) {
            Ok(blockchain) => blockchain,
            Err(e) => {
                let mut retries = 3;
                while retries >= 0 {
                    if let Ok(blockchain) = Blockchain::new(0x00) {
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
