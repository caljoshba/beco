use crate::{traits::{value::Values, key::Key}, chain::chain_custody::ChainCustody, enums::cypto_algortihms::EVMAlgortithm, errors::key::CreateKeyError, response::WalletResponse, user::public_user::PublicUser};


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

impl Key<EVMAlgortithm> for ChainCustody<EVMKey, EVMKeyValues> {
    fn create(&mut self, algorithm: Option<EVMAlgortithm>, alias: String, public_user: &PublicUser) -> Result<WalletResponse, CreateKeyError> {
        unimplemented!()
    }

    fn display(&self, public_user: &PublicUser) -> Vec<WalletResponse> {
        unimplemented!()
    }
}

fn generate_key() {

}