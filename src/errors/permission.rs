use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PermissionError<'a> {
    pub key: &'a str,
    pub message: &'a str,
}

impl<'a> fmt::Display for PermissionError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error accessing {}: {}", self.key, self.message)
    }
}