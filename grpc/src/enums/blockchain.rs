use strum::{Display, EnumString};

use crate::{
    chain::chain_custody::{ChainCustody, PublicChainCustody},
    evm::{EVMKey, EVMKeyValues},
    proto::beco::Blockchain as ProtoBlockchain,
    user::public_user::PublicUser,
    xrpl::{XRPLKey, XRPLKeyValues},
};

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Hash)]
pub enum Blockchain {
    #[strum(serialize = "UNSPECIFIED")]
    UNSPECIFIED,
    #[strum(serialize = "XRPL")]
    XRPL,
    #[strum(serialize = "EVM")]
    EVM,
}

impl From<ProtoBlockchain> for Blockchain {
    fn from(value: ProtoBlockchain) -> Self {
        match value {
            ProtoBlockchain::Unspecified => Blockchain::UNSPECIFIED,
            ProtoBlockchain::Xrpl => Blockchain::XRPL,
            ProtoBlockchain::Evm => Blockchain::EVM,
        }
    }
}

impl From<i32> for Blockchain {
    fn from(value: i32) -> Self {
        match value {
            1 => Blockchain::XRPL,
            2 => Blockchain::EVM,
            _ => Blockchain::UNSPECIFIED,
        }
    }
}

impl Into<i32> for Blockchain {
    fn into(self) -> i32 {
        match self {
            Blockchain::UNSPECIFIED => 0,
            Blockchain::XRPL => 1,
            Blockchain::EVM => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockchainCustody {
    XRPL(ChainCustody<XRPLKey, XRPLKeyValues>),
    EVM(ChainCustody<EVMKey, EVMKeyValues>),
}

impl BlockchainCustody {
    pub fn as_public(self, calling_user: &PublicUser) -> PublicChainCustody {
        match self {
            BlockchainCustody::XRPL(account) => account.clone().as_public(calling_user),
            BlockchainCustody::EVM(account) => account.clone().as_public(calling_user),
        }
    }
}
