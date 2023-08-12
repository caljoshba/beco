#![cfg(test)]

use uuid::Uuid;

use crate::user::{public_user::PublicUser, user_details::UserDetails};

#[test]
fn create_new_user_details() {
    let id = Uuid::new_v4();
    let user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), None);
}

#[test]
fn update_first_name() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), None);

    let first_name: String = "boop".into();

    let result = user_details.update_first_name(Some(first_name.clone()), &public_user);

    assert!(result.is_ok());
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), Some(first_name));
}

#[test]
fn update_first_name_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.first_name.value(&owner_public_user).unwrap(), None);

    let first_name: String = "boop".into();

    let result = user_details.update_first_name(Some(first_name.clone()), &public_user);

    assert_eq!(result.unwrap_err().message, "User does not have permission to update this value");
    assert_eq!(user_details.first_name.value(&owner_public_user).unwrap(), None);
}

#[test]
fn update_other_names() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.other_names.value(&public_user).unwrap(), None);

    let other_names: Vec<String> = vec!["boop".into()];

    let result = user_details.update_other_names(Some(other_names.clone()), &public_user);

    assert!(result.is_ok());
    assert_eq!(user_details.other_names.value(&public_user).unwrap(), Some(other_names));
}

#[test]
fn update_other_names_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None};
    
    assert_eq!(user_details.other_names.value(&owner_public_user).unwrap(), None);

    let other_names: Vec<String> = vec!["boop".into()];

    let result = user_details.update_other_names(Some(other_names.clone()), &public_user);

    assert_eq!(result.unwrap_err().message, "User does not have permission to update this value");
    assert_eq!(user_details.other_names.value(&owner_public_user).unwrap(), None);
}

#[test]
fn update_last_name() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.last_name.value(&public_user).unwrap(), None);

    let last_name: String = "boop".into();

    let result = user_details.update_last_name(Some(last_name.clone()), &public_user);

    assert!(result.is_ok());
    assert_eq!(user_details.last_name.value(&public_user).unwrap(), Some(last_name));
}

#[test]
fn update_last_name_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None };
    
    assert_eq!(user_details.last_name.value(&owner_public_user).unwrap(), None);

    let last_name: String = "boop".into();

    let result = user_details.update_last_name(Some(last_name.clone()), &public_user);

    assert_eq!(result.unwrap_err().message, "User does not have permission to update this value");
    assert_eq!(user_details.last_name.value(&owner_public_user).unwrap(), None);
}