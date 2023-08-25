use crate::{errors::BecoError, user::public_user::PublicUser};

pub trait Key<A> {
    fn create(
        &mut self,
        algorithm: Option<A>,
        alias: String,
        public_user: &PublicUser,
    ) -> Result<(), BecoError>;
}
