use std::fmt;

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub username: String,
    pub password: String,
    pub roles: Vec<Role>,
    pub tokens: Option<Tokens>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Retrieves user object from session previously set by the jwt middleware to be available for injection on controllers
impl FromRequest for User {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<User, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let default_user = User {
            id: None,
            email: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            roles: vec![],
            tokens: None,
            created_at: None,
            updated_at: None,
        };

        let user: User = match req.extensions_mut().get::<User>() {
            Some(data) => data.to_owned(),
            None => default_user.clone()
        };

        return std::future::ready(Ok(user));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tokens {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserView {
    pub email: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub roles: Vec<Role>,
}


#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Role {
    User,
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}
