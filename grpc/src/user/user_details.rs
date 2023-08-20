use crate::{permissioms::model::PermissionModel, errors::permission::PermissionError, enums::{value_reference::ValueReference, permission_model_operation::PermissionModelOperation}};

use super::public_user::PublicUser;

#[derive(Debug, Clone)]
pub struct UserDetails {
    pub id: String,
    pub first_name: PermissionModel<Option<String>>,
    pub other_names: PermissionModel<Option<Vec<String>>>,
    pub last_name: PermissionModel<Option<String>>,
}

impl UserDetails {
    pub fn new(id: String, first_name: Option<String>) -> Self {
        Self {
            id: id.clone(),
            first_name: PermissionModel::new(id.clone(), first_name, "first_name".into(), ValueReference::DETAIL_FIRST_NAME),
            other_names: PermissionModel::new(id.clone(), None, "other_names".into(), ValueReference::DETAIL_OTHER_NAMES),
            last_name: PermissionModel::new(id.clone(), None, "last_name".into(), ValueReference::DETAIL_LAST_NAME),
        }
    }

    pub fn as_public_user(&self, user: &PublicUser) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            first_name: self.first_name.value(user).unwrap_or(None),
            other_names: self.other_names.value(user).unwrap_or(None),
            last_name: self.last_name.value(user).unwrap_or(None),
        }
    }

    pub async fn update_first_name(&mut self, first_name: Option<String>, calling_user: &PublicUser, operation: PermissionModelOperation) -> Result<(), PermissionError> {
        let value = self.first_name.update(first_name, calling_user, operation).await;
        if value.is_err() {
            return Err(value.unwrap_err());
        }
        Ok(())
    }

    pub async fn update_other_names(&mut self, other_names: Option<Vec<String>>, calling_user: &PublicUser, operation: PermissionModelOperation) -> Result<(), PermissionError> {
        let value = self.other_names.update(other_names, calling_user, operation).await;
        if value.is_err() {
            return Err(value.unwrap_err());
        }
        Ok(())
    }

    pub async fn update_last_name(&mut self, last_name: Option<String>, calling_user: &PublicUser, operation: PermissionModelOperation) -> Result<(), PermissionError> {
        let value = self.last_name.update(last_name, calling_user, operation).await;
        if value.is_err() {
            return Err(value.unwrap_err());
        }
        Ok(())
    }
}