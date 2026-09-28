use mongodb::bson::oid::ObjectId;
use mongodb::bson::{DateTime, doc};
use mongodb::options::IndexOptions;
use mongodb::{Collection, Database, IndexModel};

use crate::errors::{ApiError, is_duplicate_key};
use crate::security::hash_password;
use crate::users::models::{CreateUser, Role, User};

pub struct UserService {
    users: Collection<User>,
}

impl UserService {
    pub fn new(db: &Database) -> Self {
        Self {
            users: db.collection("users"),
        }
    }

    /// Unique indexes make duplicate emails/usernames impossible, even under
    /// concurrent sign-ups. Creating an existing index is a no-op.
    pub async fn ensure_indexes(&self) -> mongodb::error::Result<()> {
        for field in ["email", "username"] {
            let index = IndexModel::builder()
                .keys(doc! { field: 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build();
            self.users.create_index(index).await?;
        }
        Ok(())
    }

    /// Self-registration always creates a plain `User`; promoting someone to
    /// `Admin` is an operator action on the database (see README).
    pub async fn create(&self, mut request: CreateUser) -> Result<User, ApiError> {
        request.email = request.email.trim().to_lowercase();
        request.username = request.username.trim().to_string();
        request.validate()?;

        let now = DateTime::now();
        let user = User {
            id: ObjectId::new(),
            email: request.email,
            username: request.username,
            password_hash: hash_password(request.password).await?,
            roles: vec![Role::User],
            session_id: None,
            created_at: now,
            updated_at: now,
        };
        match self.users.insert_one(&user).await {
            Ok(_) => Ok(user),
            Err(err) if is_duplicate_key(&err) => {
                Err(ApiError::conflict("email or username already taken"))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User, ApiError> {
        self.users
            .find_one(doc! { "username": username })
            .await?
            .ok_or_else(|| ApiError::not_found("user not found"))
    }
}
