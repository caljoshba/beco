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

use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use tokio::sync::RwLock;
use tonic::Code;

use crate::{
    entry::Entry,
    enums::data_value::{DataRequests, ProcessRequest},
    errors::BecoError,
    user::user::User,
};

#[cfg(feature = "sst")]
pub struct SST {
    // hashmap of user id and merkle tree
    trees: HashMap<String, RwLock<MerkleTree<Sha256>>>,
    // entry to perform update and export current state
    entry: Entry,
}

#[cfg(feature = "sst")]
impl SST {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            entry: Entry::new(),
        }
    }

    pub async fn update(
        &mut self,
        process_request: ProcessRequest,
    ) -> Result<User, crate::errors::BecoError> {
        let cloned_process_request = process_request.clone();
        let updated_user_result = match cloned_process_request.request.clone() {
            DataRequests::AddCryptoAccount(data_request) => {
                self.entry
                    .update_value(
                        cloned_process_request.request,
                        cloned_process_request.calling_user,
                        cloned_process_request.user_id,
                    )
                    .await
            }
            DataRequests::FirstName(data_request) => {
                self.entry
                    .update_value(
                        cloned_process_request.request,
                        cloned_process_request.calling_user,
                        cloned_process_request.user_id,
                    )
                    .await
            }
            DataRequests::OtherNames(data_request) => {
                self.entry
                    .update_value(
                        cloned_process_request.request,
                        cloned_process_request.calling_user,
                        cloned_process_request.user_id,
                    )
                    .await
            }
            DataRequests::LastName(data_request) => {
                self.entry
                    .update_value(
                        cloned_process_request.request,
                        cloned_process_request.calling_user,
                        cloned_process_request.user_id,
                    )
                    .await
            }
            DataRequests::AddUser(data_request) => {
                let (user, public_user, _) = self.entry.add_user(data_request).await;
                Ok((user, public_user))
            }
            _ => { Err(BecoError { message: "Not iomplemented".to_string(), status: Code::Unimplemented }) }
        };
        if updated_user_result.is_err() {
            return Err(updated_user_result.unwrap_err());
        }
        // update and save merkle tree
        let (user, _) = updated_user_result.unwrap();
        let merkle_update_result = self.update_merkle_tree(user.clone(), process_request).await;
        if merkle_update_result.is_err() {
            return Err(merkle_update_result.unwrap_err());
        }
        Ok(user)
    }

    async fn update_merkle_tree(
        &mut self,
        user: User,
        process_request: ProcessRequest,
    ) -> Result<(), BecoError> {
        let user_value_result = serde_json::to_string(&user);
        let process_request_value_result = serde_json::to_string(&process_request);
        if user_value_result.is_err() {
            return Err(BecoError {
                message: "Failed to serialize the user".to_string(),
                status: Code::Internal,
            });
        }
        if process_request_value_result.is_err() {
            return Err(BecoError {
                message: "Failed to serialize the request".to_string(),
                status: Code::Internal,
            });
        }
        let user_value = user_value_result.unwrap();
        let process_request_value = process_request_value_result.unwrap();
        if !self.trees.contains_key(&user.id) {
            self.load_merkle_tree_for_user(user.id.clone()).await;
        }
        // now get and update merkle tree
        {
            let tree = self.trees.get_mut(&user.id).unwrap();
            let mut writable_tree = tree.write().await;
            writable_tree.insert(Sha256::hash(user_value.as_bytes()));
            writable_tree.commit();
        }
        self.save_user_request_and_merkle(user, process_request).await
    }

    async fn load_merkle_tree_for_user(&mut self, user_id: String) {
        // should load from DB first but will implement later on
        self.trees.insert(user_id, RwLock::new(MerkleTree::new()));
    }

    async fn save_user_request_and_merkle(&mut self, user: User, process_request: ProcessRequest) -> Result<(), BecoError> {
        Ok(())
    }
}
