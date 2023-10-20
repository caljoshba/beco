use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// use uuid::Uuid;

use crate::enums::oragnisaton_relation::OrganisationRelation;

#[cfg(any(feature = "validator", feature = "sst", feature = "user"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrganisation {
    pub id: String,
    pub relation: OrganisationRelation,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}
