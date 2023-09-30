use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tonic::Code;
use uuid::Uuid;
use std::hash::Hash;

use crate::{
    chain::chain_custody::{ChainCustody, PublicChainCustody},
    enums::blockchain::{Blockchain, BlockchainCustody},
    errors::BecoError,
    proto::beco::AddAccountRequest,
    traits::key::Key,
    user::{public_user::PublicUser, user_details::UserDetails},
};

#[cfg(not(feature = "sst"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub user_details: UserDetails,
    sequence: u64,
    chain_accounts: HashMap<Blockchain, BlockchainCustody>,
    linked_users: HashMap<String, PublicUser>,
}

#[cfg(feature = "sst")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub user_details: UserDetails,
    sequence: u64,
    chain_accounts: HashMap<Blockchain, BlockchainCustody>,
    linked_users: HashMap<String, PublicUser>,
}

impl Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.user_details.hash(state);
        self.sequence.hash(state);
        self.chain_accounts.iter().for_each(|(&blockchain, custody)| {
            blockchain.hash(state);
            custody.hash(state);
        });
        self.linked_users.iter().for_each(|(user_id, public_user)| {
            user_id.hash(state);
            public_user.hash(state);
        });
    }
}

impl User {
    fn generate_default_chain_accounts(id: String) -> HashMap<Blockchain, BlockchainCustody> {
        let mut chain_accounts: HashMap<Blockchain, BlockchainCustody> = HashMap::new();
        chain_accounts.insert(
            Blockchain::XRPL,
            BlockchainCustody::XRPL(ChainCustody::new(Blockchain::XRPL, id.clone())),
        );
        chain_accounts.insert(
            Blockchain::EVM,
            BlockchainCustody::EVM(ChainCustody::new(Blockchain::EVM, id)),
        );
        chain_accounts
    }

    #[cfg(feature = "sst")]
    pub fn new(first_name: Option<String>) -> Self {
        let id = Uuid::new_v4().to_string();

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
        let chain_accounts: Vec<PublicChainCustody> = self
            .chain_accounts
            .iter()
            .map(|(_, chain_account)| chain_account.clone().as_public(calling_user))
            .collect();
        self.user_details
            .as_public_user(calling_user, chain_accounts)
    }

    pub fn add_account(
        &mut self,
        request: AddAccountRequest,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        let blockchain: Blockchain = request.blockchain.into();
        let accounts_option = self.chain_accounts.get_mut(&blockchain);
        if accounts_option.is_none() {
            return Err(BecoError {
                message: format!("No blockchain set: {}", blockchain.clone()),
                status: Code::OutOfRange,
            });
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => {
                xrpl_accounts.create(None, request, calling_user)
            }
            BlockchainCustody::EVM(evm_accounts) => {
                evm_accounts.create(None, request, calling_user)
            }
        }
    }

    pub fn propose_account(
        &self,
        request: AddAccountRequest,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        let blockchain: Blockchain = request.blockchain.into();
        let accounts_option = self.chain_accounts.get(&blockchain);
        if accounts_option.is_none() {
            return Err(BecoError {
                message: format!("No blockchain set: {}", blockchain.clone()),
                status: Code::OutOfRange,
            });
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.propose(request, calling_user),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.propose(request, calling_user),
        }
    }

    pub fn add_linked_user(&mut self, user: &PublicUser) {
        self.linked_users.insert(user.id.clone(), user.clone());
    }

    pub fn linked_users(&self) -> &HashMap<String, PublicUser> {
        &self.linked_users
    }

    pub fn remove_linked_user(
        &mut self,
        user: &PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if user != calling_user && calling_user.id != self.id.to_string() {
            return Err(BecoError {
                message: "User does not have permission to remove this linked account".into(),
                status: Code::PermissionDenied,
            });
        }
        let removed_account = self.linked_users.remove(&user.id);
        if removed_account.is_none() {
            return Err(BecoError {
                message: "User does not exist as linked account".into(),
                status: Code::NotFound,
            });
        }
        Ok(())
    }

    pub fn propose_alter_linked_user(
        &self,
        user: &PublicUser,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        if user != calling_user && calling_user.id != self.id.to_string() {
            return Err(BecoError {
                message: "User does not have permission to alter this linked account".into(),
                status: Code::PermissionDenied,
            });
        }
        let altered_account = self.linked_users.get(&user.id);
        if altered_account.is_none() {
            return Err(BecoError {
                message: "User does not exist as linked account".into(),
                status: Code::NotFound,
            });
        }
        Ok(())
    }

    pub fn can_access(&self, user: PublicUser) -> bool {
        unimplemented!()
    }
}
