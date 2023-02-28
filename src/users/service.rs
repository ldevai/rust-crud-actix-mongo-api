use bson::{Document, from_document, to_document};
use chrono::Utc;
use mongodb::{error::Error, results::InsertOneResult, sync::Collection};

use crate::environment::Environment;
use crate::errors::GenericError;
use crate::security::get_hashed_password;
use crate::users::models::{CreateUser, User, UserView};

#[derive(Clone)]
pub struct UserService {
    collection: Collection<Document>,
}

impl UserService {
    pub fn new(env: Environment) -> UserService {
        let collection: Collection<Document> = env.db().collection("users");
        UserService { collection }
    }

    pub fn create(&self, request: CreateUser) -> Result<InsertOneResult, GenericError> {
        let filter = bson::doc! {"username": &request.username };
        // let filter = bson::doc! {"$or": [{"email": &request.email }, {"username": &request.username }]};
        match self.collection.find_one(filter, None).unwrap() {
            Some(_) => return Err(GenericError { message: "User already exists" }),
            None => (),
        }

        let user = User {
            id: None,
            email: request.email,
            username: request.username,
            password: get_hashed_password(&request.password),
            roles: request.roles,
            tokens: None,
            created_at: Some(Utc::now()),
            updated_at: None,
        };

        let mut doc: Document = to_document(&user).unwrap();
        doc.remove("_id"); // Remove None field that would be saved instead of auto-generated

        let result: Result<InsertOneResult, Error> = self.collection.insert_one(doc, None);
        Ok(result.unwrap())
    }

    pub fn get_by_username(&self, username: &str) -> Result<UserView, GenericError> {
        let filter = bson::doc! {"username": username};
        let result: Result<Option<Document>, mongodb::error::Error> = self.collection.find_one(filter, None);
        match result.unwrap() {
            Some(doc) => Ok(from_document(doc).unwrap()),
            None => Err(GenericError { message: "Not found" }),
        }
    }
}
