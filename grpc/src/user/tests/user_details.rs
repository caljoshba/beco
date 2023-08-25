#![cfg(test)]

use uuid::Uuid;

use crate::user::{public_user::PublicUser, user_details::UserDetails};

#[test]
fn create_new_user_details() {
    let id = Uuid::new_v4();
    let user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), None);
}

#[tokio::test]
async fn update_first_name() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), None);

    let first_name: String = "boop".into();

    let result = user_details.first_name.update(Some(first_name.clone()), &public_user).await;

    assert!(result.is_ok());
    assert_eq!(user_details.first_name.value(&public_user).unwrap(), Some(first_name));
}

#[tokio::test]
async fn update_first_name_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.first_name.value(&owner_public_user).unwrap(), None);

    let first_name: String = "boop".into();

    let result = user_details.first_name.update(Some(first_name.clone()), &public_user).await;

    assert_eq!(result.unwrap_err().message, format!("User does not have permission to update this value: first_name"));
    assert_eq!(user_details.first_name.value(&owner_public_user).unwrap(), None);
}

#[tokio::test]
async fn update_other_names() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.other_names.value(&public_user).unwrap(), None);

    let other_names: Vec<String> = vec!["boop".into()];

    let result = user_details.other_names.update(Some(other_names.clone()), &public_user).await;

    assert!(result.is_ok());
    assert_eq!(user_details.other_names.value(&public_user).unwrap(), Some(other_names));
}

#[tokio::test]
async fn update_other_names_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.other_names.value(&owner_public_user).unwrap(), None);

    let other_names: Vec<String> = vec!["boop".into()];

    let result = user_details.other_names.update(Some(other_names.clone()), &public_user).await;

    assert_eq!(result.unwrap_err().message, format!("User does not have permission to update this value: other_names"));
    assert_eq!(user_details.other_names.value(&owner_public_user).unwrap(), None);
}

#[tokio::test]
async fn update_last_name() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.last_name.value(&public_user).unwrap(), None);

    let last_name: String = "boop".into();

    let result = user_details.last_name.update(Some(last_name.clone()), &public_user).await;

    assert!(result.is_ok());
    assert_eq!(user_details.last_name.value(&public_user).unwrap(), Some(last_name));
}

#[tokio::test]
async fn update_last_name_fail_permission() {
    let id = Uuid::new_v4();
    let mut user_details = UserDetails::new(id.to_string(), None);
    let owner_public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    let public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
    
    assert_eq!(user_details.last_name.value(&owner_public_user).unwrap(), None);

    let last_name: String = "boop".into();

    let result = user_details.last_name.update(Some(last_name.clone()), &public_user).await;

    assert_eq!(result.unwrap_err().message, format!("User does not have permission to update this value: last_name"));
    assert_eq!(user_details.last_name.value(&owner_public_user).unwrap(), None);
}