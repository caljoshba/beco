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
use serde_json::Value;
use tokio::sync::RwLock;
use tonic::Code;

use crate::{
    db::DB,
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
    db: DB,
}

#[cfg(feature = "sst")]
impl SST {
    pub fn new() -> Self {
        let db = DB::new();
        Self {
            trees: RwLock::new(HashMap::new()),
            entry: Entry::new(),
            db,
        }
    }

    pub async fn fetch_user(&self, user_id: &String) -> Option<User> {
        let user_option = self.entry.fetch_user(user_id).await;
        if user_option.is_some() {
            return user_option;
        }
        let user_result = self.db.load_user(user_id).await;
        if let Ok(user) = user_result {
            self.entry.load_user(user.clone()).await;
            return Some(user);
        }
        None
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
            let err = merkle_update_result.clone().unwrap_err();
            println!("{err:?}");
            return Err(merkle_update_result.unwrap_err());
        }
        Ok((user, public_user))
    }

    async fn update_merkle_tree(
        &self,
        user: User,
        process_request: ProcessRequest,
    ) -> Result<(), BecoError> {
        let sequence = user.sequence();
        let serialised_user_result = serde_json::to_value(&user);
        let transaction = Transaction {
            user: user.clone(),
            sequence,
            process_request,
        };
        let serialised_transaction_result = serde_json::to_value(&transaction);
        if serialised_user_result.is_err() {
            return Err(BecoError {
                message: "Failed to serialize the user".to_string(),
                status: Code::Internal,
            });
        }
        if serialised_transaction_result.is_err() {
            return Err(BecoError {
                message: "Failed to serialize the transaction".to_string(),
                status: Code::Internal,
            });
        }
        let serialised_user = serialised_user_result.unwrap();
        let serialised_transaction = serialised_transaction_result.unwrap();

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
            writable_tree.insert(Sha256::hash(serialised_transaction.to_string().as_bytes()));
            writable_tree.commit();
        }
        self.save_user_request_and_merkle(user, serialised_transaction, serialised_user)
            .await
    }

    async fn load_merkle_tree_for_user(&self, user_id: String) {
        // should load from DB first but will implement later on
        let tree_result = self.db.load_merkle(&user_id).await;
        let tree = if tree_result.is_err() {
            MerkleTree::new()
        } else {
            let leaves = tree_result.unwrap();
            MerkleTree::from_leaves(&leaves)
        };
        let mut trees = self.trees.write().await;
        trees.insert(user_id, RwLock::new(tree));
    }

    async fn save_user_request_and_merkle(
        &self,
        user: User,
        serialised_transaction: Value,
        serialised_user: Value,
    ) -> Result<(), BecoError> {
        let sequence: i64 = user.sequence().try_into().unwrap();
        let (root_option, leaves_option) = {
            let trees = self.trees.read().await;
            let tree_option = trees.get(&user.id);
            if let Some(tree) = tree_option {
                let read_tree = tree.write().await;
                (read_tree.root_hex(), read_tree.leaves())
            } else {
                (None, None)
            }
        };

        if leaves_option.is_none() || root_option.is_none() {
            return Err(BecoError {
                message: "Merkle tree does not exist".to_string(),
                status: Code::NotFound,
            });
        }

        let leaves = leaves_option.unwrap();
        let root = root_option.unwrap();

        self.db
            .save_user_request_and_merkle(
                &user.id,
                &serialised_user,
                &serialised_transaction,
                sequence,
                &root,
                leaves,
            )
            .await
    }
}
