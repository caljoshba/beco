use std::hash::{Hash, Hasher};

use crate::{location::Location, errors::location::LocationError};

use super::node::Node;

#[derive(Debug)]
pub struct Edge {
    value: u8,
    child_node: Option<Node>,
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Edge {}

impl Edge {
    pub fn new(value: u8, child_node: Option<Node>) -> Self {
        Self { value, child_node }
    }

    pub fn value(&self) -> &u8 {
        &self.value
    }

    pub fn insert_node(&mut self, query: &[u8], value: Location) -> Option<Location> {
        if let Some(child_node) = &mut self.child_node {
            return child_node.insert(&query[1..], value);
        }
        let mut child_node = Node::new(None);
        let result = child_node.insert(&query[1..], value);
        self.child_node = Some(child_node);
        result
    }

    pub fn update_node(&mut self, query: &[u8], value: Location) -> Result<Location, LocationError> {
        unimplemented!()
    }
}