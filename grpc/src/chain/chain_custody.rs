use std::{marker::PhantomData, fmt::Debug};

use crate::{enums::{blockchain::Blockchain, value_reference::ValueReference}, traits::value::Values, permissioms::model::PermissionModel, user::public_user::PublicUser};
#[derive(Debug, Clone)]
pub struct ChainCustody<T, A> where T: Values<A> + Clone + Debug {
    pub chain: Blockchain,
    pub keys: PermissionModel<Vec<T>>,
    phantom_type: PhantomData<A>
}

impl<T, A> ChainCustody<T, A> where T: Values<A> + Clone + Debug {
    pub fn new(chain: Blockchain, owner_id: String) -> Self {
        Self {
            chain,
            keys: PermissionModel::new(owner_id, vec![], "keys".into(), ValueReference::CHAIN_HEYS),
            phantom_type: PhantomData
        }
    }

    pub fn does_alias_exist(&self, alias: String, user: &PublicUser) -> bool {
        self.keys.value(user).unwrap_or(vec![]).iter().find(|&key| key.alias() == alias).is_some()
    }
}