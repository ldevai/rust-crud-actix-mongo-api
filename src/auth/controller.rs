use actix_web::{get, HttpResponse, post, Responder, web};

use crate::auth::models::{AuthRequest, TokenRefreshRequest};
use crate::users::models::User;

#[post("/api/auth/login")]
pub async fn login(app_data: web::Data<crate::AppState>, body: web::Json<AuthRequest>) -> impl Responder {
    let result = web::block(move || app_data.auth_service.login(body.into_inner())).await.unwrap();
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        // Err(e) => HttpResponse::BadRequest().json(e.into())
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

#[post("/api/auth/refresh")]
pub async fn refresh(app_data: web::Data<crate::AppState>, body: web::Json<TokenRefreshRequest>) -> impl Responder {
    let result = web::block(move || app_data.auth_service.refresh(&body.into_inner().refresh_token)).await.unwrap();
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        // Err(e) => HttpResponse::BadRequest().json(e.into())
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

/**
Checks the session user set by jwt_middleware and returns OK or BAD_REQUEST
 */
#[get("/api/auth/validate")]
pub async fn validate(current_user: User) -> impl Responder {
    match current_user.id {
        Some(_) => HttpResponse::Ok().finish(),
        None => HttpResponse::BadRequest().finish()
    }
}
