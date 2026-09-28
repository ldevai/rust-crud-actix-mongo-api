use actix_web::{HttpResponse, get, post, web};

use crate::AppState;
use crate::errors::ApiError;
use crate::security::CurrentUser;
use crate::users::models::{CreateUser, Role};

#[post("/api/user/create")]
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateUser>,
) -> Result<HttpResponse, ApiError> {
    let user = state.user_service.create(body.into_inner()).await?;
    Ok(HttpResponse::Created().json(user.view()))
}

/// Users can look themselves up; admins can look up anyone.
#[get("/api/user/{username}")]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<String>,
    current_user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let username = path.into_inner();
    if current_user.0.username != username {
        current_user.require(Role::Admin)?;
    }
    let user = state.user_service.get_by_username(&username).await?;
    Ok(HttpResponse::Ok().json(user.view()))
}
