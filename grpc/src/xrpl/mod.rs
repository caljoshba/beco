use crate::{
    chain::chain_custody::{ChainCustody, PublicKey},
    errors::BecoError,
    traits::{key::Key, value::Values},
    user::public_user::PublicUser, proto::beco::AddAccountRequest,
};
use serde::{Deserialize, Serialize};
use tonic::Code;
use xrpl::{
    constants::CryptoAlgorithm,
    core::keypairs::{derive_classic_address, derive_keypair, generate_seed},
};

#[derive(Debug, Clone, Hash)]
pub struct XRPLKeyValues {
    pub public_key: String,
    pub classic_address: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct XRPLKey {
    seed: String,
    public_key: String,
    private_key: String,
    classic_address: String,
    alias: String,
}

impl XRPLKey {
    pub fn new(
        seed: String,
        public_key: String,
        private_key: String,
        classic_address: String,
        alias: String,
    ) -> Self {
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

impl Into<PublicKey> for XRPLKey {
    fn into(self) -> PublicKey {
        PublicKey {
            alias: self.alias(),
            address: self.classic_address,
        }
    }
}

impl Key<CryptoAlgorithm> for ChainCustody<XRPLKey, XRPLKeyValues> {
    fn create(
        &mut self,
        algorithm: Option<CryptoAlgorithm>,
        request: AddAccountRequest,
        public_user: &PublicUser,
    ) -> Result<(), BecoError> {
        let alias = request.alias;
        let does_alias_exist = self.does_alias_exist(alias.clone(), public_user);
        if does_alias_exist {
            return Err(BecoError {
                message: "Alias already exists".into(),
                status: Code::AlreadyExists,
            });
        }
        let seed_result = generate_seed(None, algorithm);
        if let Err(_error) = seed_result {
            return Err(BecoError {
                message: "Error Generating seed".into(),
                status: Code::Unknown,
            });
        }
        let seed = seed_result.unwrap();
        let (public_key, private_key) = derive_keypair(&seed, false).unwrap();
        let classic_address = derive_classic_address(&public_key).unwrap();
        let key = XRPLKey::new(seed, public_key, private_key, classic_address, alias);
        let keys_result = self.keys.value_mut(public_user);
        if keys_result.is_err() {
            return Err(BecoError {
                message: "User does not have permission to create a new key".into(),
                status: Code::PermissionDenied,
            });
        }
        let keys = keys_result.unwrap();
        keys.push(key.clone());
        Ok(())
    }
}
