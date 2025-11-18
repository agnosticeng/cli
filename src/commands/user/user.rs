use serde::{Deserialize, Serialize};

/// Agnostic User entity
#[derive(Serialize, Deserialize)]
pub struct User {
    id: u8,
    username: String,
    email: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

impl User {
    pub fn id(&self) -> &u8 {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}
