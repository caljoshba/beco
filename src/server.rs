use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::proto::beco::beco_server::Beco;
use crate::proto::beco::{ListAccountRequest, ListAccountResponse, WalletResponse, AddAccountRequest, ModifyLinkedUserRequest};
use crate::proto::beco::{AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse};
use crate::user::{public_user::PublicUser, user::User};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";

#[derive(Debug)]
pub struct BecoImplementation {
    users: Arc<Mutex<HashMap<String, User>>>,
}

#[tonic::async_trait]
impl Beco for BecoImplementation {
    async fn add_user(&self, request: Request<AddUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let data = request.into_inner();
        let user = User::new(data.name);
        self.users.lock().await.insert(user.id.to_string(), user.clone());
        Ok(Response::new(user.into()))
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
        let calling_user = PublicUser::new(inner_request.calling_user, None);
        let wallet_response_result = user.add_account(inner_request.blockchain.into(), inner_request.alias, &calling_user);
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
        let calling_user = PublicUser::new(inner_request.calling_user, None);
        if let Some(user) = user_option {
            let wallet_response = user.get_chain_accounts(inner_request.blockchain.into(), &calling_user);
            return Ok(Response::new(ListAccountResponse {
                wallets: wallet_response.iter().map(|wallet| wallet.into()).collect(),
                blockchain: inner_request.blockchain,
            }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn add_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let user_option = users.get_mut(&inner_request.calling_user);
        let new_user = PublicUser::new(inner_request.user, None);
        if let Some(user) = user_option {
            user.add_linked_user(&new_user);
            return Ok(Response::new(GetUserResponse { id: new_user.id }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn remove_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let user_option = users.get_mut(&inner_request.calling_user);
        let new_user = PublicUser::new(inner_request.user, None);
        let calling_user = PublicUser::new(inner_request.calling_user, None);
        if let Some(user) = user_option {
            let result = user.remove_linked_user(&new_user, &calling_user);
            return if result.is_ok() { Ok(Response::new(GetUserResponse { id: new_user.id })) } else { Err(Status::permission_denied(result.err().unwrap().message)) };
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }
}

impl Default for BecoImplementation {
    fn default() -> Self {
        let users: HashMap<String, User> = HashMap::new();
        Self { users: Arc::new(Mutex::new(users)) }
    }
}