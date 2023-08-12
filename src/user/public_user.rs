#[derive(Debug, Clone)]
pub struct PublicUser {
    pub id: String,
    pub first_name: Option<String>,
    pub other_names: Option<Vec<String>>,
    pub last_name: Option<String>,
}

impl PublicUser {
    pub fn new(id: String, first_name: Option<String>, other_names: Option<Vec<String>>, last_name: Option<String>) -> Self {
        Self { id, first_name, other_names, last_name }
    }
}

impl PartialEq for PublicUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}