#[macro_use]
extern crate tracing;

use crate::error::MagicApiError;
use axum::{routing::get, Router};
use dotenv::dotenv;
use error::MagicAppResult;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

pub mod conn;
pub mod dto;
pub mod entities;
pub mod error;
pub mod routes;

pub const SERVE_AT: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> MagicAppResult<()> {
    dotenv().ok();

    let all_files = rolling::minutely("./logs/debug", "debug")
        .and(rolling::daily("./logs/warnings", "warnings").with_max_level(tracing::Level::WARN));

    tracing_subscriber::fmt()
        .with_writer(all_files.and(std::io::stdout))
        .with_target(true)
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // axum logs rejections from built-in extractors with the `axum::rejection`
            // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
            "rust_log=warn,magic_axum_api=info,tower_http=info,axum::rejection=trace".into()
        }))
        .init();

    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/data", routes::mount().await?)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .fallback(fallback_handler);

    let listener = tokio::net::TcpListener::bind(SERVE_AT).await.unwrap();
    info!("Listening on: http://{}", SERVE_AT);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

async fn fallback_handler() -> MagicApiError {
    MagicApiError::NotFount404
}
