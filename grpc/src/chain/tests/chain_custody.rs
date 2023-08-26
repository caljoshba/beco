#![cfg(test)]

use xrpl::constants::CryptoAlgorithm;

use crate::{
    chain::chain_custody::ChainCustody,
    enums::blockchain::Blockchain,
    traits::key::Key,
    user::public_user::PublicUser,
    xrpl::{XRPLKey, XRPLKeyValues}, proto::beco::{AddAccountRequest, Blockchain as RequestBlockchain},
};

#[test]
fn create_new_keys() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None, vec![]);
    let chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> =
        ChainCustody::new(Blockchain::XRPL, public_user.id);
    assert_eq!(chain_custody.chain, Blockchain::XRPL);
}

#[test]
fn create_new_key() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None, vec![]);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> =
        ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let request = AddAccountRequest { alias: "test".into(), blockchain: RequestBlockchain::Xrpl.into(), calling_user: "".into(), user_id: "".into() };
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), request, &public_user);
    assert_eq!(chain_custody.keys.value(&public_user).unwrap().len(), 1);
}

#[test]
fn does_alias_exist_true() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None, vec![]);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> =
        ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let request = AddAccountRequest { alias: "test".into(), blockchain: RequestBlockchain::Xrpl.into(), calling_user: "".into(), user_id: "".into() };
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), request, &public_user);
    let does_exist = chain_custody.does_alias_exist("test".into(), &public_user);
    assert_eq!(does_exist, true);
}

#[test]
fn does_alias_exist_false() {
    let public_user = PublicUser::new("blah blah blah".into(), None, None, None, vec![]);
    let mut chain_custody: ChainCustody<XRPLKey, XRPLKeyValues> =
        ChainCustody::new(Blockchain::XRPL, public_user.id.clone());
    let request = AddAccountRequest { alias: "test".into(), blockchain: RequestBlockchain::Xrpl.into(), calling_user: "".into(), user_id: "".into() };
    let _ = chain_custody.create(Some(CryptoAlgorithm::ED25519), request, &public_user);
    let does_exist = chain_custody.does_alias_exist("nope".into(), &public_user);
    assert_eq!(does_exist, false);
}
