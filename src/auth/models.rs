use serde::{Deserialize, Serialize};

use crate::users::models::{Role, Tokens};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub email: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub tokens: Tokens,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenRefreshRequest {
    pub refresh_token: String,
}
