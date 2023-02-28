use actix_web::{get, HttpResponse, post, Responder, web};

use crate::users::models::CreateUser;

#[post("/api/user/create")]
pub async fn create(app_data: web::Data<crate::AppState>, body: web::Json<CreateUser>) -> impl Responder {
    let result = web::block(move || app_data.user_service.create(body.into_inner())).await;
    match result {
        Ok(result) => HttpResponse::Ok().json(result),
        // Err(e) => HttpResponse::BadRequest().json::<GenericError>(e.into())
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

#[get("/api/user/{username}")]
pub async fn get(app_data: web::Data<crate::AppState>, path: web::Path<String>) -> impl Responder {
    let username = path.into_inner();
    let result = web::block(move || app_data.user_service.get_by_username(&username)).await;
    match result {
        Ok(result) => HttpResponse::Ok().json(result),
        // Err(e) => HttpResponse::BadRequest().json::<GenericError>(e.into())
        Err(_) => HttpResponse::BadRequest().finish()
    }
}
