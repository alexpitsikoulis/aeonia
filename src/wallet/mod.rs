use super::blockchain::Transaction;

use base58::ToBase58;
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    elliptic_curve::zeroize::Zeroizing,
    pkcs8::EncodePrivateKey,
    PublicKey, SecretKey,
};
use rand_core::OsRng;
use ripemd::Digest;

#[derive(Debug)]
pub enum Error {
    EcdsaError(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::EcdsaError(e) => e.clone(),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct Wallet {
    address: String,
    private_key: Zeroizing<String>,
    public_key: PublicKey,
}

impl Wallet {
    pub fn new(version: u8) -> Result<Self> {
        let private_key = SecretKey::random(&mut OsRng);
        let public_key = private_key.public_key();
        let private_key = private_key
            .to_pkcs8_pem(Default::default())
            .map_err(|e| Error::EcdsaError(e.to_string()))?;
        let address = Self::derive_address(public_key, version);

        Ok(Wallet {
            address,
            private_key,
            public_key,
        })
    }

    pub fn derive_address(public_key: PublicKey, version: u8) -> String {
        let mut public_key_sha256 = sha256::digest(public_key.to_string());
        let public_key_ripemd = ripemd::Ripemd160::digest(&public_key_sha256);
        let public_key_ripemd = public_key_ripemd.as_slice();
        let versioned_public_key_ripemd = &[&[version], public_key_ripemd].concat();
        public_key_sha256 = sha256::digest(versioned_public_key_ripemd.clone());
        public_key_sha256 = sha256::digest(public_key_sha256);
        let versioned_public_key_ripemd = &[
            versioned_public_key_ripemd,
            public_key_sha256.split_off(4).as_bytes(),
        ]
        .concat();
        versioned_public_key_ripemd.as_slice().to_base58()
    }

    pub fn sign_transaction(
        &mut self,
        recipient: &String,
        amount: f64,
    ) -> Result<(Transaction, Signature, VerifyingKey)> {
        let transaction = Transaction::new(self.address.clone(), recipient.clone(), amount);
        let private_key = self
            .private_key
            .parse::<SecretKey>()
            .map_err(|e| Error::EcdsaError(e.to_string()))?;
        let signing_key: SigningKey = private_key.into();
        Ok((
            transaction.clone(),
            signing_key.sign(transaction.to_string().as_bytes()),
            self.public_key.into(),
        ))
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }
}
