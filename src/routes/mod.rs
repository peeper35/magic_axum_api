use axum::{routing::get, Router};
use std::env;

use crate::routes::handlers::{get_accounts, get_magic_nos, get_trades_stats};
use crate::{conn::get_connection, error::MagicAppResult};

pub mod handlers;
mod helpers;

pub async fn mount() -> MagicAppResult<Router> {
    let db_url = env::var("DATABASE_URL")
        .inspect_err(|e| warn!("Missing Database URL From DotEnv FIle: {e}"))?;
    let db_conn = get_connection(&db_url)
        .await
        .inspect_err(|e| error!("Error while connecting to PG Database: {e}"))?;

    let router = Router::new()
        .route("/accounts", get(get_accounts))
        .route("/account/magic_nos", get(get_magic_nos))
        .route("/account/trades_stats", get(get_trades_stats))
        .with_state(db_conn);

    Ok(router)
}
