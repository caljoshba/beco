#![cfg(test)]

// use tokio::sync::mpsc::Sender;
// use uuid::Uuid;

// use crate::{enums::blockchain::Blockchain, user::{user::User, public_user::PublicUser}};

// #[test]
// fn create_new_user() {
//     let user: User = User::new(Some("hjdsj-98d8-oops".into(), Sender::));
//     let public_user = PublicUser { id: user.id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let accounts = user.get_chain_accounts(Blockchain::XRPL, &public_user);
//     assert_eq!(accounts, vec![]);
// }

// #[test]
// fn add_account() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let public_user = PublicUser { id: user.id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let result = user.add_account(Blockchain::XRPL, "test".into(), &public_user).unwrap();
//     let accounts = user.get_chain_accounts(Blockchain::XRPL, &public_user);
//     let expected_response = vec![WalletResponse {
//         alias: "test".into(),
//         public_key: result.public_key,
//         classic_address: result.classic_address,
//     }];
//     assert_eq!(accounts, expected_response);
// }

// #[test]
// fn add_account_invalid_blockchain() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let public_user = PublicUser { id: user.id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let result = user.add_account(Blockchain::UNSPECIFIED, "test".into(), &public_user);
//     let expected_error = "No blockchain set";
//     assert_eq!(result.unwrap_err().message, expected_error);
// }

// #[test]
// fn add_linked_user() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let id = Uuid::new_v4();
//     let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     user.add_linked_user(&public_user);
//     let linked_users = user.linked_users();
//     assert_eq!(linked_users.len(), 1);
//     assert_eq!(linked_users.get(&id.to_string()).unwrap(), &public_user);
// }

// #[test]
// fn remove_linked_user() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let id = Uuid::new_v4();
//     let calling_user = PublicUser { id: user.id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     user.add_linked_user(&public_user);
//     let linked_users = user.linked_users();

//     assert_eq!(linked_users.len(), 1);
//     assert_eq!(linked_users.get(&id.to_string()).unwrap(), &public_user);
    
//     let result = user.remove_linked_user(&public_user, &calling_user);

//     assert_eq!(result.err(), None);
//     let updated_linked_users = user.linked_users();
//     assert_eq!(updated_linked_users.len(), 0);
//     assert_eq!(updated_linked_users.get(&id.to_string()), None);
// }

// #[test]
// fn remove_linked_user_as_linked_user() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let id = Uuid::new_v4();
//     let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     user.add_linked_user(&public_user);
//     let linked_users = user.linked_users();

//     assert_eq!(linked_users.len(), 1);
//     assert_eq!(linked_users.get(&id.to_string()).unwrap(), &public_user);
    
//     let result = user.remove_linked_user(&public_user, &public_user);

//     assert_eq!(result.err(), None);
//     let updated_linked_users = user.linked_users();
//     assert_eq!(updated_linked_users.len(), 0);
//     assert_eq!(updated_linked_users.get(&id.to_string()), None);
// }

// #[test]
// fn remove_linked_user_invalid_calling_user() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let id = Uuid::new_v4();
//     let calling_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     user.add_linked_user(&public_user);
//     let linked_users = user.linked_users();

//     assert_eq!(linked_users.len(), 1);
//     assert_eq!(linked_users.get(&id.to_string()).unwrap(), &public_user);
    
//     let result = user.remove_linked_user(&public_user, &calling_user);

//     assert_eq!(result.err().unwrap().message, "User does not have permission to remove this linked account");
//     let updated_linked_users = user.linked_users();
//     assert_eq!(updated_linked_users.len(), 1);
//     assert_eq!(updated_linked_users.get(&id.to_string()).unwrap(), &public_user);
// }

// #[test]
// fn remove_linked_user_does_not_exist() {
//     let mut user: User = User::new(Some("hjdsj-98d8-oops".into()));
//     let id = Uuid::new_v4();
//     let calling_user = PublicUser { id: user.id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let public_user = PublicUser { id: id.to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     let non_existant_public_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None, other_names: None, last_name: None, chain_accounts: vec![] };
//     user.add_linked_user(&public_user);
//     let linked_users = user.linked_users();

//     assert_eq!(linked_users.len(), 1);
//     assert_eq!(linked_users.get(&id.to_string()).unwrap(), &public_user);
    
//     let result = user.remove_linked_user(&non_existant_public_user, &calling_user);

//     assert_eq!(result.err().unwrap().message, "User does not exist as linked account");
//     let updated_linked_users = user.linked_users();
//     assert_eq!(updated_linked_users.len(), 1);
//     assert_eq!(updated_linked_users.get(&id.to_string()).unwrap(), &public_user);
// }