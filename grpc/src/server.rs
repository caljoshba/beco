use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::entry::Entry;
use crate::enums::data_value::DataRequests;
use crate::proto::beco::beco_server::Beco;
use crate::proto::beco::{
    AddAccountRequest, ModifyLinkedUserRequest, ModifyNameRequest, ModifyOtherNamesRequest,
};
use crate::proto::beco::{AddUserRequest, GetUserResponse, ListUserRequest, ListUserResponse};

#[derive(Debug)]
pub struct BecoImplementation {
    entry: Arc<Entry>,
}

impl BecoImplementation {
    pub fn new(entry: Arc<Entry>) -> Self {
        Self { entry }
    }
}

#[tonic::async_trait]
impl Beco for BecoImplementation {
    async fn add_user(
        &self,
        request: Request<AddUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self.entry.add_user(inner_request).await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self.entry.list_user(inner_request).await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    async fn add_account(
        &self,
        request: Request<AddAccountRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self
            .entry
            .propose(
                DataRequests::AddCryptoAccount(inner_request.clone()),
                inner_request.calling_user.clone(),
                inner_request.user_id.clone(),
            )
            .await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    async fn update_first_name(
        &self,
        request: Request<ModifyNameRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self
            .entry
            .propose(
                DataRequests::FirstName(inner_request.clone()),
                inner_request.calling_user.clone(),
                inner_request.user_id.clone(),
            )
            .await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    async fn update_other_names(
        &self,
        request: Request<ModifyOtherNamesRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self
            .entry
            .propose(
                DataRequests::OtherNames(inner_request.clone()),
                inner_request.calling_user.clone(),
                inner_request.user_id.clone(),
            )
            .await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    async fn update_last_name(
        &self,
        request: Request<ModifyNameRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let inner_request = request.into_inner();
        let result = self
            .entry
            .propose(
                DataRequests::LastName(inner_request.clone()),
                inner_request.calling_user.clone(),
                inner_request.user_id.clone(),
            )
            .await;
        if let Err(err) = result {
            return Err(Status::new(err.status, err.message));
        }
        Ok(Response::new(result.unwrap()))
    }

    // async fn add_linked_user(
    //     &self,
    //     request: Request<ModifyLinkedUserRequest>,
    // ) -> Result<Response<GetUserResponse>, Status> {
    //     let inner_request = request.into_inner();
    //     let result = self.entry.add_linked_user(inner_request).await;
    //     if let Err(err) = result {
    //         return Err(Status::new(err.status, err.message));
    //     }
    //     Ok(Response::new(result.unwrap()))
    // }

    // async fn remove_linked_user(
    //     &self,
    //     request: Request<ModifyLinkedUserRequest>,
    // ) -> Result<Response<GetUserResponse>, Status> {
    //     let inner_request = request.into_inner();
    //     let result = self.entry.remove_linked_user(inner_request).await;
    //     if let Err(err) = result {
    //         return Err(Status::new(err.status, err.message));
    //     }
    //     Ok(Response::new(result.unwrap()))
    // }
}
