use std::collections::HashMap;

use crate::{location::Location, errors::location::LocationError};

use super::edge::Edge;

#[derive(Debug)]
pub struct Node {
    value: Option<Location>,
    edges: HashMap<u8, Edge>
}

impl Node {
    pub fn new(value: Option<Location>) -> Self {
        Self { 
            value,
            edges: HashMap::new(),
        }
    }

    pub fn value(&self) -> &Option<Location> {
        &self.value
    }

    pub fn insert(&mut self, query: &[u8], value: Location) -> Option<Location> {
        if query.len() == 0 && self.value.is_none() {
            self.value = Some(value);
            return None;
        } else if query.len() == 0 {
            return self.value.clone();
        }
        let current_value = query[0];
        if let Some(edge) = self.edges.get_mut(&current_value) {
            return edge.insert_node(query, value)
        }
        let mut edge = Edge::new(current_value, None);
        let result = edge.insert_node(query, value);
        self.edges.insert(current_value, edge);
        result
    }

    pub fn update(&mut self, query: &[u8], value: Location) -> Result<Location, LocationError> {
        unimplemented!()
    }
}