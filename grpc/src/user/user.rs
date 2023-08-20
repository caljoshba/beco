use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    enums::blockchain::{Blockchain, BlockchainCustody},
    chain::chain_custody::ChainCustody,
    response::WalletResponse,
    errors::{key::CreateKeyError, permission::PermissionError},
    traits::key::Key, user::{public_user::PublicUser, user_details::UserDetails}
};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub user_details: UserDetails,
    sequence: u64,
    chain_accounts: HashMap<Blockchain, BlockchainCustody>,
    linked_users: HashMap<String, PublicUser>,
}

impl Default for User {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Self {
            id: id.clone(),
            user_details: UserDetails::new(id.to_string(), None),
            sequence: 0,
            chain_accounts: User::generate_default_chain_accounts(id.to_string()),
            linked_users: HashMap::new(),
        }
    }
}

impl User {
    fn generate_default_chain_accounts(id: String) -> HashMap<Blockchain, BlockchainCustody> {
        let mut chain_accounts: HashMap<Blockchain, BlockchainCustody> = HashMap::new();
        chain_accounts.insert(Blockchain::XRPL, BlockchainCustody::XRPL(ChainCustody::new(Blockchain::XRPL, id.clone())));
        chain_accounts.insert(Blockchain::EVM, BlockchainCustody::EVM(ChainCustody::new(Blockchain::EVM, id)));
        chain_accounts

    }
    pub fn new(first_name: Option<String>) -> Self {
        let id = Uuid::new_v4();

        Self {
            id: id.clone(),
            chain_accounts: User::generate_default_chain_accounts(id.to_string()),
            sequence: 0,
            user_details: UserDetails::new(id.to_string(), first_name),
            linked_users: HashMap::new(),
        }
    }

    pub fn increase_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence.clone()
    }

    pub fn sequence(&self) -> u64 {
        self.sequence.clone()
    }

    pub fn as_public_user(&self, calling_user: &PublicUser) -> PublicUser {
        self.user_details.as_public_user(calling_user)
    }

    pub fn user_details_mut(&mut self, calling_user: &PublicUser) -> Result<&mut UserDetails, PermissionError> {
        if calling_user.id != self.id.to_string() {
            return Err(PermissionError { key: "Get mutable User Details", message: "User does not have permission to update user details" });
        }
        Ok(&mut self.user_details)
    }

    pub fn get_chain_accounts(&self, blockchain: Blockchain, public_user: &PublicUser) -> Vec<WalletResponse> {
        let accounts_option = self.chain_accounts.get(&blockchain);
        if accounts_option.is_none() {
            return vec![];
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.display(public_user),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.display(public_user),
        }
    }

    pub fn add_account(&mut self, blockchain: Blockchain, alias: String, public_user: &PublicUser) -> Result<WalletResponse, CreateKeyError> {
        let accounts_option = self.chain_accounts.get_mut(&blockchain);
        if accounts_option.is_none() {
            return Err(CreateKeyError{ chain: blockchain.clone(), message: "No blockchain set".into() });
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.create(None, alias, public_user),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.create(None, alias, public_user),
        }
    }

    pub fn add_linked_user(&mut self, user: &PublicUser) {
        self.linked_users.insert(user.id.clone(), user.clone());
    }

    pub fn linked_users(&self) -> &HashMap<String, PublicUser> {
        &self.linked_users
    }

    pub fn remove_linked_user(&mut self, user: &PublicUser, calling_user: &PublicUser) -> Result<(), PermissionError> {
        if user != calling_user && calling_user.id != self.id.to_string() {
            return Err(PermissionError { key: "remove_linked_account", message: "User does not have permission to remove this linked account" });
        }
        let removed_account = self.linked_users.remove(&user.id);
        if removed_account.is_none() {
            return Err(PermissionError { key: "remove_linked_account", message: "User does not exist as linked account" });
        }
        Ok(())
    }

    pub fn can_access(&self, user: PublicUser) -> bool {
        unimplemented!()
    }


}