use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::{FromRequest, HttpRequest, web};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::ApiError;
use crate::users::models::{Role, User};

pub const ACCESS_TOKEN_TTL: u64 = 15 * 60;
pub const REFRESH_TOKEN_TTL: u64 = 7 * 24 * 60 * 60;
pub const TOKEN_COOKIE: &str = "token";

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// `sid` is the user's current session: logging in again or refreshing
/// replaces it, which revokes every token issued before.
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub sid: String,
    pub typ: TokenType,
    pub exp: u64,
}

pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        // Only HS256 is accepted, and `exp` must be present and in the future.
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "sub"]);
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            validation,
        }
    }

    pub fn issue(
        &self,
        user_id: &str,
        session_id: &str,
        typ: TokenType,
    ) -> Result<String, ApiError> {
        let ttl = match typ {
            TokenType::Access => ACCESS_TOKEN_TTL,
            TokenType::Refresh => REFRESH_TOKEN_TTL,
        };
        let claims = Claims {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            typ,
            exp: get_current_timestamp() + ttl,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding).map_err(ApiError::internal)
    }

    pub fn verify(&self, token: &str, typ: TokenType) -> Result<Claims, ApiError> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .ok()
            .map(|data| data.claims)
            .filter(|claims| claims.typ == typ)
            .ok_or_else(|| ApiError::unauthorized("invalid or expired token"))
    }
}

// bcrypt is deliberately slow, so it runs on actix's blocking thread pool
// instead of stalling the async workers.
pub async fn hash_password(password: String) -> Result<String, ApiError> {
    web::block(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
        .await
        .map_err(ApiError::internal)?
        .map_err(ApiError::internal)
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, ApiError> {
    web::block(move || bcrypt::verify(password, &hash))
        .await
        .map_err(ApiError::internal)?
        .map_err(ApiError::internal)
}

/// The logged-in user, loaded from MongoDB. Add it as a handler argument to
/// require authentication; use `Option<CurrentUser>` where login is optional.
pub struct CurrentUser(pub User);

impl CurrentUser {
    pub fn require(&self, role: Role) -> Result<(), ApiError> {
        if self.0.roles.contains(&role) {
            Ok(())
        } else {
            Err(ApiError::forbidden(format!("requires role {role:?}")))
        }
    }
}

impl FromRequest for CurrentUser {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, ApiError>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let state = req.app_data::<web::Data<AppState>>().cloned();
        let token = token_from_request(req);
        Box::pin(async move {
            let state = state.ok_or_else(|| ApiError::internal("AppState is not registered"))?;
            let token = token.ok_or_else(|| ApiError::unauthorized("missing token"))?;
            state
                .auth_service
                .authenticate(&token)
                .await
                .map(CurrentUser)
        })
    }
}

/// `Authorization: Bearer <jwt>` first, then the `token` cookie set at login.
fn token_from_request(req: &HttpRequest) -> Option<String> {
    let bearer = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_string);
    bearer.or_else(|| {
        req.cookie(TOKEN_COOKIE)
            .map(|cookie| cookie.value().to_string())
    })
}
