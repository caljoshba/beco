use serde::{Serialize, Deserialize};

use crate::{
    chain::chain_custody::{ChainCustody, PublicKey},
    enums::cypto_algortihms::EVMAlgortithm,
    errors::BecoError,
    traits::{key::Key, value::Values},
    user::public_user::PublicUser, proto::beco::AddAccountRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct EVMKeyValues {
    pub public_key: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct EVMKey {
    public_key: String,
    private_key: String,
    alias: String,
}

impl Values<EVMKeyValues> for EVMKey {
    fn values(&self) -> EVMKeyValues {
        EVMKeyValues {
            public_key: self.public_key.clone(),
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
        None
    }
}

impl Into<PublicKey> for EVMKey {
    fn into(self) -> PublicKey {
        PublicKey {
            alias: self.alias(),
            address: self.public_key,
        }
    }
}

impl Key<EVMAlgortithm> for ChainCustody<EVMKey, EVMKeyValues> {
    fn create(
        &mut self,
        algorithm: Option<EVMAlgortithm>,
        request: AddAccountRequest,
        public_user: &PublicUser,
    ) -> Result<(), BecoError> {
        unimplemented!()
    }
}

fn generate_key() {}
