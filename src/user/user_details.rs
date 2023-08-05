use crate::{permissioms::model::PermissionModel};

use super::public_user::PublicUser;

#[derive(Debug, Clone)]
pub struct UserDetails {
    pub id: String,
    name: PermissionModel<String>,
}

impl UserDetails {
    pub fn new(owner: PublicUser, name: String) -> Self {
        Self {
            id: owner.id.clone(),
            name: PermissionModel::new(owner, name, "name".into()),
        }
    }
}