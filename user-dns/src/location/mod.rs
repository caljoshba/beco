use std::fmt::Display;

use url::Url;

#[derive(Debug, Clone)]
pub struct Location {
    url: Url,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Location {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}