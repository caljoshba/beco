use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};
use tonic::Code;

use crate::{
    enums::data_value::DataRequests,
    errors::BecoError,
    proto::beco::{
        AddAccountRequest, AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse,
        ModifyLinkedUserRequest,
    },
    user::{public_user::PublicUser, user::User},
};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";
const NOT_AUTH: &str = "Not authorised to perform this action";

#[derive(Debug)]
pub struct Entry {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
    tx_p2p: Sender<Value>,
    rx_p2p: Receiver<Value>,
    tx_grpc: Sender<Value>,
    rx_grpc: Receiver<Value>,
}

impl Entry {
    pub fn new(
        tx_p2p: Sender<Value>,
        rx_p2p: Receiver<Value>,
        tx_grpc: Sender<Value>,
        rx_grpc: Receiver<Value>,
    ) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            tx_p2p,
            rx_p2p,
            tx_grpc,
            rx_grpc,
        }
    }
    async fn get_public_user<'a>(
        &'a self,
        user_option: &Option<&'a RwLock<User>>,
        user_id: String,
        calling_user_id: String,
    ) -> PublicUser {
        let user_basic = PublicUser::new(user_id, None, None, None, vec![]);
        let calling_user_basic = PublicUser::new(calling_user_id, None, None, None, vec![]);
        if let Some(user_lock) = user_option {
            let user = user_lock.read().await;
            let user_public = user.as_public_user(&calling_user_basic);
            return user_public;
        }
        user_basic
    }

    pub async fn add_user(&self, request: AddUserRequest) -> GetUserResponse {
        let user = User::new(request.name, self.tx_p2p.clone(), self.tx_grpc.clone());
        let mut users = self.users.write().await;
        users.insert(user.id.to_string(), RwLock::new(user.clone()));
        let calling_user = self
            .get_public_user(
                &users.get(&user.id.to_string()),
                user.id.to_string(),
                user.id.to_string(),
            )
            .await;
        calling_user.into()
    }

    pub async fn list_user(&self, request: ListUserRequest) -> ListUserResponse {
        let users = &mut self.users.read().await;
        let public_user = self
            .get_public_user(
                &users.get(&request.user_id),
                request.user_id.clone(),
                request.calling_user.clone(),
            )
            .await;
        ListUserResponse {
            users: vec![public_user.into()],
        }
    }

    // should be a proposal - pass in as param
    pub async fn add_account(
        &self,
        request: AddAccountRequest,
    ) -> Result<GetUserResponse, BecoError> {
        let users = &self.users.read().await;
        let calling_user = self
            .get_public_user(
                &users.get(&request.calling_user),
                request.calling_user.clone(),
                request.calling_user.clone(),
            )
            .await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            return Err(BecoError {
                message: BAD_ACCOUNT.to_string(),
                status: Code::NotFound,
            });
        }
        let user = &mut user_option.unwrap().write().await;
        let wallet_response_result = user
            .add_account(request.blockchain.into(), request.alias, &calling_user)
            .clone();
        if let Err(err) = wallet_response_result {
            let message = err.message.clone();
            return Err(BecoError {
                message,
                status: Code::InvalidArgument,
            });
        }
        Ok(user.as_public_user(&calling_user).into())
    }

    pub async fn add_linked_user(
        &self,
        request: ModifyLinkedUserRequest,
    ) -> Result<GetUserResponse, BecoError> {
        let users = &mut self.users.read().await;
        let user_option = users.get(&request.calling_user);
        let new_user = PublicUser::new(request.user_id, None, None, None, vec![]);
        if let Some(user) = user_option {
            user.write().await.add_linked_user(&new_user);
            let calling_user = self
                .get_public_user(
                    &users.get(&request.calling_user),
                    request.calling_user.clone(),
                    request.calling_user.clone(),
                )
                .await;
            return Ok(calling_user.into());
        }
        Err(BecoError {
            message: BAD_ACCOUNT.to_string(),
            status: Code::NotFound,
        })
    }

    pub async fn remove_linked_user(
        &self,
        request: ModifyLinkedUserRequest,
    ) -> Result<GetUserResponse, BecoError> {
        let users = &mut self.users.read().await;
        let calling_user = self
            .get_public_user(
                &users.get(&request.calling_user),
                request.calling_user.clone(),
                request.calling_user.clone(),
            )
            .await;
        let remove_user = PublicUser::new(request.user_id.clone(), None, None, None, vec![]);
        let user_option = users.get(&request.calling_user);
        if let Some(user_lock) = user_option {
            let user = &mut user_lock.write().await;
            let result = user.remove_linked_user(&remove_user, &calling_user);
            return if result.is_ok() {
                Ok(user.as_public_user(&calling_user).into())
            } else {
                Err(BecoError {
                    message: result.err().unwrap().message.to_string(),
                    status: Code::PermissionDenied,
                })
            };
        }
        Err(BecoError {
            message: BAD_ACCOUNT.to_string(),
            status: Code::NotFound,
        })
    }

    pub async fn propose(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
    ) -> Result<GetUserResponse, BecoError> {
        let users = &self.users.read().await;
        let calling_user = self
            .get_public_user(
                &users.get(&calling_user_id),
                calling_user_id.clone(),
                calling_user_id.clone(),
            )
            .await;
        let user = self
            .get_public_user(
                &users.get(&user_id),
                user_id.clone(),
                calling_user_id.clone(),
            )
            .await;
        let user_option = users.get(&user_id);
        if user_option.is_none() {
            return Err(BecoError {
                message: BAD_ACCOUNT.to_string(),
                status: Code::NotFound,
            });
        }
        let read_user = user_option.unwrap().read().await;
        let result = match data_request {
            DataRequests::FirstName(request) => {
                read_user
                    .user_details
                    .first_name
                    .propose(Some(request.name), &calling_user)
                    .await
            }
            DataRequests::OtherNames(request) => {
                read_user
                    .user_details
                    .other_names
                    .propose(Some(request.other_names), &calling_user)
                    .await
            }
            DataRequests::LastName(request) => {
                read_user
                    .user_details
                    .last_name
                    .propose(Some(request.name), &calling_user)
                    .await
            }
        };
        if result.is_err() {
            Err(BecoError {
                message: NOT_AUTH.to_string(),
                status: Code::PermissionDenied,
            })
        } else {
            Ok(read_user.as_public_user(&calling_user).into())
        }
    }

    pub async fn update(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
    ) -> Result<GetUserResponse, BecoError> {
        let users = &mut self.users.read().await;
        let calling_user = self
            .get_public_user(
                &users.get(&calling_user_id),
                calling_user_id.clone(),
                calling_user_id.clone(),
            )
            .await;
        let user = self
            .get_public_user(
                &users.get(&user_id),
                user_id.clone(),
                calling_user_id.clone(),
            )
            .await;
        let user_option = users.get(&user_id);
        if user_option.is_none() {
            return Err(BecoError {
                message: BAD_ACCOUNT.to_string(),
                status: Code::NotFound,
            });
        }
        let write_user = &mut user_option.unwrap().write().await;
        let result = match data_request {
            DataRequests::FirstName(request) => {
                write_user
                    .user_details
                    .first_name
                    .update(Some(request.name), &calling_user)
                    .await
            }
            DataRequests::OtherNames(request) => {
                write_user
                    .user_details
                    .other_names
                    .update(Some(request.other_names), &calling_user)
                    .await
            }
            DataRequests::LastName(request) => {
                write_user
                    .user_details
                    .last_name
                    .update(Some(request.name), &calling_user)
                    .await
            }
        };
        if result.is_err() {
            let error = result.unwrap_err();
            Err(error)
        } else {
            Ok(write_user.as_public_user(&calling_user).into())
        }
    }
}
