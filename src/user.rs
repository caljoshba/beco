use std::collections::HashMap;

use uuid::{Uuid};

use crate::{enums::blockchain::{Blockchain, BlockchainCustody}, keys::ChainCustody, traits::key::Key, response::WalletResponse, errors::key::CreateKeyError};

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    // user_details: UserDetails,
    chain_accounts: HashMap<Blockchain, BlockchainCustody>
}

impl User {
    pub fn new() -> Self {
        let mut chain_accounts: HashMap<Blockchain, BlockchainCustody> = HashMap::new();
        chain_accounts.insert(Blockchain::XRPL, BlockchainCustody::XRPL(ChainCustody::new(Blockchain::XRPL)));
        chain_accounts.insert(Blockchain::EVM, BlockchainCustody::EVM(ChainCustody::new(Blockchain::EVM)));

        let id = Uuid::new_v4();
        println!("User ID: {:?}", id);
        Self {
            id,
            chain_accounts,
        }
    }

    pub fn get_chain_accounts(&self, blockchain: Blockchain) -> Vec<WalletResponse> {
        let accounts_option = self.chain_accounts.get(&blockchain);
        if accounts_option.is_none() {
            return vec![];
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.display(),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.display(),
            // _ => vec![],
        }
    }

    pub fn add_account(&mut self, blockchain: Blockchain, alias: String) -> Result<WalletResponse, CreateKeyError> {
        let accounts_option = self.chain_accounts.get_mut(&blockchain);
        if accounts_option.is_none() {
            return Err(CreateKeyError{ chain: blockchain.clone(), message: "No blockchain set" });
        }
        match accounts_option.unwrap() {
            BlockchainCustody::XRPL(xrpl_accounts) => xrpl_accounts.create(None, alias),
            BlockchainCustody::EVM(evm_accounts) => evm_accounts.create(None, alias),
            // _ => vec![],
        }
    }


}