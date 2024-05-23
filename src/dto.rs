use crate::error::MagicApiError;
use axum::extract::FromRequestParts;
use axum::{body::Body, extract::FromRequest, http::Response, response::IntoResponse};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(FromRequestParts, Deserialize)]
#[from_request(via(axum::extract::Query), rejection(MagicApiError))]
pub struct MagicApiQuery<T>(pub T);

#[derive(Deserialize)]
pub struct GetMagicNosRequestQuery {
    pub account: String,
}

#[derive(Deserialize)]
pub struct GetSingleTradesStats {
    pub account: String,
    pub magic_no: i32,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct GetAllTradesStats {
    pub account: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
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

#[derive(Serialize)]
pub struct GetSingleMagicNoStats {
    pub account: String,
    pub magic_number: i32,
    pub trades_count: usize,
    pub gross_profit: f32,
    pub gross_loss: f32,
    pub net_profit: f32,
    pub profit_factor: f32,
    pub winners_count: usize,
    pub losers_count: usize,
    pub win_percentage: f32,
    pub lose_percentage: f32,
    pub avg_win: f32,
    pub avg_lose: f32,
    pub avg_trade: f32,
    pub best_trade: f32,
    pub worst_trade: f32,
    pub longest_trade: String,
    pub longest_trade_time: String,
    pub shortest_trade: String,
    pub shortest_trade_time: String,
    pub avg_trade_time: String,
    pub total_win_pips: f32,
    pub total_lose_pips: f32,
    pub net_pips: f32,
    pub avg_win_pips: f32,
    pub avg_lose_pips: f32,
    pub avg_net_pips: f32,
    pub consecutive_win_trades: Option<usize>,
    pub consecutive_profits: Option<f32>,
    pub consecutive_lose_trades: Option<usize>,
    pub consecutive_losses: Option<f32>,
}

#[derive(Serialize)]
pub struct GetAllMagicNoStats {
    data: Vec<GetSingleMagicNoStats>,
}

#[allow(clippy::new_without_default)]
impl GetAllMagicNoStats {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn data(&self) -> &Vec<GetSingleMagicNoStats> {
        &self.data
    }

    pub fn push_stat(&mut self, stat: GetSingleMagicNoStats) {
        self.data.push(stat);
    }
}
