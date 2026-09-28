use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, get, web};
use mongodb::Client;

use crate::auth::service::AuthService;
use crate::errors::ApiError;
use crate::users::service::UserService;

mod auth;
mod errors;
mod security;
mod test_controller;
mod users;

pub struct AppState {
    pub auth_service: AuthService,
    pub user_service: UserService,
}

#[get("/health")]
async fn health() -> &'static str {
    "ok"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let port: u16 = env_or("PORT", "8080")
        .parse()
        .expect("PORT must be a number");
    let mongo_url = env_or("MONGO_URL", "mongodb://localhost:27017");
    let mongo_db = env_or("MONGO_DB", "demo");
    let secret = std::env::var("AUTH_SECRET").expect("AUTH_SECRET must be set");

    let client = Client::with_uri_str(&mongo_url)
        .await
        .expect("invalid MONGO_URL");
    let db = client.database(&mongo_db);
    let user_service = UserService::new(&db);
    user_service
        .ensure_indexes()
        .await
        .expect("cannot reach MongoDB");

    let state = web::Data::new(AppState {
        auth_service: AuthService::new(&db, secret.as_bytes()),
        user_service,
    });

    log::info!("listening on :{port}");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| ApiError::bad_request(err.to_string()).into()),
            )
            .wrap(Logger::default())
            .service(health)
            .service(auth::controller::login)
            .service(auth::controller::refresh)
            .service(auth::controller::validate)
            .service(users::controller::create)
            .service(users::controller::get)
            .service(test_controller::public)
            .service(test_controller::protected_user)
            .service(test_controller::protected_admin)
            .default_service(web::to(errors::not_found))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}
