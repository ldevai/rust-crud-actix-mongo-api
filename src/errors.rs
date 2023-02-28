use actix_web::ResponseError;
use derive_more::{Display, Error};
use serde::Serialize;

#[derive(Debug, Display, Error, Serialize)]
#[display(fmt = "{}", message)]
pub struct GenericError {
    pub message: &'static str,
}

impl ResponseError for GenericError {}
