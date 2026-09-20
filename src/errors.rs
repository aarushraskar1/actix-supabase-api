use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("resource not found")]
    NotFound,
    #[error("invalid request: {0}")]
    BadRequest(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        if let Self::Database(source) = self {
            error!(error = %source, "database request failed");
        }

        let message = match self {
            Self::Database(_) => "an internal database error occurred".to_owned(),
            _ => self.to_string(),
        };

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self
                .status_code()
                .canonical_reason()
                .unwrap_or("request error"),
            message,
        })
    }
}
