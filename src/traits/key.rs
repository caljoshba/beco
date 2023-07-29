use crate::{errors::key::CreateKeyError, response::WalletResponse};

pub trait Key<A> {
    fn create(&mut self, algorithm: Option<A>, alias: String) -> Result<WalletResponse, CreateKeyError>;
    fn display(&self) -> Vec<WalletResponse>;
}