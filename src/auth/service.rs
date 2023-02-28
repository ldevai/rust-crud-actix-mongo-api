use bson::{doc, Document, from_document};
use chrono::Utc;
use mongodb::sync::Collection;

use crate::auth::models::{AuthRequest, AuthResponse};
use crate::environment::Environment;
use crate::errors::GenericError;
use crate::security::{get_jwt_for_user, verify_password};
use crate::users::models::{Tokens, User};

#[derive(Clone)]
pub struct AuthService {
    collection: Collection<Document>,
}

impl AuthService {
    pub fn new(env: Environment) -> AuthService {
        let collection: Collection<Document> = env.db().collection("users");
        AuthService { collection }
    }

    pub fn login(&self, request: AuthRequest) -> Result<AuthResponse, GenericError> {
        // Find user by email
        let filter = doc! {"email": &request.email };
        let existing: User = match self.collection.find_one(filter, None).unwrap() {
            Some(obj) => from_document(obj).unwrap(),
            None => return Err(GenericError { message: "Not found" }),
        };

        // Validate passwords
        match verify_password(&request.password, &existing.password) {
            true => (),
            false => return Err(GenericError { message: "Invalid credentials" }),
        };

        self.generate_tokens_and_update(existing)
    }

    /**
    Used by jwt_middleware to check if token has not been revoked
     */
    pub fn validate(&self, token: &str) -> Result<User, GenericError> {
        // todo!("Get session from faster store such as redis");
        let filter = doc! { "tokens.access_token": &token };
        let user: User = match self.collection.find_one(filter, None).unwrap() {
            Some(obj) => from_document(obj).unwrap(),
            None => return Err(GenericError { message: "Not found" }),
        };
        Ok(user)
    }


    /**
    Generates new tokens if the given refresh_token is valid
     */
    pub fn refresh(&self, token: &str) -> Result<AuthResponse, GenericError> {
        // todo!("Get session from faster store such as redis");
        let filter = doc! { "tokens.refresh_token": &token };
        let user: User = match self.collection.find_one(filter, None).unwrap() {
            Some(obj) => from_document(obj).unwrap(),
            None => return Err(GenericError { message: "Not found" }),
        };
        self.generate_tokens_and_update(user)
    }

    pub fn generate_tokens_and_update(&self, mut user: User) -> Result<AuthResponse, GenericError> {
        // Generate tokens and save them
        let access_token = get_jwt_for_user(&user);
        // todo!("Improve refresh token")
        let refresh_token = get_jwt_for_user(&user);
        user.tokens = Some(Tokens {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
        });
        user.updated_at = Some(Utc::now());

        let filter = doc! { "_id": user.id };
        let updates = doc! { "$set": bson::to_document(&user).unwrap() };
        self.collection.update_one(filter, updates, None).unwrap();

        // Create response object
        let result = AuthResponse {
            email: user.email.to_string(),
            username: user.username.to_string(),
            roles: user.roles,
            tokens: user.tokens.unwrap(),
        };
        Ok(result)
    }
}
