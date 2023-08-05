#![cfg(test)]

use crate::{enums::blockchain::Blockchain, chain::chain_user::User, response::WalletResponse};

#[test]
fn create_new_user() {
    let user: User = User::new("hjdsj-98d8-oops".into());
    let accounts = user.get_chain_accounts(Blockchain::XRPL, &user.public_user);
    assert_eq!(accounts, vec![]);
}

#[test]
fn add_new_account() {
    let mut user: User = User::new("hjdsj-98d8-oops".into());
    let public_user = user.clone().public_user;
    let result = user.add_account(Blockchain::XRPL, "test".into(), &public_user).unwrap();
    let accounts = user.get_chain_accounts(Blockchain::XRPL, &public_user);
    let expected_response = vec![WalletResponse {
        alias: "test".into(),
        public_key: result.public_key,
        classic_address: result.classic_address,
    }];
    assert_eq!(accounts, expected_response);
}