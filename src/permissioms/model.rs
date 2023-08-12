use crate::{user::public_user::PublicUser, errors::permission::PermissionError};

#[derive(Debug, Clone)]
pub struct PermissionModel<T> where T: Clone {
    owner_id: String,
    editors: Vec<PublicUser>,
    viewers: Vec<PublicUser>,
    value: T,
    key: String,
}

impl<T> PermissionModel<T>  where T: Clone {
    pub fn new(owner_id: String, value: T, key: String) -> Self {
        Self { owner_id, editors: vec![], viewers: vec![], value, key }
    }
    pub fn value(&self, user: &PublicUser) -> Result<T, PermissionError> {
        if user.id != self.owner_id && self.viewers.iter().find(|&usr| usr == user).is_none() && self.editors.iter().find(|&usr| usr == user).is_none() {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this value" })
        }
        Ok(self.value.clone())
    }

    pub fn value_mut(&mut self, user: &PublicUser) -> Result<&mut T, PermissionError> {
        if user.id != self.owner_id && self.editors.iter().find(|&usr| usr == user).is_none(){
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this mut value" })
        }
        Ok(&mut self.value)
    }

    pub fn update(&mut self, value: T, user: &PublicUser) -> Result<&T, PermissionError> {
        if user.id != self.owner_id && self.editors.iter().find(|&usr| usr == user).is_none(){
            return Err(PermissionError { key: &self.key, message: "User does not have permission to update this value" })
        }
        self.value = value;
        Ok(&self.value)
    }

    pub fn add_viewer(&mut self, user: PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if self.owner_id != calling_user.id && self.editors.iter().find(|&usr| usr == calling_user).is_none() {
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
        if self.owner_id != calling_user.id && self.editors.iter().find(|&usr| usr == calling_user).is_none() {
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
        if self.owner_id != calling_user.id && self.editors.iter().find(|&usr| usr == calling_user).is_none() {
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
        if self.owner_id != calling_user.id && self.editors.iter().find(|&usr| usr == calling_user).is_none() {
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
}