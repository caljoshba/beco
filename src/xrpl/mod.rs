use crate::{traits::{key::Key, value::Values}, errors::key::CreateKeyError, keys::ChainCustody};
use xrpl::{core::keypairs::{generate_seed, derive_keypair, derive_classic_address}, constants::CryptoAlgorithm};

#[derive(Debug, Clone)]
pub struct XRPLKeyValues {
    public_key: String,
    classic_address: String,
    alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct XRPLKey {
    seed: String,
    public_key: String,
    private_key: String,
    classic_address: String,
    alias: Option<String>,
}

impl XRPLKey {
    pub fn new(seed: String, public_key: String, private_key: String, classic_address: String, alias: Option<String>) -> Self {
        Self {
            seed,
            public_key,
            private_key,
            classic_address,
            alias,
        }
    }
}

impl Values<XRPLKeyValues> for XRPLKey {
    fn values(&self) -> XRPLKeyValues {
        XRPLKeyValues {
            public_key: self.public_key.clone(),
            classic_address: self.classic_address.clone(),
            alias: self.alias.clone(),
        }
    }
}

impl Key<CryptoAlgorithm, XRPLKey> for ChainCustody<XRPLKey, XRPLKeyValues>  {
    fn create(&mut self, algorithm: Option<CryptoAlgorithm>, alias: Option<String>) -> Result<XRPLKey, CreateKeyError> {
        let seed_result = generate_seed(None, algorithm);
        if let Err(_error) = seed_result {
            return Err(CreateKeyError{ chain: self.chain.clone(), message: "Something" });
        }
        let seed = seed_result.unwrap();
        let (public_key, private_key) = derive_keypair(&seed, false).unwrap();
        let classic_address = derive_classic_address(&public_key).unwrap();
        let key = XRPLKey::new(seed, public_key, private_key, classic_address, alias);
        self.keys.push(key.clone());
        Ok(key)
    }
}