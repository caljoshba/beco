use super::user_details::UserDetails;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicUser {
    pub id: String,
}

impl PublicUser {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl From<UserDetails> for PublicUser {
    fn from(value: UserDetails) -> Self {
        Self { id: value.id }
    }
}