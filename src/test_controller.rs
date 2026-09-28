use actix_web::{HttpResponse, get};
use serde_json::json;

use crate::errors::ApiError;
use crate::security::CurrentUser;
use crate::users::models::Role;

#[get("/api/public")]
pub async fn public(current_user: Option<CurrentUser>) -> HttpResponse {
    let username = current_user.map(|user| user.0.username);
    HttpResponse::Ok().json(json!({ "username": username, "endpoint_security": "PUBLIC" }))
}

#[get("/api/protected/user")]
pub async fn protected_user(current_user: CurrentUser) -> HttpResponse {
    HttpResponse::Ok()
        .json(json!({ "username": current_user.0.username, "endpoint_security": "ANY ROLE" }))
}

#[get("/api/protected/admin")]
pub async fn protected_admin(current_user: CurrentUser) -> Result<HttpResponse, ApiError> {
    current_user.require(Role::Admin)?;
    Ok(HttpResponse::Ok()
        .json(json!({ "username": current_user.0.username, "endpoint_security": "ROLE_ADMIN" })))
}
