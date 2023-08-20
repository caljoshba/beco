use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tonic::Code;

use crate::{user::{user::User, public_user::PublicUser}, proto::beco::{ListUserRequest, ListUserResponse, AddAccountRequest, GetUserResponse, AddUserRequest, ListAccountResponse, ListAccountRequest, ModifyNameRequest, ModifyOtherNamesRequest, ModifyLinkedUserRequest}, errors::user::UserError, response::WalletResponse, enums::permission_model_operation::PermissionModelOperation};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";
const NOT_AUTH: &str = "Not authorised to perform this action";

#[derive(Debug)]
pub struct Entry {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
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

    pub async fn add_user(&self, request: AddUserRequest) -> GetUserResponse {
        let user = User::new(request.name);
        let mut users = self.users.write().await;
        users.insert(user.id.to_string(), RwLock::new(user.clone()));
        let calling_user = self.get_public_user(&users.get(&user.id.to_string()), user.id.to_string(), user.id.to_string()).await;
        calling_user.into()
    }

    pub async fn list_user(&self, request: ListUserRequest) -> ListUserResponse {
        let users = &mut self.users.read().await;
        let public_user = self.get_public_user(&users.get(&request.user_id), request.user_id.clone(), request.calling_user.clone()).await;
        ListUserResponse{ users: vec![public_user.into()] }
    }

    // should be a proposal - pass in as param
    pub async fn add_account(&self, request: AddAccountRequest, operation: PermissionModelOperation) -> Result<WalletResponse, UserError> {
        let users = &self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            return Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound });
        }
        let user = &mut user_option.unwrap().write().await;
        let wallet_response_result = user.add_account(request.blockchain.into(), request.alias, &calling_user).clone();
        if let Err(err) = wallet_response_result {
            let message = err.message.clone();
            return Err(UserError{ message, status: Code::InvalidArgument });
        }
        Ok(wallet_response_result.unwrap())
    }

    pub async fn list_account(&self, request: ListAccountRequest) -> Result<ListAccountResponse, UserError> {
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let user_option = users.get(&request.user_id);
        if let Some(user) = user_option {
            let wallet_response = user.read().await.get_chain_accounts(request.blockchain.into(), &calling_user);
            return Ok(ListAccountResponse {
                wallets: wallet_response.iter().map(|wallet| wallet.into()).collect(),
                blockchain: request.blockchain,
            });
        }
        Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound })
    }

    pub async fn update_first_name(&self, request: ModifyNameRequest, operation: PermissionModelOperation) -> Result<GetUserResponse, UserError> {
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            return Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound });
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.first_name.update(Some(request.name), &calling_user, operation).await;
        if result.is_err() {
            Err(UserError{ message: NOT_AUTH.to_string(), status: Code::PermissionDenied })
        } else {
            Ok(user.as_public_user(&calling_user).into())
        }
    }

    pub async fn update_other_names(&self, request: ModifyOtherNamesRequest, operation: PermissionModelOperation) -> Result<GetUserResponse, UserError> {
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            return Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound });
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.other_names.update(Some(request.other_names), &calling_user, operation).await;
        if result.is_err() {
            Err(UserError{ message: NOT_AUTH.to_string(), status: Code::PermissionDenied })
        } else {
            Ok(user.as_public_user(&calling_user).into())
        }
    }

    pub async fn update_last_name(&self, request: ModifyNameRequest, operation: PermissionModelOperation) -> Result<GetUserResponse, UserError> {
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            return Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound });
        }
        let user = &mut user_option.unwrap().write().await;
        let result = user.user_details.last_name.update(Some(request.name), &calling_user, operation).await;
        if result.is_err() {
            Err(UserError{ message: NOT_AUTH.to_string(), status: Code::PermissionDenied })
        } else {
            Ok(user.as_public_user(&calling_user).into())
        }
    }

    pub async fn add_linked_user(&self, request: ModifyLinkedUserRequest, operation: PermissionModelOperation) -> Result<GetUserResponse, UserError> {
        let users = &mut self.users.read().await;
        let user_option = users.get(&request.calling_user);
        let new_user = PublicUser::new(request.user_id, None, None, None);
        if let Some(user) = user_option {
            user.write().await.add_linked_user(&new_user);
            let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
            return Ok(calling_user.into());
        }
        Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound })
    }

    pub async fn remove_linked_user(&self, request:ModifyLinkedUserRequest, operation: PermissionModelOperation) -> Result<GetUserResponse, UserError> {
        let users = &mut self.users.read().await;
        let calling_user = self.get_public_user(&users.get(&request.calling_user), request.calling_user.clone(), request.calling_user.clone()).await;
        let remove_user = PublicUser::new(request.user_id.clone(), None, None, None);
        let user_option = users.get(&request.calling_user);
        if let Some(user_lock) = user_option {
            let user = &mut user_lock.write().await;
            let result = user.remove_linked_user(&remove_user, &calling_user);
            return if result.is_ok() { Ok(user.as_public_user(&calling_user).into()) } else { Err(UserError{ message: result.err().unwrap().message.to_string(), status: Code::PermissionDenied }) };
        }
        Err(UserError{ message: BAD_ACCOUNT.to_string(), status: Code::NotFound })
    }
}