use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

use axum::http::StatusCode;
use axum::{extract::rejection::JsonRejection, response::IntoResponse};
use std::env::VarError;

use crate::dto::MagicApiJson;

pub type MagicAppResult<T> = Result<T, MagicAppError>;
pub type MagicApiResult<T> = Result<T, MagicApiError>;

#[derive(Error, Debug)]
pub enum MagicAppError {
    #[error("Unable To Connect To PG Database")]
    InvalidDbConnection(#[from] DbErr),
    #[error("Environment Variable Not Set")]
    EnvVarError(#[from] VarError),
}

#[derive(Error, Debug)]
pub enum MagicApiError {
    #[error("404")]
    NotFount404,
    #[error("JSON Rejection")]
    JsonRejection(#[from] JsonRejection),
    #[error("DB Error")]
    DbError(#[from] DbErr),
}

impl IntoResponse for MagicApiError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        pub struct ErrorResponse {
            status_code: u16,
            message: String,
        }

        let (status, message) = match self {
            MagicApiError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            MagicApiError::NotFount404 => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            MagicApiError::DbError(e) => {
                warn!("Db Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
        };

        (
            status,
            MagicApiJson(ErrorResponse {
                status_code: status.as_u16(),
                message,
            }),
        )
            .into_response()
    }
}
