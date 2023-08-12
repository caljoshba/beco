use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    users: Arc<Mutex<HashMap<String, User>>>,
}

struct RequestUser {
    pub public_user: PublicUser,
    pub calling_user: PublicUser,
}

impl BecoImplementation {

    fn get_public_user<'a>(&'a self, user_option: &Option<&'a User>, user_id: String, calling_user_id: String) -> PublicUser {
        let user_basic = PublicUser::new(user_id, None, None, None);
        let calling_user_basic = PublicUser::new(calling_user_id, None, None, None);
        if let Some(user) = user_option {
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
        self.users.lock().await.insert(user.id.to_string(), user.clone());
        Ok(Response::new(user.into()))
    }

    async fn list_user(&self, request: Request<ListUserRequest>) -> Result<Response<ListUserResponse>, Status> {
        unimplemented!()
    }

    async fn add_account(&self, request: Request<AddAccountRequest>) -> Result<Response<WalletResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.user_id), inner_request.user_id.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let user_option = users.get_mut(&inner_request.user_id);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = user_option.unwrap();
        let wallet_response_result = user.add_account(inner_request.blockchain.into(), inner_request.alias, &request_users.calling_user);
        if let Err(err) = wallet_response_result {
            return Err(Status::already_exists(err.message));
        }
        let wallet_response = wallet_response_result.unwrap();
        Ok(Response::new(wallet_response.into()))
    }

    async fn list_account(&self, request: Request<ListAccountRequest>) -> Result<Response<ListAccountResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.user_id), inner_request.user_id.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let user_option = users.get_mut(&inner_request.user_id);
        if let Some(user) = user_option {
            let wallet_response = user.get_chain_accounts(inner_request.blockchain.into(), &request_users.calling_user);
            return Ok(Response::new(ListAccountResponse {
                wallets: wallet_response.iter().map(|wallet| wallet.into()).collect(),
                blockchain: inner_request.blockchain,
            }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn update_first_name(&self, request: Request<ModifyNameRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let user_option = users.get_mut(&inner_request.calling_user);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = user_option.unwrap();
        let result = user.user_details.first_name.update(Some(inner_request.name), &request_users.calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(GetUserResponse { id: request_users.calling_user.id }))
        }
    }

    async fn update_other_names(&self, request: Request<ModifyOtherNamesRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let user_option = users.get_mut(&inner_request.calling_user);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = user_option.unwrap();
        let result = user.user_details.other_names.update(Some(inner_request.other_names), &request_users.calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(GetUserResponse { id: request_users.calling_user.id }))
        }
    }

    async fn update_last_name(&self, request: Request<ModifyNameRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let user_option = users.get_mut(&inner_request.calling_user);
        if user_option.is_none() {
            return Err(Status::not_found(BAD_ACCOUNT));
        }
        let user = user_option.unwrap();
        let result = user.user_details.last_name.update(Some(inner_request.name), &request_users.calling_user);
        if result.is_err() {
            Err(Status::permission_denied(NOT_AUTH))
        } else {
            Ok(Response::new(GetUserResponse { id: request_users.calling_user.id }))
        }
    }

    async fn add_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let user_option = users.get_mut(&inner_request.calling_user);
        let new_user = PublicUser::new(inner_request.user, None, None, None);
        if let Some(user) = user_option {
            user.add_linked_user(&new_user);
            return Ok(Response::new(GetUserResponse { id: new_user.id }));
        }
        Err(Status::not_found(BAD_ACCOUNT))
    }

    async fn remove_linked_user(&self, request:Request<ModifyLinkedUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let users = &mut self.users.lock().await;
        let request_users = RequestUser { 
            public_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
            calling_user: self.get_public_user(&users.get(&inner_request.calling_user), inner_request.calling_user.clone(), inner_request.calling_user.clone()),
        };
        let remove_user = PublicUser::new(inner_request.user, None, None, None);
        let user_option = users.get_mut(&inner_request.calling_user);
        if let Some(user) = user_option {
            let result = user.remove_linked_user(&remove_user, &request_users.calling_user);
            return if result.is_ok() { Ok(Response::new(GetUserResponse { id: remove_user.id })) } else { Err(Status::permission_denied(result.err().unwrap().message)) };
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