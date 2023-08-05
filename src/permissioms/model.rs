use crate::{user::public_user::PublicUser, errors::permission::PermissionError};

#[derive(Debug, Clone)]
pub struct PermissionModel<T> where T: Clone {
    owner: PublicUser,
    editors: Vec<PublicUser>,
    viewers: Vec<PublicUser>,
    value: T,
    key: String,
}

impl<T> PermissionModel<T>  where T: Clone {
    pub fn new(owner: PublicUser, value: T, key: String) -> Self {
        Self { owner, editors: vec![], viewers: vec![], value, key }
    }
    pub fn value(&self, user: &PublicUser) -> Result<T, PermissionError> {
        if user != &self.owner && self.viewers.iter().find(|&usr| usr == user).is_none() {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this value" })
        }
        Ok(self.value.clone())
    }

    pub fn value_mut(&mut self, user: &PublicUser) -> Result<&mut T, PermissionError> {
        if user != &self.owner {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this value" })
        }
        Ok(&mut self.value)
    }

    pub fn add_viewer(&mut self, user: PublicUser) -> Result<(), PermissionError> {
        if self.owner == user {
            return Err(PermissionError { key: &self.key, message: "User is owner" })
        }
        if self.viewers.iter().find(|&usr| usr == &user).is_some() {
            return Err(PermissionError { key: &self.key, message: "User already has permission to view this value" })
        }
        self.viewers.push(user);
        Ok(())
    }

    pub fn remove_user(&mut self, user: PublicUser) -> Result<(), PermissionError> {
        if self.owner == user {
            return Err(PermissionError { key: &self.key, message: "Cannot remove owner" })
        }
        if !self.viewers.iter().any(|usr| usr == &user) {
            return Err(PermissionError { key: &self.key, message: "User does not have permission to view this value" })
        }
        self.viewers = self.viewers.iter().cloned().filter(|usr| usr != &user).collect();
        Ok(())
    }
}