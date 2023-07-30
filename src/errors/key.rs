use std::fmt;

use crate::enums::blockchain::Blockchain;

#[derive(Debug, Clone)]
pub struct CreateKeyError<'a> {
    pub chain: Blockchain,
    pub message: &'a str,
}

impl<'a> fmt::Display for CreateKeyError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error creating key on {}: {}", self.chain, self.message)
    }
}