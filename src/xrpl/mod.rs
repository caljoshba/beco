#![allow(dead_code)]
mod test;

use crate::{enums::blockchain::Blockchain, traits::key::Key, errors::key::CreateKeyError};
use xrpl::{core::keypairs::{generate_seed, derive_keypair, derive_classic_address}, constants::CryptoAlgorithm};

#[derive(Debug, Clone)]
pub struct XRPLKey {
    pub seed: String,
    pub public_key: String,
    pub private_key: String,
    pub classic_address: String,
}

impl XRPLKey {
    pub fn new(seed: String, public_key: String, private_key: String, classic_address: String) -> Self {
        Self {
            seed,
            public_key,
            private_key,
            classic_address,
        }
    }
}

pub struct Keys {
    pub chain: Blockchain,
    pub keys: Vec<XRPLKey>,
}

impl Keys {
    pub fn new() -> Self {
        Self {
            chain: Blockchain::XRPL,
            keys: vec![],
        }
    }
}

impl Key<CryptoAlgorithm, XRPLKey> for Keys {
    fn create(&mut self, algorithm: Option<CryptoAlgorithm>) -> Result<XRPLKey, CreateKeyError> {
        let seed_result = generate_seed(None, algorithm);
        if let Err(_error) = seed_result {
            return Err(CreateKeyError{ chain: self.chain.clone(), message: "Something" });
        }
        let seed = seed_result.unwrap();
        let (public_key, private_key) = derive_keypair(&seed, false).unwrap();
        let classic_address = derive_classic_address(&public_key).unwrap();
        let key = XRPLKey::new(seed, public_key, private_key, classic_address);
        self.keys.push(key.clone());
        Ok(key)
    }
}