use std::{collections::HashMap, sync::Arc, time::Duration};

use event_listener::Event;
// use event_listener::Event;
use serde_json::Value;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};
use tonic::Code;

use crate::{
    enums::data_value::{DataRequestType, DataRequests, ProcessRequest},
    errors::BecoError,
    proto::beco::{
        AddAccountRequest, AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse,
        ModifyLinkedUserRequest,
    },
    user::{public_user::PublicUser, user::User}, utils::ProposeEvent,
};

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";
const NOT_AUTH: &str = "Not authorised to perform this action";

#[derive(Debug)]
pub struct Entry {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
    tx_p2p: Sender<Value>,
    tx_grpc: Sender<Value>,
    pub rx_grpc: Receiver<Value>,
    pub event_listeners: RwLock<HashMap<u64, ProposeEvent>>,
    pub events: RwLock<HashMap<u64, Event>>,
}

impl Entry {
    pub fn new(tx_p2p: Sender<Value>, tx_grpc: Sender<Value>, rx_grpc: Receiver<Value>) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            tx_p2p,
            tx_grpc,
            rx_grpc,
            event_listeners: RwLock::new(HashMap::new()),
            events: RwLock::new(HashMap::new()),
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
        let wallet_response_result = user.add_account(request, &calling_user).clone();
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

    pub async fn ping_event(&self, hash: &u64) {
        if let Some(event) = self.events.read().await.get(hash) {
            event.notify(1);
        }
    }

    pub async fn create_event(&self, hash: u64) {
        let event = ProposeEvent::new(DataRequestType::PROPOSE, Duration::from_secs(5));
        self.event_listeners.write().await.insert(hash, event);
        self.events.write().await.insert(hash, Event::new());
    }

    pub async fn remove_event(&self, hash: &u64) {
        self.event_listeners.write().await.remove(hash);
        self.events.write().await.remove(&hash);
    }

    pub async fn fail_event(&self, hash: u64) {
        if let Some(event) = self.event_listeners.write().await.get_mut(&hash) {
            event.update(DataRequestType::FAILED);
        }        
    }

    pub async fn success_event(&self, hash: u64) {
        if let Some(event) = self.event_listeners.write().await.get_mut(&hash) {
            event.update(DataRequestType::VALIDATED);
        }
    }

    pub async fn propose(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
        status: DataRequestType,
        hash: u64,
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
            let propose_request = ProcessRequest {
                signatures: vec![],
                status: DataRequestType::VALIDATED,
                request: data_request,
                calling_user: calling_user_id,
                user_id: user_id,
                hash: hash,
            };
            let send_result = self
                .tx_p2p
                .send(serde_json::to_value(&propose_request).unwrap())
                .await;
            return Err(BecoError {
                message: BAD_ACCOUNT.to_string(),
                status: Code::NotFound,
            });
        }
        let result = {
            let read_user = user_option.unwrap().read().await;
            match data_request.clone() {
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
                DataRequests::AddAccount(request) => read_user.propose_account(request, &calling_user),
            }
        };
        
        if result.is_err() && status == DataRequestType::PROPOSE {
            Err(result.unwrap_err())
        } else if result.is_err() && status == DataRequestType::CORROBORATE {
            let propose_request = ProcessRequest {
                signatures: vec!["Me2".into()],
                status: DataRequestType::INVALID,
                request: data_request,
                calling_user: calling_user_id,
                user_id: user_id,
                hash: hash,
            };
            let send_result = self
                .tx_p2p
                .send(serde_json::to_value(&propose_request).unwrap())
                .await;
            Err(result.unwrap_err())
        } else {
            let propose_request = ProcessRequest {
                signatures: vec!["Me".into()],
                status,
                request: data_request,
                calling_user: calling_user_id,
                user_id: user_id,
                hash: hash,
            };
            let result = self
                .tx_p2p
                .send(serde_json::to_value(&propose_request).unwrap())
                .await;
            if let Some(event) = self.events.read().await.get(&hash) {
                event.listen().await;
            }
            let event_listeners = self.event_listeners.read().await;
            let event_option = event_listeners.get(&hash);
            
            let success = if let Some(event) = event_option {
                let result = event.await;
                result == DataRequestType::VALIDATED
            } else { false };
            
            if success {
                Ok(user_option.unwrap().read().await.as_public_user(&calling_user).into())
            } else {
                Err(BecoError {
                    message: "Failed to validate the request within 3 seconds".into(),
                    status: Code::DeadlineExceeded,
                })
            }
        }
    }

    pub async fn update(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
        hash: u64,
    ) -> Result<GetUserResponse, BecoError> {
        let users = self.users.read().await;
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
            DataRequests::AddAccount(request) => write_user.add_account(request, &calling_user),
        };
        if result.is_err() {
            let error = result.unwrap_err();
            Err(error)
        } else {
            Ok(write_user.as_public_user(&calling_user).into())
        }
    }
}
