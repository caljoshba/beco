use std::fmt;

use tonic::Code;

#[derive(Debug, Clone, PartialEq)]
pub struct BecoError {
    pub message: String,
    pub status: Code,
}

impl fmt::Display for BecoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<tokio_postgres::Error> for BecoError {
    fn from(value: tokio_postgres::Error) -> Self {
        Self { message: value.to_string(), status: Code::Internal }
    }
}