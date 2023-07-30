// use futures::Stream;
// use std::borrow::BorrowMut;
use std::collections::HashMap;
// use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
// use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};

// use crate::store::inventory_server::Inventory;
// use crate::store::{
//     InventoryChangeResponse, InventoryUpdateResponse, Item, ItemIdentifier, PriceChangeRequest,
//     QuantityChangeRequest,
// };

use crate::chain::chain_user::User;
use crate::proto::beco::beco_server::Beco;
use crate::proto::beco::{ListAccountRequest, ListAccountResponse, WalletResponse, AddAccountRequest};
use crate::proto::beco::{AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse};
// use crate::wallet::keys_server::Keys;
// use crate::wallet::{AccountType, AccountListResponse, WalletResponse, AddAccount};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";

#[derive(Debug)]
pub struct BecoImplementation {
    users: Arc<Mutex<HashMap<String, User>>>,
}

#[tonic::async_trait]
impl Beco for BecoImplementation {
    async fn add_user(&self, request: Request<AddUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        unimplemented!()
    }

    async fn list_user(&self, request: Request<ListUserRequest>) -> Result<Response<ListUserResponse>, Status> {
        unimplemented!()
    }

    async fn add_account(&self, request: Request<AddAccountRequest>) -> Result<Response<WalletResponse>, Status> {
        let inner_request = request.into_inner();
        let mut users = self.users.lock().await;
        let user_option = users.get_mut(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = user_option.unwrap();
        let wallet_response_result = user.add_account(inner_request.blockchain.into(), inner_request.alias);
        if let Err(err) = wallet_response_result {
            return Err(Status::already_exists(err.message));
        }
        let wallet_response = wallet_response_result.unwrap();
        Ok(Response::new(wallet_response.into()))
    }

    async fn list_account(&self, request: Request<ListAccountRequest>) -> Result<Response<ListAccountResponse>, Status> {
        let inner_request = request.into_inner();
        let users = self.users.lock().await;
        let user_option: Option<&User> = users.get(&inner_request.user_id);
        if let Some(user) = user_option {
            let wallet_response = user.get_chain_accounts(inner_request.blockchain.into());
            return Ok(Response::new(ListAccountResponse {
                wallets: wallet_response.iter().map(|wallet| wallet.into()).collect(),
                blockchain: inner_request.blockchain,
            }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }
}

impl Default for BecoImplementation {
    fn default() -> Self {
        let user = User::new();
        let mut users: HashMap<String, User> = HashMap::new();
        users.insert(user.id.to_string(), user);
        Self { users: Arc::new(Mutex::new(users)) }
    }
}