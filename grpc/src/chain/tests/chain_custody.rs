#![cfg(test)]

use xrpl::constants::CryptoAlgorithm;

use crate::{enums::blockchain::Blockchain, chain::chain_custody::ChainCustody, xrpl::{XRPLKey, XRPLKeyValues}, traits::key::Key, user::public_user::PublicUser};

#[test]
fn create_new_keys() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None);
    let chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL, public_user.id);
    assert_eq!(chain_custody.chain, Blockchain::XRPL);
}

#[test]
fn create_new_key() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into(), &public_user);
    assert_eq!(chain_custody.keys.value(&public_user).unwrap().len(), 1);
}

#[test]
fn does_alias_exist_true() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into(), &public_user);
    let does_exist = chain_custody.does_alias_exist("test".into(), &public_user);
    assert_eq!(does_exist, true);
}

#[test]
fn does_alias_exist_false() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> = ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), "test".into(), &public_user);
    let does_exist = chain_custody.does_alias_exist("nope".into(), &public_user);
    assert_eq!(does_exist, false);
}