use std::marker::PhantomData;

use crate::{enums::blockchain::Blockchain, traits::value::Values};
#[derive(Debug, Clone)]
pub struct ChainCustody<T, A> where T: Values<A> {
    pub chain: Blockchain,
    pub keys: Vec<T>,
    phantom_type: PhantomData<A>
}

impl<T, A> ChainCustody<T, A> where T: Values<A> {
    pub fn new(chain: Blockchain) -> Self {
        Self {
            chain,
            keys: vec![],
            phantom_type: PhantomData
        }
    }

    pub fn does_alias_exist(&self, alias: String) -> bool {
        self.keys.iter().find(|&key| key.alias() == alias).is_some()
    }
}