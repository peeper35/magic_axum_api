use crate::error::MagicApiError;
use axum::{body::Body, extract::FromRequest, http::Response, response::IntoResponse};
use serde::Serialize;

pub enum Message {
    EmptyData,
}

impl Into<String> for Message {
    fn into(self) -> String {
        match self {
            Self::EmptyData => "Empty Data".to_string(),
        }
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(MagicApiError))]
pub struct MagicApiJson<T>(pub T);

impl<T> MagicApiJson<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T> IntoResponse for MagicApiJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        axum::Json(self.0).into_response()
    }
}

#[derive(Serialize)]
pub struct ResponseData<D> {
    pub status_code: u16,
    pub msg: Option<String>,
    pub data: Option<D>,
}

impl<T> ResponseData<T> {
    pub fn new(status_code: u16, msg: Option<String>, data: Option<T>) -> Self {
        Self {
            status_code,
            msg,
            data,
        }
    }
}

#[derive(Serialize)]
pub struct GetAccounts {
    pub accounts: Vec<String>,
}
