use serde::{Serialize, Deserialize};

use crate::{enums::data_value::ProcessRequest, user::user::User};

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub user: User,
    pub process_request: ProcessRequest,
}
