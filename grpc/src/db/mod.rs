mod queries;

use deadpool_postgres::{Config, Object, Pool, PoolConfig, Runtime};
use serde_json::Value;
use tokio_postgres::{NoTls, Statement};
use tonic::Code;
use uuid::Uuid;

use crate::{errors::BecoError, user::user::User};

pub struct DB {
    pool: Pool,
}

impl DB {
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

        Self { pool }
    }

    pub async fn load_user(&self, user_id: &String) -> Result<User, BecoError> {
        let client = self.pool.get().await.unwrap();
        self.load_user_using_client(user_id, &client).await
    }

    pub async fn load_user_using_client(
        &self,
        user_id: &String,
        client: &Object,
    ) -> Result<User, BecoError> {
        let user_uuid = Uuid::parse_str(&user_id).unwrap();
        // let client = self.pool.get().await.unwrap();
        let select_user_statement = client.prepare_cached(queries::user::SELECT).await.unwrap();
        let row_result = client
            .query_one(&select_user_statement, &[&user_uuid])
            .await;
        if row_result.is_err() {
            return Err(row_result.unwrap_err().into());
        }
        let row = row_result.unwrap();
        Ok(serde_json::from_value(row.get("details")).unwrap())
    }

    pub async fn load_merkle(&self, user_id: &String) -> Result<Vec<[u8; 32]>, BecoError> {
        let client = self.pool.get().await.unwrap();
        self.load_merkle_with_client(user_id, &client).await
    }

    pub async fn load_merkle_with_client(
        &self,
        user_id: &String,
        client: &Object,
    ) -> Result<Vec<[u8; 32]>, BecoError> {
        let select_merkle_statement = client
            .prepare_cached(queries::merkle::SELECT)
            .await
            .unwrap();
        let row_result = client
            .query_one(&select_merkle_statement, &[&user_id])
            .await;
        if row_result.is_err() {
            return Err(row_result.unwrap_err().into());
        }
        let row = row_result.unwrap();
        let leaves: Vec<&[u8]> = row.get::<&str, Vec<&[u8]>>("leaves");
        let merkle_leaves: Vec<[u8; 32]> = leaves
            .iter()
            .map(|&leaf| leaf[0..32].try_into().unwrap())
            .collect();
        Ok(merkle_leaves)
    }

    pub async fn save_user_request_and_merkle(
        &self,
        user_id: &String,
        serialised_user: &Value,
        serialised_transaction: &Value,
        sequence: i64,
        root: &String,
        leaves: Vec<[u8; 32]>,
    ) -> Result<(), BecoError> {
        let mut client = self.pool.get().await.unwrap();

        // prepare statements

        let queries = vec![
            queries::user::INSERT,
            queries::user::UPDATE,
            queries::transaction::INSERT,
            queries::leaf::INSERT,
        ];
        let statements = DB::prepare_statements(queries, &client).await;
        let [insert_user_statement, update_user_statement, insert_transaction_statement, insert_leaf_statement] =
            &statements[..]
        else {
            return Err(BecoError {
                message: "Failed to prepare statements".into(),
                status: Code::Internal,
            });
        };

        // check if user exists

        let user_uuid = Uuid::parse_str(user_id).unwrap();
        let existing_user_result = self.load_user_using_client(user_id, &client).await;

        // start transaction

        let db_transaction = client.transaction().await.unwrap();

        // insert or update user

        let user_upsert_result = if existing_user_result.is_err() {
            db_transaction
                .query(
                    insert_user_statement,
                    &[&user_uuid, &serialised_user, &sequence],
                )
                .await
        } else {
            db_transaction
                .query(
                    update_user_statement,
                    &[&user_uuid, &serialised_user, &sequence],
                )
                .await
        };

        if user_upsert_result.is_err() {
            let err = user_upsert_result.unwrap_err();
            return Err(err.into());
        }

        // insert transaction

        let transaction_insert_result = db_transaction
            .query(
                insert_transaction_statement,
                &[&serialised_transaction, &user_uuid, &sequence, &root],
            )
            .await;
        if transaction_insert_result.is_err() {
            return Err(transaction_insert_result.unwrap_err().into());
        }

        let transaction_insert_rows = transaction_insert_result.unwrap();

        if transaction_insert_rows.len() == 0 {
            return Err(BecoError {
                message: "Problem inserting merkle into DB".to_string(),
                status: Code::Internal,
            });
        }
        let row = &transaction_insert_rows[0];
        let transaction_id = row.get::<&str, i64>("id");

        // insert leaf

        let latest_leaf: &[u8] = leaves.last().unwrap();
        let insert_leaf_result = db_transaction
            .query(
                insert_leaf_statement,
                &[&latest_leaf, &user_uuid, &transaction_id],
            )
            .await;

        if insert_leaf_result.is_err() {
            return Err(insert_leaf_result.unwrap_err().into());
        }

        // commit

        let commit_result = db_transaction.commit().await;
        if commit_result.is_err() {
            return Err(commit_result.unwrap_err().into());
        }
        Ok(())
    }

    async fn prepare_statements(queries: Vec<&str>, client: &Object) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];
        for query in queries {
            let statement = client.prepare_cached(query).await.unwrap();
            statements.push(statement);
        }

        statements
    }
}
