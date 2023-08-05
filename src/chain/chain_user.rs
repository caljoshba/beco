use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    enums::blockchain::{Blockchain, BlockchainCustody},
    chain::chain_custody::ChainCustody,
    response::WalletResponse,
    errors::key::CreateKeyError,
    traits::key::Key, user::{public_user::PublicUser, user_details::UserDetails}, proto::beco::GetUserResponse
};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub public_user: PublicUser,
    user_details: UserDetails,
    chain_accounts: HashMap<Blockchain, BlockchainCustody>
}

impl User {
    pub fn new(name: String) -> Self {
        let id = Uuid::new_v4();
        let public_user = PublicUser::new(id.to_string());
        let mut chain_accounts: HashMap<Blockchain, BlockchainCustody> = HashMap::new();
        chain_accounts.insert(Blockchain::XRPL, BlockchainCustody::XRPL(ChainCustody::new(Blockchain::XRPL, public_user.clone())));
        chain_accounts.insert(Blockchain::EVM, BlockchainCustody::EVM(ChainCustody::new(Blockchain::EVM, public_user.clone())));

        Self {
            id,
            public_user: public_user.clone(),
            chain_accounts,
            user_details: UserDetails::new(public_user, name),
        }
    }

    pub fn get_chain_accounts(&self, blockchain: Blockchain, public_user: &PublicUser) -> Vec<WalletResponse> {
        let accounts_option = self.chain_accounts.get(&blockchain);
        if accounts_option.is_none() {
            return vec![];
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.display(public_user),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.display(public_user),
            // _ => vec![],
        }
    }

    pub fn add_account(&mut self, blockchain: Blockchain, alias: String, public_user: &PublicUser) -> Result<WalletResponse, CreateKeyError> {
        let accounts_option = self.chain_accounts.get_mut(&blockchain);
        if accounts_option.is_none() {
            return Err(CreateKeyError{ chain: blockchain.clone(), message: "No blockchain set" });
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.create(None, alias, public_user),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.create(None, alias, public_user),
            // _ => vec![],
        }
    }

    pub fn can_access(&self, user: PublicUser) -> bool {
        unimplemented!()
    }


}

impl Into<GetUserResponse> for User {
    fn into(self) -> GetUserResponse {
        GetUserResponse {
            id: self.id.to_string(),
        }
    }
}