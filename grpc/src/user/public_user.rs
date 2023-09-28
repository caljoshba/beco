use serde::{Serialize, Deserialize};

use crate::{chain::chain_custody::PublicChainCustody, proto::beco::GetUserResponse};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct PublicUser {
    pub id: String,
    pub first_name: Option<String>,
    pub other_names: Option<Vec<String>>,
    pub last_name: Option<String>,
    pub chain_accounts: Vec<PublicChainCustody>,
}

impl PublicUser {
    pub fn new(
        id: String,
        first_name: Option<String>,
        other_names: Option<Vec<String>>,
        last_name: Option<String>,
        chain_accounts: Vec<PublicChainCustody>,
    ) -> Self {
        Self {
            id,
            first_name,
            other_names,
            last_name,
            chain_accounts,
        }
    }
}

impl PartialEq for PublicUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Into<GetUserResponse> for PublicUser {
    fn into(self) -> GetUserResponse {
        GetUserResponse {
            id: self.id.clone(),
            first_name: self.first_name.clone(),
            other_names: if self.other_names.is_none() {
                vec![]
            } else {
                self.other_names.unwrap()
            },
            last_name: self.last_name,
            chain_accounts: self.chain_accounts.iter().map(|chain_accounts| chain_accounts.clone().into()).collect(),
        }
    }
}
