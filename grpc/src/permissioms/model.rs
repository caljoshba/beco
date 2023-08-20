use std::fmt::Debug;

use crate::{user::public_user::PublicUser, errors::permission::PermissionError, enums::{permission_model_operation::PermissionModelOperation, value_reference::ValueReference}};

#[derive(Debug, Clone)]
pub struct PermissionModel<T> where T: Clone + Debug {
    owner_id: String,
    editors: Vec<PublicUser>,
    viewers: Vec<PublicUser>,
    value: T,
    key: String,
    reference: ValueReference,
}

impl<T> PermissionModel<T>  where T: Clone + Debug {
    pub fn new(owner_id: String, value: T, key: String, reference: ValueReference) -> Self {
        Self { owner_id, editors: vec![], viewers: vec![], value, key, reference }
    }
    pub fn value(&self, user: &PublicUser) -> Result<T, PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, user) && !PermissionModel::is_owner_or_viewer(&self, user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this value" })
        }
        Ok(self.value.clone())
    }

    pub fn value_mut(&mut self, user: &PublicUser) -> Result<&mut T, PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this mut value" })
        }
        Ok(&mut self.value)
    }

    pub async fn propose<'a>(this: Self, value: T, user: &PublicUser) -> Result<(), PermissionError<'a>> {
        // this will generate a request that is passed to the p2p server for validation
        // an event listener is attached, waiting for a FAILED or VALIDATED call from p2p server
        // return OK or Err depending on that response 
        // unimplemented!()
        Ok(())
    }

    pub async fn update(&mut self, value: T, calling_user: &PublicUser, operation: PermissionModelOperation) -> Result<&T, PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user){
            return Err(PermissionError { key: &self.key, message: "User does not have permission to update this value" })
        }
        let result = if operation == PermissionModelOperation::PROPOSE {
            let this = self.clone();
            PermissionModel::propose(this, value.clone(), calling_user).await
        } else { Ok(()) };
        if let Err(error) = result {
            return Err(error);
        }
        self.value = value;
        Ok(&self.value)
    }

    pub fn add_viewer(&mut self, user: PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to add a viewer" })
        }
        if self.owner_id == user.id {
            return Err(PermissionError { key: &self.key, message: "User is owner" })
        }
        if self.viewers.iter().find(|&usr| usr == &user).is_some() {
            return Err(PermissionError { key: &self.key, message: "User already has permission to view this value" })
        }
        self.viewers.push(user);
        Ok(())
    }

    pub fn add_editor(&mut self, user: PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to add an editor" })
        }
        if self.owner_id == user.id{
            return Err(PermissionError { key: &self.key, message: "User is owner" })
        }
        if self.editors.iter().find(|&usr| usr == &user).is_some() {
            return Err(PermissionError { key: &self.key, message: "User already has permission to edit this value" })
        }
        self.editors.push(user);
        Ok(())
    }

    pub fn remove_viewer(&mut self, user: PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to remove a viewer" })
        }
        if self.owner_id == user.id {
            return Err(PermissionError { key: &self.key, message: "Cannot remove owner" })
        }
        if !self.viewers.iter().any(|usr| usr == &user) {
            return Err(PermissionError { key: &self.key, message: "User is not a viewer" })
        }
        self.viewers = self.viewers.iter().cloned().filter(|usr| usr != &user).collect();
        Ok(())
    }

    pub fn remove_editor(&mut self, user: PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if !PermissionModel::is_owner_or_editor(&self, calling_user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to remove an editor" })
        }
        if self.owner_id == user.id {
            return Err(PermissionError { key: &self.key, message: "Cannot remove owner" })
        }
        if !self.editors.iter().any(|usr| usr == &user) {
            return Err(PermissionError { key: &self.key, message: "User is not an editor" })
        }
        self.editors = self.editors.iter().cloned().filter(|usr| usr != &user).collect();
        Ok(())
    }

    fn is_owner_or_editor(this: &Self, user: &PublicUser) -> bool {
        user.id == this.owner_id || this.editors.iter().find(|&usr| usr == user).is_some()
    }

    fn is_owner_or_viewer(this: &Self, user: &PublicUser) -> bool {
        user.id == this.owner_id || this.viewers.iter().find(|&usr| usr == user).is_some()
    }
}