// need to implement a simple merkle tree
// each transaction needs to contain the request that was used to modify the state
// needs to include the exact state of the user/organisation
// need to check that the transaction was valid. If invalid, send a RELOAD notification to all grpc nodes
// save the new state to the DB

// This is where we create new users, not in grpc
// incoming request to create a new user with user details
// output is the response to a LOAD command

//

use std::collections::HashMap;

// use k256::sha2::Sha256;
use rs_merkle::{algorithms::Sha256, MerkleTree};

use crate::{proto::beco::AddUserRequest, enums::data_value::ProcessRequest, entry::Entry};

pub struct Orchestrator {
    trees: HashMap<String, MerkleTree<Sha256>>,
    entry: Entry,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            entry: Entry::new(),
        }
    }

    pub fn create_new_user(&mut self, request: AddUserRequest) {
        let user = self.entry.add_user(request);
    }
}
