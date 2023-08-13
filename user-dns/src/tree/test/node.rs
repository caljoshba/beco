#![cfg(test)]

use url::Url;

use crate::{tree::node::Node, location::Location};

#[test]
fn create_new_node() {
    let node = Node::new(None);
    assert_eq!(node.value(), &None);
}

#[test]
fn insert_new_location() {
    let key = "blah";
    let url = Url::parse("https://example.com").unwrap();
    let location = Location::new(url);
    let mut node = Node::new(None);
    let result = node.insert(key.as_bytes(), location.clone());
    assert_eq!(result, None);

    let new_url = Url::parse("https://example2.com").unwrap();
    let new_location = Location::new(new_url);
    let re_insert_result = node.insert(key.as_bytes(), new_location);
    assert_eq!(re_insert_result, Some(location));
}