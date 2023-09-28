use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use tonic::Code;
use std::hash::Hash;

use crate::{
    enums::value_reference::ValueReference, errors::BecoError, user::public_user::PublicUser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionModel<T>
where
    T: Clone + Debug + Hash,
{
    owner_id: String,
    editors: Vec<PublicUser>,
    viewers: Vec<PublicUser>,
    value: T,
    key: String,
    reference: ValueReference,
}

impl<T> Hash for PermissionModel<T>
where
    T: Clone + Debug + Hash, {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.owner_id.hash(state);
        self.editors.hash(state);
        self.viewers.hash(state);
        self.value.hash(state);
        self.key.hash(state);
        self.reference.hash(state);
    }
}

impl<T> PermissionModel<T>
where
    T: Clone + Debug + Hash,
{
    pub fn new(owner_id: String, value: T, key: String, reference: ValueReference) -> Self {
        Self {
            owner_id,
            editors: vec![],
            viewers: vec![],
            value,
            key,
            reference,
        }
    }
    pub fn value(&self, user: &PublicUser) -> Result<T, BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, user)
            && !PermissionModel::is_owner_or_viewer(&self, user)
        {
            return Err(BecoError {
                message: format!(
                    "User does not have permission to view this value: {}",
                    self.key
                ),
                status: Code::PermissionDenied,
            });
        }
        Ok(self.value.clone())
    }

    pub fn value_mut(&mut self, user: &PublicUser) -> Result<&mut T, BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, user) {
            return Err(BecoError {
                message: format!(
                    "User does not have permission to view this mut value: {}",
                    self.key
                ),
                status: Code::PermissionDenied,
            });
        }
        Ok(&mut self.value)
    }

    pub async fn propose<'a>(&self, value: T, calling_user: &PublicUser) -> Result<(), BecoError> {
        // this will generate a request that is passed to the p2p server for validation
        // an event listener is attached, waiting for a FAILED or VALIDATED call from p2p server
        // return OK or Err depending on that response
        // unimplemented!()
        Ok(())
    }

    pub async fn update(&mut self, value: T, calling_user: &PublicUser) -> Result<(), BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(BecoError {
                message: format!(
                    "User does not have permission to update this value: {}",
                    self.key
                ),
                status: Code::PermissionDenied,
            });
        }
        self.value = value;
        Ok(())
    }

    pub fn add_viewer(
        &mut self,
        user: PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(BecoError {
                message: "User does not have permission to add a viewer".into(),
                status: Code::PermissionDenied,
            });
        }
        if self.owner_id == user.id {
            return Err(BecoError {
                message: "User is owner".into(),
                status: Code::AlreadyExists,
            });
        }
        if self.viewers.iter().find(|&usr| usr == &user).is_some() {
            return Err(BecoError {
                message: "User already has permission to view this value".into(),
                status: Code::AlreadyExists,
            });
        }
        self.viewers.push(user);
        Ok(())
    }

    pub fn add_editor(
        &mut self,
        user: PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(BecoError {
                message: "User does not have permission to add an editor".into(),
                status: Code::PermissionDenied,
            });
        }
        if self.owner_id == user.id {
            return Err(BecoError {
                message: "User is owner".into(),
                status: Code::AlreadyExists,
            });
        }
        if self.editors.iter().find(|&usr| usr == &user).is_some() {
            return Err(BecoError {
                message: "User already has permission to edit this value".into(),
                status: Code::AlreadyExists,
            });
        }
        self.editors.push(user);
        Ok(())
    }

    pub fn remove_viewer(
        &mut self,
        user: PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(BecoError {
                message: "User does not have permission to remove a viewer".into(),
                status: Code::PermissionDenied,
            });
        }
        if self.owner_id == user.id {
            return Err(BecoError {
                message: "Cannot remove owner".into(),
                status: Code::PermissionDenied,
            });
        }
        if !self.viewers.iter().any(|usr| usr == &user) {
            return Err(BecoError {
                message: "User is not a viewer".into(),
                status: Code::NotFound,
            });
        }
        self.viewers = self
            .viewers
            .iter()
            .cloned()
            .filter(|usr| usr != &user)
            .collect();
        Ok(())
    }

    pub fn remove_editor(
        &mut self,
        user: PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(BecoError {
                message: "User does not have permission to remove an editor".into(),
                status: Code::PermissionDenied,
            });
        }
        if self.owner_id == user.id {
            return Err(BecoError {
                message: "Cannot remove owner".into(),
                status: Code::PermissionDenied,
            });
        }
        if !self.editors.iter().any(|usr| usr == &user) {
            return Err(BecoError {
                message: "User is not an editor".into(),
                status: Code::NotFound,
            });
        }
        self.editors = self
            .editors
            .iter()
            .cloned()
            .filter(|usr| usr != &user)
            .collect();
        Ok(())
    }

    pub fn is_owner_or_editor(this: &Self, user: &PublicUser) -> bool {
        user.id == this.owner_id || this.editors.iter().find(|&usr| usr == user).is_some()
    }

    pub fn is_owner_or_viewer(this: &Self, user: &PublicUser) -> bool {
        user.id == this.owner_id || this.viewers.iter().find(|&usr| usr == user).is_some()
    }
}
