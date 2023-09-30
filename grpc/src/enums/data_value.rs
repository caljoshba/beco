use std::{collections::HashSet, hash::Hash};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    proto::beco::{AddAccountRequest, AddUserRequest, ModifyNameRequest, ModifyOtherNamesRequest},
    user::user::User,
};

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq)]
pub enum DataValue {
    #[strum(serialize = "FIRST_NAME")]
    FirstName,
    #[strum(serialize = "OTHER_NAMES")]
    OtherNames,
    #[strum(serialize = "LAST_NAME")]
    LastName,
    #[strum(serialize = "LINKED_USERS")]
    LinkedUsers,
    #[strum(serialize = "CHAIN_ACCOUNTS")]
    ChainAccounts,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessRequest {
    pub validated_signatures: HashSet<String>,
    pub failed_signatures: HashSet<String>,
    pub ignore_signatures: HashSet<String>,
    pub status: DataRequestType,
    pub request: DataRequests,
    pub calling_user: String,
    pub user_id: String,
    pub hash: u64,
    pub datetime: Option<DateTime<Utc>>,
    pub connected_peers: usize,
    pub originator_hash: Option<u64>,
    pub originator_peer_id: Option<String>,
}

impl Hash for ProcessRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.request.hash(state);
        self.datetime.hash(state);
    }
}

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataRequestType {
    #[strum(serialize = "PROPOSE")]
    PROPOSE,
    #[strum(serialize = "CORROBORATE")]
    CORROBORATE,
    #[strum(serialize = "IGNORED")]
    IGNORED,
    #[strum(serialize = "INVALID")]
    INVALID,
    #[strum(serialize = "VALID")]
    VALID,
    #[strum(serialize = "VALIDATED")]
    VALIDATED,
    #[strum(serialize = "FAILED")]
    FAILED,
    #[strum(serialize = "LOAD")]
    LOAD,
    #[strum(serialize = "NOT_FOUND")]
    NOTFOUND,
    #[strum(serialize = "NEW")]
    NEW,
    #[strum(serialize = "RESPONSE")]
    RESPONSE,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub enum DataRequests {
    FirstName(ModifyNameRequest),
    OtherNames(ModifyOtherNamesRequest),
    LastName(ModifyNameRequest),
    AddUser(AddUserRequest),
    LoadUser(User),
    AddCryptoAccount(AddAccountRequest),
    // RemoveLinkedUserRequest(ModifyLinkedUserRequest),
    // AddLinkedUserRequest(ModifyLinkedUserRequest),
    // AddLinkedUser(ModifyLinkedUserRequest),
    // RemoveLinkedUser(ModifyLinkedUserRequest),
}
