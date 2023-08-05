#![cfg(test)]

use crate::{enums::blockchain::Blockchain, chain::chain_user::User, response::WalletResponse};

#[test]
fn create_new_user() {
    let user: User = User::new();
    let accounts = user.get_chain_accounts(Blockchain::XRPL);
    assert_eq!(accounts, vec![]);
}

#[test]
fn add_new_account() {
    let mut user: User = User::new();
    let result = user.add_account(Blockchain::XRPL, "test".into()).unwrap();
    let accounts = user.get_chain_accounts(Blockchain::XRPL);
    let expected_response = vec![WalletResponse {
        alias: "test".into(),
        public_key: result.public_key,
        classic_address: result.classic_address,
    }];
    assert_eq!(accounts, expected_response);
}