#![cfg(test)]

use xrpl::constants::CryptoAlgorithm;

use crate::{enums::blockchain::Blockchain, chain::chain_custody::ChainCustody, xrpl::{XRPLKey, XRPLKeyValues}, traits::key::Key};

#[test]
fn create_new_keys() {
    let chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL);
    assert_eq!(chain_custody.chain, Blockchain::XRPL);
}

#[test]
fn create_new_key() {
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL);
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into());
    assert_eq!(chain_custody.keys.len(), 1);
}

#[test]
fn does_alias_exist_true() {
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL);
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into());
    let does_exist = chain_custody.does_alias_exist("test".into());
    assert_eq!(does_exist, true);
}

#[test]
fn does_alias_exist_false() {
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL);
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into());
    let does_exist = chain_custody.does_alias_exist("nope".into());
    assert_eq!(does_exist, false);
}