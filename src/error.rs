use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt::Display;
use strum_macros::AsRefStr;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, AsRefStr)]
pub enum Error {
    SystemTimeError,
    EnvironmentError,
    RequestError,
    SerializationError,
    DeserializationError,
    MissingChoiceError,
    StreamError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(self);

        response
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::error::Error for Error {}
