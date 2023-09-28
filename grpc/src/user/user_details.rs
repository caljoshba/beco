use serde::{Serialize, Deserialize};
use std::hash::Hash;

use crate::{
    chain::chain_custody::PublicChainCustody, enums::value_reference::ValueReference,
    permissions::model::PermissionModel,
};

use super::public_user::PublicUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetails {
    pub id: String,
    pub first_name: PermissionModel<Option<String>>,
    pub other_names: PermissionModel<Option<Vec<String>>>,
    pub last_name: PermissionModel<Option<String>>,
}

impl Hash for UserDetails {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.first_name.hash(state);
        self.other_names.hash(state);
        self.last_name.hash(state);
    }
}

impl UserDetails {
    pub fn new(id: String, first_name: Option<String>) -> Self {
        Self {
            id: id.clone(),
            first_name: PermissionModel::new(
                id.clone(),
                first_name,
                "first_name".into(),
                ValueReference::DETAIL_FIRST_NAME,
            ),
            other_names: PermissionModel::new(
                id.clone(),
                None,
                "other_names".into(),
                ValueReference::DETAIL_OTHER_NAMES,
            ),
            last_name: PermissionModel::new(
                id.clone(),
                None,
                "last_name".into(),
                ValueReference::DETAIL_LAST_NAME,
            ),
        }
    }

    pub fn as_public_user(
        &self,
        user: &PublicUser,
        chain_accounts: Vec<PublicChainCustody>,
    ) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            first_name: self.first_name.value(user).unwrap_or(None),
            other_names: self.other_names.value(user).unwrap_or(None),
            last_name: self.last_name.value(user).unwrap_or(None),
            chain_accounts,
        }
    }
}
