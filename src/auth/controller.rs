use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{HttpResponse, get, post, web};

use crate::AppState;
use crate::auth::models::{AuthRequest, AuthResponse, TokenRefreshRequest};
use crate::errors::ApiError;
use crate::security::{ACCESS_TOKEN_TTL, CurrentUser, TOKEN_COOKIE};

#[post("/api/auth/login")]
pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<AuthRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = state.auth_service.login(body.into_inner()).await?;
    Ok(with_token_cookie(response))
}

#[post("/api/auth/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<TokenRefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = state.auth_service.refresh(&body.refresh_token).await?;
    Ok(with_token_cookie(response))
}

/// 200 with the current user when the token (header or cookie) is valid, 401 otherwise.
#[get("/api/auth/validate")]
pub async fn validate(current_user: CurrentUser) -> HttpResponse {
    HttpResponse::Ok().json(current_user.0.view())
}

/// Browsers get the access token as an HttpOnly cookie, out of reach of page scripts;
/// API clients use the same token from the JSON body as a Bearer header.
fn with_token_cookie(response: AuthResponse) -> HttpResponse {
    let cookie = Cookie::build(TOKEN_COOKIE, response.tokens.access_token.clone())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(ACCESS_TOKEN_TTL as i64))
        .finish();
    HttpResponse::Ok().cookie(cookie).json(response)
}
