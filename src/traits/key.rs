use crate::errors::key::CreateKeyError;

pub trait Key<A, K> {
    fn create(&mut self, algorithm: Option<A>, alias: Option<String>) -> Result<K, CreateKeyError>;
}