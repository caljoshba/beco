// need to implement a simple merkle tree
// each transaction needs to contain the request that was used to modify the state
// needs to include the exact state of the user/organisation
// need to check that the transaction was valid. If invalid, send a RELOAD notification to all grpc nodes
// save the new state to the DB

// This is where we create new users, not in grpc
// incoming request to create a new user with user details
// output is the response to a LOAD command

//

mod transaction;

use std::collections::HashMap;

use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use tokio::sync::RwLock;
use tonic::Code;

use crate::{
    entry::Entry,
    enums::data_value::{DataRequests, ProcessRequest},
    errors::BecoError,
    user::{public_user::PublicUser, user::User},
};

use self::transaction::Transaction;

#[cfg(feature = "sst")]
pub struct SST {
    // hashmap of user id and merkle tree
    trees: RwLock<HashMap<String, RwLock<MerkleTree<Sha256>>>>,
    // entry to perform update and export current state
    entry: Entry,
}

#[cfg(feature = "sst")]
impl SST {
    pub fn new() -> Self {
        Self {
            trees: RwLock::new(HashMap::new()),
            entry: Entry::new(),
        }
    }

    pub async fn fetch_user(&self, user_id: &String) -> Option<User> {
        self.entry.fetch_user(user_id).await
    }

    pub async fn update(
        &self,
        process_request: ProcessRequest,
    ) -> Result<(User, PublicUser), crate::errors::BecoError> {
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
            DataRequests::AddUser(data_request) => self.entry.add_user(data_request).await,
            _ => Err(BecoError {
                message: "Not iomplemented".to_string(),
                status: Code::Unimplemented,
            }),
        };
        if updated_user_result.is_err() {
            return Err(updated_user_result.unwrap_err());
        }
        // update and save merkle tree
        let (user, public_user) = updated_user_result.unwrap();
        let merkle_update_result = self.update_merkle_tree(user.clone(), process_request).await;
        if merkle_update_result.is_err() {
            return Err(merkle_update_result.unwrap_err());
        }
        Ok((user, public_user))
    }

    async fn update_merkle_tree(
        &self,
        user: User,
        process_request: ProcessRequest,
    ) -> Result<(), BecoError> {
        // let user_value_result = serde_json::to_string(&user);
        // let process_request_value_result = serde_json::to_string(&process_request);
        let transaction = Transaction {
            user: user.clone(),
            process_request,
        };
        let serialised_transaction_result = serde_json::to_string(&transaction);
        // if user_value_result.is_err() {
        //     return Err(BecoError {
        //         message: "Failed to serialize the user".to_string(),
        //         status: Code::Internal,
        //     });
        // }
        // if process_request_value_result.is_err() {
        //     return Err(BecoError {
        //         message: "Failed to serialize the request".to_string(),
        //         status: Code::Internal,
        //     });
        // }
        if serialised_transaction_result.is_err() {
            return Err(BecoError {
                message: "Failed to serialize the transaction".to_string(),
                status: Code::Internal,
            });
        }
        let serialised_transaction = serialised_transaction_result.unwrap();
        // let user_value = user_value_result.unwrap();
        // let process_request_value = process_request_value_result.unwrap();
        let tree_loaded = {
            let trees = self.trees.read().await;
            trees.contains_key(&user.id)
        };
        if !tree_loaded {
            self.load_merkle_tree_for_user(user.id.clone()).await;
        }
        // now get and update merkle tree
        
        {
            let mut trees = self.trees.write().await;
            let tree = trees.get_mut(&user.id).unwrap();
            let mut writable_tree = tree.write().await;
            writable_tree.insert(Sha256::hash(
                serialised_transaction.as_bytes(),
            ));
            writable_tree.commit();
        }
        self.save_user_request_and_merkle(user, serialised_transaction)
            .await
    }

    async fn load_merkle_tree_for_user(&self, user_id: String) {
        // should load from DB first but will implement later on
        let mut trees = self.trees.write().await;
        trees.insert(user_id, RwLock::new(MerkleTree::new()));
    }

    async fn save_user_request_and_merkle(
        &self,
        user: User,
        serialised_transaction: String,
    ) -> Result<(), BecoError> {
        let leaves_option = {
            let trees = self.trees.read().await;
            let tree_option = trees.get(&user.id);
            if let Some(tree) = tree_option {
                let read_tree = tree.read().await;
                read_tree.leaves()
            } else {
                None
            }
        };

        if let Some(leaves) = leaves_option {
            // leaves is already in bytes, just save to the DB
        }
        Ok(())
    }
}
