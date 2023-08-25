use strum::{Display, EnumString};

use crate::proto::beco::{ModifyNameRequest, ModifyOtherNamesRequest};

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

pub enum DataRequests {
    FirstName(ModifyNameRequest),
    OtherNames(ModifyOtherNamesRequest),
    LastName(ModifyNameRequest),
    // AddLinkedUser(ModifyLinkedUserRequest),
    // RemoveLinkedUser(ModifyLinkedUserRequest),
}
