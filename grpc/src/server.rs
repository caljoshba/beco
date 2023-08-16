use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::proto::beco::beco_server::Beco;
use crate::proto::beco::{ListAccountRequest, ListAccountResponse, WalletResponse, AddAccountRequest, ModifyLinkedUserRequest, ModifyNameRequest, ModifyOtherNamesRequest};
use crate::proto::beco::{AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse};
use crate::user::{public_user::PublicUser, user::User};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";
const NOT_AUTH: &str = "Not authorised to perform this action";

#[derive(Debug)]
pub struct BecoImplementation {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
}

impl BecoImplementation {

    async fn get_public_user<'a>(&'a self, user_option: &Option<&'a RwLock<User>>, user_id: String, calling_user_id: String) -> PublicUser {
        let user_basic = PublicUser::new(user_id, None, None, None);
        let calling_user_basic = PublicUser::new(calling_user_id, None, None, None);
        if let Some(user_lock) = user_option {
            let user = user_lock.read().await;
            let user_public = user.as_public_user(&calling_user_basic);
            return user_public;
        }
        user_basic
    }
}

#[tonic::async_trait]
impl Beco for BecoImplementation {
    async fn add_user(&self, request: Request<AddUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let data = request.into_inner();
        let user = User::new(data.name);
        let mut users = self.users.write().await;
        users.insert(user.id.to_string(), RwLock::new(user.clone()));
        let calling_user = self.get_public_user(&users.get(&user.id.to_string()), user.id.to_string(), user.id.to_string()).await;
        Ok(Response::new(calling_user.into()))
    }

    async fn list_user(&self, request: Request<ListUserRequest>) -> Result<Response<ListUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let public_user = self.get_public_user(&users.get(&inner_request.user_id), inner_request.user_id.clone(), inner_request.calling_user.clone()).await;
        Ok(Response::new(ListUserResponse{ users: vec![public_user.into()] }))
    }

    async fn add_account(&self, request: Request<AddAccountRequest>) -> Result<Response<WalletResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
        let user_option = users.get(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = &mut user_option.unwrap().write().await;
        let wallet_response_result = user.add_account(inner_request.blockchain.into(), inner_request.alias, &calling_user);
        if let Err(err) = wallet_response_result {
            return Err(Status::already_exists(err.message));
        }
        let wallet_response = wallet_response_result.unwrap();
        Ok(Response::new(wallet_response.into()))
    }

    async fn list_account(&self, request: Request<ListAccountRequest>) -> Result<Response<ListAccountResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
        let user_option = users.get(&inner_request.user_id);
        if let Some(user) = user_option {
            let wallet_response = user.read().await.get_chain_accounts(inner_request.blockchain.into(), &calling_user);
            return Ok(Response::new(ListAccountResponse {
                wallets: wallet_response.iter().map(|wallet| wallet.into()).collect(),
                blockchain: inner_request.blockchain,
            }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn update_first_name(&self, request: Request<ModifyNameRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
        let user_option = users.get(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.first_name.update(Some(inner_request.name), &calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(user.as_public_user(&calling_user).into()))
        }
    }

    async fn update_other_names(&self, request: Request<ModifyOtherNamesRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;

        let user_option = users.get(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.other_names.update(Some(inner_request.other_names), &calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(user.as_public_user(&calling_user).into()))
        }
    }

    async fn update_last_name(&self, request: Request<ModifyNameRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
        let user_option = users.get(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.last_name.update(Some(inner_request.name), &calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(user.as_public_user(&calling_user).into()))
        }
    }

    async fn add_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let user_option = users.get(&inner_request.calling_user);
        let new_user = PublicUser::new(inner_request.user_id, None, None, None);
        if let Some(user) = user_option {
            user.write().await.add_linked_user(&new_user);
            let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
            return Ok(Response::new(calling_user.into()));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn remove_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()).await;
        let remove_user = PublicUser::new(inner_request.user_id.clone(), None, None, None);
        let user_option = users.get(&inner_request.calling_user);
        if let Some(user_lock) = user_option {
            let user = &mut user_lock.write().await;
            let result = user.remove_linked_user(&remove_user, &calling_user);
            return if result.is_ok() { Ok(Response::new(user.as_public_user(&calling_user).into())) } else { Err(Status::permission_denied(result.err().unwrap().message)) };
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }
}

impl Default for BecoImplementation {
    fn default() -> Self {
        let users: HashMap<String, RwLock<User>> = HashMap::new();
        Self { users: Arc::new(RwLock::new(users)) }
    }
}