use mongodb::bson::oid::ObjectId;
use mongodb::bson::{DateTime, doc};
use mongodb::{Collection, Database};

use crate::auth::models::{AuthRequest, AuthResponse, Tokens};
use crate::errors::ApiError;
use crate::security::{Keys, TokenType, verify_password};
use crate::users::models::User;

pub struct AuthService {
    users: Collection<User>,
    keys: Keys,
}

impl AuthService {
    pub fn new(db: &Database, secret: &[u8]) -> Self {
        Self {
            users: db.collection("users"),
            keys: Keys::new(secret),
        }
    }

    pub async fn login(&self, request: AuthRequest) -> Result<AuthResponse, ApiError> {
        let invalid = || ApiError::unauthorized("invalid email or password");
        let email = request.email.trim().to_lowercase();
        let user = self
            .users
            .find_one(doc! { "email": email })
            .await?
            .ok_or_else(invalid)?;
        if !verify_password(request.password, user.password_hash.clone()).await? {
            return Err(invalid());
        }

        let session_id = ObjectId::new().to_hex();
        self.users
            .update_one(doc! { "_id": user.id }, set_session(&session_id))
            .await?;
        self.tokens_for(user, &session_id)
    }

    /// Trades a refresh token for a new token pair. The swap is atomic, so each
    /// refresh token works exactly once.
    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponse, ApiError> {
        let claims = self.keys.verify(refresh_token, TokenType::Refresh)?;
        let user_id = parse_id(&claims.sub)?;
        let session_id = ObjectId::new().to_hex();
        let user = self
            .users
            .find_one_and_update(
                doc! { "_id": user_id, "session_id": &claims.sid },
                set_session(&session_id),
            )
            .await?
            .ok_or_else(|| ApiError::unauthorized("refresh token already used or revoked"))?;
        self.tokens_for(user, &session_id)
    }

    /// Resolves an access token to its user, rejecting tokens from a replaced session.
    pub async fn authenticate(&self, access_token: &str) -> Result<User, ApiError> {
        let claims = self.keys.verify(access_token, TokenType::Access)?;
        let user_id = parse_id(&claims.sub)?;
        self.users
            .find_one(doc! { "_id": user_id, "session_id": &claims.sid })
            .await?
            .ok_or_else(|| ApiError::unauthorized("session expired, log in again"))
    }

    fn tokens_for(&self, user: User, session_id: &str) -> Result<AuthResponse, ApiError> {
        let user_id = user.id.to_hex();
        Ok(AuthResponse {
            tokens: Tokens {
                access_token: self.keys.issue(&user_id, session_id, TokenType::Access)?,
                refresh_token: self.keys.issue(&user_id, session_id, TokenType::Refresh)?,
            },
            email: user.email,
            username: user.username,
            roles: user.roles,
        })
    }
}

fn set_session(session_id: &str) -> mongodb::bson::Document {
    doc! { "$set": { "session_id": session_id, "updated_at": DateTime::now() } }
}

fn parse_id(sub: &str) -> Result<ObjectId, ApiError> {
    ObjectId::parse_str(sub).map_err(|_| ApiError::unauthorized("invalid or expired token"))
}
