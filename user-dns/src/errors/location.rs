use std::fmt;

use crate::location::Location;

#[derive(Debug, Clone)]
pub struct LocationError {
    original_location: Location,
    new_location: Location,
}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error updating location from: {} to {}", self.original_location, self.new_location)
    }
}