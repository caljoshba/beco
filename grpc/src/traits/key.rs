use crate::{errors::BecoError, user::public_user::PublicUser, proto::beco::AddAccountRequest};

pub trait Key<A> {
    fn create(
        &mut self,
        algorithm: Option<A>,
        request: AddAccountRequest,
        public_user: &PublicUser,
    ) -> Result<(), BecoError>;
}
