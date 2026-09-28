use mongodb::bson::DateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::errors::ApiError;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Role {
    User,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub session_id: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl User {
    pub fn view(&self) -> UserView {
        UserView {
            id: self.id.to_hex(),
            email: self.email.clone(),
            username: self.username.clone(),
            roles: self.roles.clone(),
            created_at: self.created_at.try_to_rfc3339_string().unwrap_or_default(),
        }
    }
}

/// What the API returns for a user: never the password hash or session.
#[derive(Serialize)]
pub struct UserView {
    pub id: String,
    pub email: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl CreateUser {
    pub fn validate(&self) -> Result<(), ApiError> {
        let username_ok = (3..=32).contains(&self.username.len())
            && self
                .username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
        if !self.email.contains('@') || self.email.len() > 254 {
            Err(ApiError::bad_request("a valid email is required"))
        } else if !username_ok {
            Err(ApiError::bad_request(
                "username must be 3-32 characters: letters, digits, _ . -",
            ))
        } else if !(8..=72).contains(&self.password.len()) {
            // bcrypt only uses the first 72 bytes of a password.
            Err(ApiError::bad_request("password must be 8-72 characters"))
        } else {
            Ok(())
        }
    }
}
