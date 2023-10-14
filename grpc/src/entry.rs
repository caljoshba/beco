#[cfg(feature = "sst")]
use crate::{
    enums::data_value::DataRequests,
    errors::BecoError,
    proto::beco::{AddAccountRequest, AddUserRequest, GetUserResponse},
    user::{public_user::PublicUser, user::User},
};
#[cfg(not(feature = "sst"))]
use crate::{
    enums::data_value::{DataRequestType, DataRequests, ProcessRequest},
    errors::BecoError,
    proto::beco::{
        AddAccountRequest, AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse,
    },
    user::{public_user::PublicUser, user::User},
    utils::{calculate_hash, ProposeEvent},
};
#[cfg(not(feature = "sst"))]
use event_listener::Event;
#[cfg(not(feature = "sst"))]
use serde_json::Value;
#[cfg(feature = "sst")]
use std::{collections::HashMap, sync::Arc};
#[cfg(not(feature = "sst"))]
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
#[cfg(feature = "sst")]
use tokio::sync::RwLock;
#[cfg(not(feature = "sst"))]
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};
#[cfg(not(feature = "sst"))]
use tonic::Code;
#[cfg(feature = "sst")]
use tonic::Code;

const BAD_BLOCKCHAIN: &str = "Invalid Blockchain value provided";
const BAD_ACCOUNT: &str = "No matching account found";
const NOT_AUTH: &str = "Not authorised to perform this action";

#[cfg(not(feature = "sst"))]
#[derive(Debug)]
pub struct Entry {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
    tx_p2p: Sender<Value>,
    tx_grpc: Sender<Value>,
    pub rx_grpc: Receiver<Value>,
    pub completion_loops: RwLock<HashMap<u64, ProposeEvent>>,
    pub events: RwLock<HashMap<u64, Event>>,
    // add a timeout queue that the p2p side can check - just an Instant::now() thing rather than a future so it can be checked multiple times
}

#[cfg(feature = "sst")]
#[derive(Debug)]
pub struct Entry {
    users: Arc<RwLock<HashMap<String, RwLock<User>>>>,
}

