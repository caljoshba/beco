use crate::{traits::{value::Values, key::Key}, keys::ChainCustody, enums::cypto_algortihms::EVMAlgortithm, errors::key::CreateKeyError, response::WalletResponse};


#[derive(Debug, Clone)]
pub struct EVMKeyValues {
    pub public_key: String,
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct EVMKey {
    public_key: String,
    private_key: String,
    alias: String,
}

impl Values<EVMKeyValues> for EVMKey {
    fn values(&self) -> EVMKeyValues {
        EVMKeyValues { public_key: self.public_key.clone(), alias: self.alias.clone() }
    }
}

impl Key<EVMAlgortithm> for ChainCustody<EVMKey, EVMKeyValues> {
    fn create(&mut self, algorithm: Option<EVMAlgortithm>, alias: String) -> Result<WalletResponse, CreateKeyError> {
        unimplemented!()
    }

    fn display(&self) -> Vec<WalletResponse> {
        unimplemented!()
    }
}

fn generate_key() {

}