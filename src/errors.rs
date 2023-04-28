use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Auth {
    Authentication,
    Authorization,
}

#[derive(Debug, Serialize)]
pub enum ErrorTypes {
    ValidationError,
    DbError,
    NotFoundError,
    Auth(Auth),
    JwtError,
}

#[derive(Debug, Serialize)]
pub struct Error {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: ErrorTypes,
}

impl Error {
    pub fn new(cause: Option<String>, message: Option<String>, error_type: ErrorTypes) -> Self {
        Error {
            cause,
            message,
            error_type,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json_string = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", json_string)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self.error_type {
            ErrorTypes::ValidationError => StatusCode::BAD_REQUEST,
            ErrorTypes::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorTypes::NotFoundError => StatusCode::NOT_FOUND,
            ErrorTypes::Auth(auth) => match auth {
                Auth::Authentication => StatusCode::UNAUTHORIZED,
                Auth::Authorization => StatusCode::FORBIDDEN,
            },
            ErrorTypes::JwtError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
}
