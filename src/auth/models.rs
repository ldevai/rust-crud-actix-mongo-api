use serde::{Deserialize, Serialize};

use crate::users::models::Role;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct TokenRefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub email: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub tokens: Tokens,
}

#[derive(Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}
