use crate::{errors::key::CreateKeyError, response::WalletResponse, user::public_user::PublicUser};

pub trait Key<A> {
    fn create(&mut self, algorithm: Option<A>, alias: String, public_user: &PublicUser) -> Result<WalletResponse, CreateKeyError>;
    fn display(&self, public_user: &PublicUser) -> Vec<WalletResponse>;
}