use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};


#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Hash, Serialize, Deserialize, Copy)]
pub enum UserRequestType {
    #[strum(serialize = "EMPLOYER")]
    EMPLOYER,
}