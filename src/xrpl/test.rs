#![cfg(test)]

use crate::enums::blockchain::Blockchain;

use super::Keys;

#[test]
fn create_new_key() {
    let keys = Keys::new();
    assert_eq!(keys.chain, Blockchain::XRPL);
}