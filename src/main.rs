use std::io::Result;

use actix_web::{App, HttpServer};
use actix_web::web::Data;
use actix_web_grants::GrantsMiddleware;

use crate::auth::service::AuthService;
use crate::environment::Environment;
use crate::users::service::UserService;

mod environment;
mod errors;
mod auth;
mod users;
mod security;

mod test_controller;

pub struct AppState {
    auth_service: AuthService,
    user_service: UserService,
}

impl AppState {
    pub fn new(env: Environment) -> Self {
        AppState {
            auth_service: AuthService::new(env.clone()),
            user_service: UserService::new(env.clone()),
        }
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let env = Environment::new().await;
    let host = &env.config().host.clone();

    HttpServer::new(move || {
        let app_state = AppState::new(env.clone());
        App::new()
            .app_data(Data::new(app_state))
            .wrap(GrantsMiddleware::with_extractor(security::jwt_middleware))

            .service(auth::controller::login)
            .service(auth::controller::refresh)
            .service(auth::controller::validate)

            .service(users::controller::create)
            .service(users::controller::get)

            .service(test_controller::public)
            .service(test_controller::protected_user)
            .service(test_controller::protected_admin)
    })
        .bind(host)?
        .run()
        .await
}
