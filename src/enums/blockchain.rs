use strum::{ Display, EnumString };

use crate::{chain::chain_custody::ChainCustody, xrpl::{XRPLKey, XRPLKeyValues}, evm::{EVMKey, EVMKeyValues}, proto::beco::Blockchain as ProtoBlockchain};

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

#[derive(Debug, Clone)]
pub enum BlockchainCustody {
    XRPL(ChainCustody<XRPLKey, XRPLKeyValues>),
    EVM(ChainCustody<EVMKey, EVMKeyValues>),
}