use axum::{routing::get, Router};
use std::env;

use crate::routes::handlers::get_accounts;
use crate::{conn::get_connection, error::MagicAppResult};

pub mod handlers;

pub async fn mount() -> MagicAppResult<Router> {
    let db_url = env::var("DATABASE_URL")
        .inspect_err(|e| warn!("Missing Database URL From DotEnv FIle: {e}"))?;
    let db_conn = get_connection(&db_url)
        .await
        .inspect_err(|e| error!("Error while connecting to PG Database: {e}"))?;

    let router = Router::new()
        .route("/get_accounts", get(get_accounts))
        .with_state(db_conn);

    Ok(router)
}