impl Entry {
    #[cfg(not(feature = "sst"))]
    pub fn new(tx_p2p: Sender<Value>, tx_grpc: Sender<Value>, rx_grpc: Receiver<Value>) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            tx_p2p,
            tx_grpc,
            rx_grpc,
            completion_loops: RwLock::new(HashMap::new()),
            events: RwLock::new(HashMap::new()),
        }
    }
    #[cfg(feature = "sst")]
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
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
    #[cfg(not(feature = "sst"))]
    pub async fn add_user(&self, request: AddUserRequest) -> Result<GetUserResponse, BecoError> {
        // let does_calling_user_exist = self
        //     .does_user_exist(request.calling_user.clone(), request.calling_user.clone())
        //     .await;
        // if does_calling_user_exist.is_err() {
        //     return Err(does_calling_user_exist.unwrap_err());
        // }
        let calling_user = {
            let users = self.users.read().await;
            self.get_public_user(
                &users.get(&request.calling_user),
                request.calling_user.clone(),
                request.calling_user.clone(),
            )
            .await
        };
        let data_request = DataRequests::AddUser(request.clone());
        let hash = calculate_hash(&data_request);
        {
            self.create_event(hash.clone(), None).await;
        }
        let process_request = ProcessRequest {
            validated_signatures: HashSet::new(),
            failed_signatures: HashSet::new(),
            ignore_signatures: HashSet::new(),
            status: DataRequestType::NEW,
            request: data_request,
            calling_user: request.calling_user.clone(),
            user_id: "".to_string(),
            hash: hash,
            datetime: None,
            connected_peers: 0,
            originator_hash: Some(hash),
            originator_peer_id: None,
        };
        self.send_message_return_public(&process_request, hash, &calling_user)
            .await
    }

    #[cfg(not(feature = "sst"))]
    async fn send_message_wait(
        &self,
        process_request: &ProcessRequest,
        hash: u64,
    ) -> Result<String, BecoError> {
        let mut user_id: Option<String> = None;
        let _ = self
            .tx_p2p
            .send(serde_json::to_value(&process_request).unwrap())
            .await;
        let success = {
            if let Some(event) = self.events.read().await.get(&hash) {
                event.listen().await;
            }

            let completion_loops = self.completion_loops.read().await;
            let completion_option = completion_loops.get(&hash);
            if let Some(completion) = completion_option {
                let (request_type, user_id_option) = completion.await;
                if let Some(user) = user_id_option.clone() {
                    user_id = Some(user);
                }
                request_type != DataRequestType::FAILED
            } else {
                false
            }
        };

        {
            self.remove_event(&hash).await;
        }
        if !success {
            return Err(BecoError {
                message: "Failed to validate the request within 5 seconds".into(),
                status: Code::DeadlineExceeded,
            });
        }
        if user_id.is_none() {
            return Err(BecoError {
                message: "Invalid user ID".into(),
                status: Code::NotFound,
            });
        }
        Ok(user_id.unwrap())
    }

    #[cfg(not(feature = "sst"))]
    async fn send_message_return_public(
        &self,
        process_request: &ProcessRequest,
        hash: u64,
        calling_user: &PublicUser,
    ) -> Result<GetUserResponse, BecoError> {
        let user_id_result = self.send_message_wait(process_request, hash).await;
        if user_id_result.is_err() {
            return Err(user_id_result.unwrap_err());
        }
        let user_id = user_id_result.unwrap();
        let users = self.users.read().await;
        let user_option = users.get(&user_id);
        let public_user = user_option
            .unwrap()
            .read()
            .await
            .as_public_user(calling_user);
        Ok(public_user.into())
    }

    #[cfg(feature = "sst")]
    pub async fn add_user(&self, request: AddUserRequest) -> Result<(User, PublicUser), BecoError> {
        // need to implement a check to see if they exist first
        // need something like the national insurance number for this
        let user = User::new(Some(request.name));
        let mut users = self.users.write().await;
        users.insert(user.id.to_string(), RwLock::new(user.clone()));
        let calling_user = self
            .get_public_user(
                &users.get(&user.id.to_string()),
                user.id.to_string(),
                user.id.to_string(),
            )
            .await;
        Ok((user, calling_user))
    }

    #[cfg(feature = "sst")]
    pub async fn fetch_user(&self, user_id: &String) -> Option<User> {
        let users = self.users.read().await;
        let user_option = users.get(user_id);
        if let Some(user) = user_option {
            return Some(user.read().await.clone());
        }
        None
    }

    #[cfg(not(feature = "sst"))]
    pub async fn is_user_loaded(&self, user: &User) -> bool {
        let users = &mut self.users.read().await;
        users.get(&user.id).is_some()
    }

    pub async fn load_user(&self, user: User) {
        let users = &mut self.users.write().await;
        users.insert(user.id.clone(), RwLock::new(user));
    }

    #[cfg(not(feature = "sst"))]
    async fn does_user_exist(
        &self,
        user_id: String,
        calling_user_id: String,
    ) -> Result<bool, BecoError> {
        let user_exists_locally = {
            let users_read = self.users.read().await;
            users_read.contains_key(&user_id)
        };
        if user_exists_locally {
            return Ok(true);
        }
        let data_request = DataRequests::FetchUser(ListUserRequest {
            user_id: user_id.clone(),
            calling_user: calling_user_id.clone(),
        });
        let hash = calculate_hash(&data_request);
        {
            self.create_event(hash.clone(), Some(user_id.clone())).await;
        }

        let process_request = ProcessRequest {
            validated_signatures: HashSet::new(),
            failed_signatures: HashSet::new(),
            ignore_signatures: HashSet::new(),
            status: DataRequestType::FETCH,
            request: data_request,
            calling_user: calling_user_id,
            user_id: user_id,
            hash: hash,
            datetime: None,
            connected_peers: 0,
            originator_hash: Some(hash),
            originator_peer_id: None,
        };
        let user_id_result = self.send_message_wait(&process_request, hash).await;
        if user_id_result.is_err() {
            return Err(user_id_result.unwrap_err());
        }
        Ok(true)
    }

    #[cfg(feature = "sst")]
    async fn does_user_exist(
        &self,
        user_id: String,
        calling_user_id: String,
    ) -> Result<bool, BecoError> {
        let user_exists_locally = {
            let users_read = self.users.read().await;
            users_read.contains_key(&user_id)
        };
        if user_exists_locally {
            return Ok(true);
        }
        // let data_request = DataRequests::LoadUser(None);
        // let hash = calculate_hash(&data_request);
        // {
        //     self.create_event(hash.clone(), Some(user_id.clone())).await;
        // }

        // let process_request = ProcessRequest {
        //     validated_signatures: HashSet::new(),
        //     failed_signatures: HashSet::new(),
        //     ignore_signatures: HashSet::new(),
        //     status: DataRequestType::FETCH,
        //     request: data_request,
        //     calling_user: calling_user_id,
        //     user_id: user_id,
        //     hash: hash,
        //     datetime: None,
        //     connected_peers: 0,
        //     originator_hash: Some(hash),
        //     originator_peer_id: None,
        // };
        // let user_id_result = self.send_message_wait(&process_request, hash).await;
        // at this point, load from DB
        let user_id_result = Ok(());
        if user_id_result.is_err() {
            return Err(user_id_result.unwrap_err());
        }
        Ok(true)
    }

    #[cfg(not(feature = "sst"))]
    pub async fn list_user(&self, request: ListUserRequest) -> Result<ListUserResponse, BecoError> {
        let does_user_exist = self
            .does_user_exist(request.user_id.clone(), request.calling_user.clone())
            .await;
        if does_user_exist.is_err() {
            return Err(does_user_exist.unwrap_err());
        }
        let users = &mut self.users.read().await;
        let public_user = self
            .get_public_user(
                &users.get(&request.user_id),
                request.user_id.clone(),
                request.calling_user.clone(),
            )
            .await;
        Ok(ListUserResponse {
            users: vec![public_user.into()],
        })
    }

    // should be a proposal - pass in as param
    pub async fn add_account(
        &self,
        request: AddAccountRequest,
    ) -> Result<GetUserResponse, BecoError> {
        let does_user_exist = self
            .does_user_exist(request.user_id.clone(), request.calling_user.clone())
            .await;
        if does_user_exist.is_err() {
            return Err(does_user_exist.unwrap_err());
        }
        let does_calling_user_exist = self
            .does_user_exist(request.calling_user.clone(), request.calling_user.clone())
            .await;
        if does_calling_user_exist.is_err() {
            return Err(does_calling_user_exist.unwrap_err());
        }

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

    // pub async fn add_linked_user(
    //     &self,
    //     request: ModifyLinkedUserRequest,
    // ) -> Result<GetUserResponse, BecoError> {
    //     let users = &mut self.users.read().await;
    //     let user_option = users.get(&request.calling_user);
    //     let new_user = PublicUser::new(request.user_id, None, None, None, vec![]);
    //     if let Some(user) = user_option {
    //         user.write().await.add_linked_user(&new_user);
    //         let calling_user = self
    //             .get_public_user(
    //                 &users.get(&request.calling_user),
    //                 request.calling_user.clone(),
    //                 request.calling_user.clone(),
    //             )
    //             .await;
    //         return Ok(calling_user.into());
    //     }
    //     Err(BecoError {
    //         message: BAD_ACCOUNT.to_string(),
    //         status: Code::NotFound,
    //     })
    // }

    // pub async fn remove_linked_user(
    //     &self,
    //     request: ModifyLinkedUserRequest,
    // ) -> Result<GetUserResponse, BecoError> {
    //     let users = &mut self.users.read().await;
    //     let calling_user = self
    //         .get_public_user(
    //             &users.get(&request.calling_user),
    //             request.calling_user.clone(),
    //             request.calling_user.clone(),
    //         )
    //         .await;
    //     let remove_user = PublicUser::new(request.user_id.clone(), None, None, None, vec![]);
    //     let user_option = users.get(&request.calling_user);
    //     if let Some(user_lock) = user_option {
    //         let user = &mut user_lock.write().await;
    //         let result = user.remove_linked_user(&remove_user, &calling_user);
    //         return if result.is_ok() {
    //             Ok(user.as_public_user(&calling_user).into())
    //         } else {
    //             Err(BecoError {
    //                 message: result.err().unwrap().message.to_string(),
    //                 status: Code::PermissionDenied,
    //             })
    //         };
    //     }
    //     Err(BecoError {
    //         message: BAD_ACCOUNT.to_string(),
    //         status: Code::NotFound,
    //     })
    // }

    #[cfg(not(feature = "sst"))]
    pub async fn ping_event(&self, hash: &u64) {
        if let Some(event) = self.events.read().await.get(hash) {
            event.notify(1);
        }
    }
    #[cfg(not(feature = "sst"))]
    pub async fn create_event(&self, hash: u64, user_id: Option<String>) {
        let event = ProposeEvent::new(DataRequestType::PROPOSE, user_id, Duration::from_secs(5));
        self.completion_loops.write().await.insert(hash, event);
        self.events.write().await.insert(hash, Event::new());
    }
    #[cfg(not(feature = "sst"))]
    pub async fn remove_event(&self, hash: &u64) {
        self.completion_loops.write().await.remove(hash);
        self.events.write().await.remove(&hash);
    }
    #[cfg(not(feature = "sst"))]
    pub async fn fail_event(&self, hash: u64, user_id: Option<String>) {
        if let Some(event) = self.completion_loops.write().await.get_mut(&hash) {
            event.update(DataRequestType::FAILED, user_id);
        }
    }
    #[cfg(not(feature = "sst"))]
    pub async fn success_event(&self, hash: u64, user_id: Option<String>, status: DataRequestType) {
        if let Some(event) = self.completion_loops.write().await.get_mut(&hash) {
            event.update(status, user_id);
        }
    }
    #[cfg(not(feature = "sst"))]
    pub async fn does_event_exist(&self, hash: u64) -> bool {
        let completion_loops = self.completion_loops.read().await;
        completion_loops.get(&hash).is_some()
    }
    #[cfg(not(feature = "sst"))]
    async fn propose_value(
        &self,
        user_option: &Option<&RwLock<User>>,
        request: DataRequests,
        calling_user: &PublicUser,
    ) -> Result<(), BecoError> {
        let read_user = user_option.unwrap().read().await;
        match request {
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
            DataRequests::AddCryptoAccount(request) => {
                read_user.propose_account(request, &calling_user)
            }
            DataRequests::AddUser(_) | DataRequests::LoadUser(_) | DataRequests::FetchUser(_) => {
                Err(BecoError {
                    message: "Invalid path to perform action".to_string(),
                    status: Code::Internal,
                })
            }
        }
    }

    #[cfg(not(feature = "sst"))]
    pub async fn corroborate(&self, request: &mut ProcessRequest) {
        let users = &self.users.read().await;
        let calling_user = self
            .get_public_user(
                &users.get(&request.calling_user),
                request.calling_user.clone(),
                request.calling_user.clone(),
            )
            .await;
        let user = self
            .get_public_user(
                &users.get(&request.user_id),
                request.user_id.clone(),
                request.calling_user.clone(),
            )
            .await;
        let user_option = users.get(&request.user_id);
        if user_option.is_none() {
            request.status = DataRequestType::IGNORED;
            let send_result = self
                .tx_p2p
                .send(serde_json::to_value(&request).unwrap())
                .await;
            return;
        }
        let result = self
            .propose_value(&user_option, request.request.clone(), &calling_user)
            .await;

        if result.is_err() {
            request.status = DataRequestType::INVALID;
            let send_result = self
                .tx_p2p
                .send(serde_json::to_value(&request).unwrap())
                .await;
        } else {
            request.status = DataRequestType::VALID;
            let send_result = self
                .tx_p2p
                .send(serde_json::to_value(&request).unwrap())
                .await;
        }
    }
    #[cfg(not(feature = "sst"))]
    pub async fn propose(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
    ) -> Result<GetUserResponse, BecoError> {
        let does_user_exist = self
            .does_user_exist(user_id.clone(), calling_user_id.clone())
            .await;
        if does_user_exist.is_err() {
            return Err(does_user_exist.unwrap_err());
        }
        let does_calling_user_exist = self
            .does_user_exist(calling_user_id.clone(), calling_user_id.clone())
            .await;
        if does_calling_user_exist.is_err() {
            return Err(does_calling_user_exist.unwrap_err());
        }
        let (calling_user, result) = {
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

            let result: Result<(), BecoError> = if user_option.is_none() {
                return Err(BecoError {
                    message: BAD_ACCOUNT.to_string(),
                    status: Code::NotFound,
                });
            } else {
                self.propose_value(&user_option, data_request.clone(), &calling_user)
                    .await
            };

            (calling_user, result)
        };

        if result.is_err() {
            Err(result.unwrap_err())
        } else {
            let hash = calculate_hash(&data_request);
            {
                self.create_event(hash.clone(), Some(user_id.clone())).await;
            }

            let process_request = ProcessRequest {
                validated_signatures: HashSet::new(),
                failed_signatures: HashSet::new(),
                ignore_signatures: HashSet::new(),
                status: DataRequestType::PROPOSE,
                request: data_request,
                calling_user: calling_user_id,
                user_id: user_id,
                hash: hash,
                datetime: None,
                connected_peers: 0,
                originator_hash: None,
                originator_peer_id: None,
            };
            self.send_message_return_public(&process_request, hash, &calling_user)
                .await
        }
    }

    #[cfg(not(feature = "sst"))]
    pub async fn update(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
    ) -> Result<GetUserResponse, BecoError> {
        let result = self
            .update_value(data_request, calling_user_id, user_id)
            .await;
        if result.is_err() {
            let error = result.unwrap_err();
            Err(error)
        } else {
            let (user, calling_user) = result.unwrap();
            Ok(user.as_public_user(&calling_user).into())
        }
    }

    pub async fn update_value(
        &self,
        data_request: DataRequests,
        calling_user_id: String,
        user_id: String,
    ) -> Result<(User, PublicUser), BecoError> {
        let does_user_exist = self
            .does_user_exist(user_id.clone(), calling_user_id.clone())
            .await;
        if does_user_exist.is_err() {
            return Err(does_user_exist.unwrap_err());
        }
        let does_calling_user_exist = self
            .does_user_exist(calling_user_id.clone(), calling_user_id.clone())
            .await;
        if does_calling_user_exist.is_err() {
            return Err(does_calling_user_exist.unwrap_err());
        }
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
            DataRequests::AddCryptoAccount(request) => {
                write_user.add_account(request, &calling_user)
            }
            _ => Ok(()),
        };
        if result.is_err() {
            let error = result.unwrap_err();
            Err(error)
        } else {
            write_user.increase_sequence();
            Ok((write_user.clone(), calling_user))
        }
    }
}
