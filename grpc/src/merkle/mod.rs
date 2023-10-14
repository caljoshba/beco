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

use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio_postgres::{NoTls, Row};
use tonic::Code;
use uuid::Uuid;

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
    pool: Pool,
}

#[cfg(feature = "sst")]
impl SST {
    pub fn new() -> Self {
        let pool_config = PoolConfig::new(10);
        let mut pg_config = Config::new();
        pg_config.dbname = Some("beco".to_string());
        pg_config.host = Some("localhost".to_string());
        pg_config.port = Some(5432);
        pg_config.user = Some("beco".to_string());
        pg_config.password = Some("during".to_string());
        pg_config.pool = Some(pool_config);
        let pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        Self {
            trees: RwLock::new(HashMap::new()),
            entry: Entry::new(),
            pool,
        }
    }

    pub async fn fetch_user(&self, user_id: &String) -> Option<User> {
        let user_option = self.entry.fetch_user(user_id).await;
        if user_option.is_some() {
            return user_option;
        }
        let user_result = self.get_user_from_db(user_id).await;
        if let Ok(user_row) = user_result {
            let user: User = serde_json::from_value(user_row.get("details")).unwrap();
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
            self.load_merkle_tree_for_user(user.id.clone(), sequence - 1)
                .await;
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

    async fn load_merkle_tree_for_user(&self, user_id: String, sequence: u64) {
        // should load from DB first but will implement later on
        let client = self.pool.get().await.unwrap();
        let select_merkle_statement = client
            .prepare_cached(
                "SELECT
        t.merkle_root_hex,
        array_agg(leaves.content) AS leaves
       FROM personal.user u
       INNER JOIN personal.transaction t
       ON t.user_id = u.id
       INNER JOIN (
           SELECT
             content,
             user_id
         FROM
         personal.leaf
         ORDER BY created_at ASC
       ) leaves
       ON leaves.user_id = u.id
       WHERE u.id=$1
       GROUP BY t.merkle_root_hex;",
            )
            .await
            .unwrap();
        let row_result = client
            .query_one(&select_merkle_statement, &[&user_id])
            .await;
        let tree = if row_result.is_err() {
            MerkleTree::new()
        } else {
            let row = row_result.unwrap();
            let leaves: Vec<&[u8]> = row.get("leaves");
            let merkle_leaves: Vec<[u8; 32]> = leaves
                .iter()
                .map(|&leaf| leaf[0..32].try_into().unwrap())
                .collect();
            MerkleTree::from_leaves(&merkle_leaves)
        };
        let mut trees = self.trees.write().await;
        trees.insert(user_id, RwLock::new(tree));
    }

    async fn get_user_from_db(&self, user_id: &String) -> Result<Row, tokio_postgres::Error> {
        let user_uuid = Uuid::parse_str(&user_id).unwrap();
        let client = self.pool.get().await.unwrap();
        let select_user_statement = client
            .prepare_cached("SELECT id, details, sequence_number FROM personal.user WHERE id=$1")
            .await
            .unwrap();
        client.query_one(&select_user_statement, &[&user_uuid]).await
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

        let mut client = self.pool.get().await.unwrap();
        let insert_user_statement = client
            .prepare_cached(
                "INSERT INTO personal.user (id, details, sequence_number) VALUES ($1, $2, $3);",
            )
            .await
            .unwrap();
        let update_user_statement = client
            .prepare_cached("UPDATE personal.user SET details=$2, sequence_number=$3 WHERE id=$1;")
            .await
            .unwrap();
        let insert_transaction_statement = client
            .prepare_cached(
                "INSERT INTO personal.transaction (transaction, user_id, sequence_number, merkle_root_hex) VALUES ($1, $2, $3, $4) RETURNING id;",
            )
            .await
            .unwrap();
        let insert_leaf_statement = client
            .prepare_cached(
                "INSERT INTO personal.leaf (content, user_id, transaction_id) VALUES ($1, $2, $3);",
            )
            .await
            .unwrap();

        if leaves_option.is_none() || root_option.is_none() {
            return Err(BecoError {
                message: "Merkle tree does not exist".to_string(),
                status: Code::NotFound,
            });
        }

        let user_id = Uuid::parse_str(&user.id).unwrap();

        let db_transaction = client.transaction().await.unwrap();
        let existing_user_result = self.get_user_from_db(&user.id).await;
        println!("{existing_user_result:?}");
        let user_upsert_result = if existing_user_result.is_err() {
            db_transaction
                .query(
                    &insert_user_statement,
                    &[&user_id, &serialised_user, &sequence],
                )
                .await
        } else {
            db_transaction
                .query(
                    &update_user_statement,
                    &[&user_id, &serialised_user, &sequence],
                )
                .await
        };

        if user_upsert_result.is_err() {
            let err = user_upsert_result.unwrap_err();
            println!("{err:?}");
            return Err(err.into());
        }

        let leaves = leaves_option.unwrap();
        let root = root_option.unwrap();

        let transaction_insert_result = db_transaction
            .query(
                &insert_transaction_statement,
                &[&serialised_transaction, &user_id, &sequence, &root],
            )
            .await;
        if transaction_insert_result.is_err() {
            return Err(transaction_insert_result.unwrap_err().into());
        }

        let transaction_insert_rows = transaction_insert_result.unwrap();

        if transaction_insert_rows.len() == 0 {
            println!("{transaction_insert_rows:?}");
            return Err(BecoError {
                message: "Problem inserting merkle into DB".to_string(),
                status: Code::Internal,
            });
        }
        let row = &transaction_insert_rows[0];
        let transaction_id = row.get::<&str, i64>("id");

        let latest_leaf: &[u8] = leaves.last().unwrap();
        let insert_leaf_result = db_transaction
            .query(
                &insert_leaf_statement,
                &[&latest_leaf, &user_id, &transaction_id],
            )
            .await;

        if insert_leaf_result.is_err() {
            return Err(insert_leaf_result.unwrap_err().into());
        }

        let commit_result = db_transaction.commit().await;
        if commit_result.is_err() {
            return Err(commit_result.unwrap_err().into());
        }

        Ok(())
    }
}
