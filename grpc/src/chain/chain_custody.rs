use std::{fmt::Debug, marker::PhantomData};

use tonic::Code;

use crate::{
    enums::{blockchain::Blockchain, value_reference::ValueReference},
    errors::BecoError,
    permissions::model::PermissionModel,
    proto::beco::{AddAccountRequest, ChainResponse, WalletResponse},
    traits::value::Values,
    user::public_user::PublicUser,
};
#[derive(Debug, Clone)]
pub struct ChainCustody<T, A>
where
    T: Values<A> + Clone + Debug + Into<PublicKey>,
{
    pub chain: Blockchain,
    pub keys: PermissionModel<Vec<T>>,
    phantom_type: PhantomData<A>,
}

impl<T, A> ChainCustody<T, A>
where
    T: Values<A> + Clone + Debug + Into<PublicKey>,
{
    pub fn new(chain: Blockchain, owner_id: String) -> Self {
        Self {
            chain,
            keys: PermissionModel::new(owner_id, vec![], "keys".into(), ValueReference::CHAIN_HEYS),
            phantom_type: PhantomData,
        }
    }

    pub fn does_alias_exist(&self, alias: String, user: &PublicUser) -> bool {
        self.keys
            .value(user)
            .unwrap_or(vec![])
            .iter()
            .find(|&key| key.alias() == alias)
            .is_some()
    }

    pub fn propose(
        &self,
        request: AddAccountRequest,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        let does_alias_exist = self.does_alias_exist(request.alias.clone(), calling_user);
        if does_alias_exist {
            return Err(BecoError {
                message: "Alias already exists".into(),
                status: Code::AlreadyExists,
            });
        }
        if !PermissionModel::is_owner_or_editor(&self.keys, calling_user) {
            return Err(BecoError {
                message: "User does not have permission to create a new key".into(),
                status: Code::PermissionDenied,
            });
        }
        Ok(())
    }

    pub fn as_public(&self, calling_user: &PublicUser) -> PublicChainCustody {
        let keys = self.keys.value(calling_user).unwrap_or(vec![]);
        PublicChainCustody {
            chain: self.chain.clone(),
            keys: keys.iter().map(|key| key.clone().into()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublicKey {
    pub alias: String,
    pub address: String,
}

impl Into<WalletResponse> for PublicKey {
    fn into(self) -> WalletResponse {
        WalletResponse { alias: self.alias, address: self.address }
    }
}

#[derive(Debug, Clone)]
pub struct PublicChainCustody {
    pub chain: Blockchain,
    pub keys: Vec<PublicKey>,
}

impl Into<ChainResponse> for PublicChainCustody {
    fn into(self) -> ChainResponse {
        ChainResponse { chain: self.chain.into(), keys: self.keys.iter().map(|key| key.clone().into()).collect() }
    }
}
