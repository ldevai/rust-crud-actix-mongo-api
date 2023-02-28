use actix_web::{Error, HttpMessage};
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use bcrypt::hash;
use chrono::{Duration, Utc};

use crate::AppState;
use crate::auth::models::Claims;
use crate::errors::GenericError;
use crate::users::models::User;

//
// jwt methods
//
fn get_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET").unwrap().into_bytes()
}

/**
Parses JWT token from Cookie or Authorization header, adds user to session and returns list of roles for integration with actix_web_grants
 */
pub async fn jwt_middleware(req: &ServiceRequest) -> Result<Vec<String>, actix_web::Error> {
    // Obtain token
    let token = match req.cookie("token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers().get(actix_web::http::header::AUTHORIZATION)
                .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
        }) {
        Some(result) => result,
        None => return Ok(vec![])
    };

    // Parse and validate JWT token
    match jsonwebtoken::decode::<Claims>(&token.clone(), &jsonwebtoken::DecodingKey::from_secret(&get_secret()), &jsonwebtoken::Validation::default()) {
        Ok(_) => {}
        Err(_) => return Err(Error::from(GenericError { message: "Error validating token" }))
    }

    // Validate against database - ideally should be done from a cache
    let app_state = req.app_data::<Data<AppState>>().unwrap();
    match app_state.auth_service.validate(&token) {
        Ok(user) => {
            req.extensions_mut().insert(user.clone());
            let roles: Vec<String> = user.roles.iter().map(|r| {
                let mut role_name = "ROLE_".to_string();
                role_name.push_str(&r.to_string().to_uppercase());
                role_name
            }).collect();
            Ok(roles)
        }
        Err(e) => {
            println!("[jwt_middleware] Error validating token {:?}", e.message);
            Ok(vec![])
        }
    }
}

/**
Generates token for user
 */
pub fn get_jwt_for_user(user: &User) -> String {
    let expiration_time = Utc::now()
        .checked_add_signed(Duration::hours(8))
        .expect("invalid timestamp")
        .timestamp();

    let roles_str: Vec<String> = user.roles.iter().map(|r| r.to_string()).collect();
    let role_str = roles_str.join(",").to_string();

    let user_claims = Claims {
        sub: user.username.clone(),
        role: role_str,
        exp: expiration_time as usize,
    };

    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &user_claims,
        &jsonwebtoken::EncodingKey::from_secret(&get_secret()),
    ) {
        Ok(t) => t,
        Err(_) => panic!(),
    };

    token
}

//
// hashing methods
//
pub fn get_hashed_password(password: &str) -> String {
    const COST: u32 = 6;
    let password_hash = hash(password, COST).unwrap().to_string();
    password_hash
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    bcrypt::verify(password, password_hash).is_ok()
}
