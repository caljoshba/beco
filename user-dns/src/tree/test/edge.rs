#![cfg(test)]

use crate::tree::edge::Edge;

#[test]
fn create_new_edge() {
    let edge = Edge::new(1, None);
    assert_eq!(edge.value(), &1u8);
}