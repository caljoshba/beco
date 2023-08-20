use std::fmt;

use crate::enums::blockchain::Blockchain;

#[derive(Debug, Clone)]
pub struct CreateKeyError {
    pub chain: Blockchain,
    pub message: String,
}

impl fmt::Display for CreateKeyError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error creating key on {}: {}", self.chain, self.message)
    }
}