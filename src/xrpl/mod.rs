use crate::{traits::{key::Key, value::Values}, errors::key::CreateKeyError, chain::chain_custody::ChainCustody, response::WalletResponse};
use xrpl::{core::keypairs::{generate_seed, derive_keypair, derive_classic_address}, constants::CryptoAlgorithm};

#[derive(Debug, Clone)]
pub struct XRPLKeyValues {
    pub public_key: String,
    pub classic_address: String,
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct XRPLKey {
    seed: String,
    public_key: String,
    private_key: String,
    classic_address: String,
    alias: String,
}

impl XRPLKey {
    pub fn new(seed: String, public_key: String, private_key: String, classic_address: String, alias: String) -> Self {
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

    fn alias(&self) -> String {
        self.alias.clone()
    }

    fn public_key(&self) -> String {
        self.public_key.clone()
    }

    fn classic_address(&self) -> Option<String> {
        Some(self.classic_address.clone())
    }
}

impl Key<CryptoAlgorithm> for ChainCustody<XRPLKey, XRPLKeyValues>  {
    fn create(&mut self, algorithm: Option<CryptoAlgorithm>, alias: String) -> Result<WalletResponse, CreateKeyError> {
        let does_alias_exist = self.does_alias_exist(alias.clone());
        if does_alias_exist {
            return Err(CreateKeyError{ chain: self.chain.clone(), message: "Alias already exists" });
        }
        let seed_result = generate_seed(None, algorithm);
        if let Err(_error) = seed_result {
            return Err(CreateKeyError{ chain: self.chain.clone(), message: "Error Generating seed" });
        }
        let seed = seed_result.unwrap();
        let (public_key, private_key) = derive_keypair(&seed, false).unwrap();
        let classic_address = derive_classic_address(&public_key).unwrap();
        let key = XRPLKey::new(seed, public_key, private_key, classic_address, alias);
        self.keys.push(key.clone());
        Ok(WalletResponse {
            alias: key.alias(),
            public_key: key.public_key(),
            classic_address: key.classic_address(),
        })
    }

    fn display(&self) -> Vec<WalletResponse> {
        self.keys.iter().map(|key| {
            WalletResponse {
                alias: key.alias(),
                public_key: key.public_key(),
                classic_address: key.classic_address(),
            }
        }).collect()
    }
}