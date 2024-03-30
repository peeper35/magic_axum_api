use crate::error::MagicApiError;
use axum::extract::FromRequestParts;
use axum::{body::Body, extract::FromRequest, http::Response, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(FromRequestParts, Deserialize)]
#[from_request(via(axum::extract::Query), rejection(MagicApiError))]
pub struct MagicApiQuery<T>(pub T);

#[derive(Deserialize)]
pub struct GetMagicNosRequestQuery {
    pub account: String,
}

pub enum Message {
    EmptyData,
}

impl From<Message> for String {
    fn from(val: Message) -> Self {
        match val {
            Message::EmptyData => "Empty Data".to_string(),
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

#[derive(Serialize)]
pub struct GetAccountMagicNos {
    pub account: String,
    pub magic_numbers: Vec<i32>,
}
