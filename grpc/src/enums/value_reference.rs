use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum ValueReference {
    DETAIL_FIRST_NAME = 1,
    DETAIL_OTHER_NAMES = 2,
    DETAIL_LAST_NAME = 3,
    CHAIN_HEYS = 4,
}