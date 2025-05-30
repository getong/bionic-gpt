pub mod api_keys;
pub mod api_pipeline;
pub mod audit_trail;
pub mod config;
pub mod console;
pub mod datasets;
pub mod documents;
pub mod email;
pub mod errors;
pub mod history;
pub mod integrations;
pub mod jwt;
pub mod layout;
pub mod metrics;
pub mod models;
pub mod oidc_endpoint;
pub mod pipelines;
pub mod profile;
pub mod prompts;
pub mod rate_limits;
pub mod static_files;
pub mod team;
pub mod teams;

use axum_extra::routing::RouterExt;
pub use errors::CustomError;
pub use jwt::Jwt;

use axum::{Extension, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Get log level from environment variable or default to INFO
    let log_level = std::env::var("LOG_LEVEL")
        .map(|level| match level.to_uppercase().as_str() {
            "TRACE" => tracing::Level::TRACE,
            "DEBUG" => tracing::Level::DEBUG,
            "INFO" => tracing::Level::INFO,
            "WARN" => tracing::Level::WARN,
            "ERROR" => tracing::Level::ERROR,
            _ => {
                eprintln!("Unknown log level: {}, defaulting to INFO", level);
                tracing::Level::INFO
            }
        })
        .unwrap_or(tracing::Level::INFO);

    // Create a filter that only enables your crates and disables others
    let filter = tracing_subscriber::EnvFilter::new("")
        // Disable all crates by default
        .add_directive(tracing_subscriber::filter::LevelFilter::OFF.into())
        // Enable your crates with the specified log level
        // Adjust these patterns to match your crate names
        .add_directive(format!("web_server={}", log_level).parse().unwrap())
        .add_directive(format!("db={}", log_level).parse().unwrap())
        .add_directive(format!("llm_proxy={}", log_level).parse().unwrap())
        // Add more of your crates as needed
        ;

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = config::Config::new();
    let pool = db::create_pool(&config.app_database_url);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // build our application with a route
    let app = Router::new()
        .typed_get(static_files::static_path)
        .typed_get(metrics::track_metrics)
        .typed_get(oidc_endpoint::index)
        .merge(api_pipeline::routes(&config))
        .merge(api_keys::routes())
        .merge(audit_trail::routes())
        .merge(console::routes())
        .merge(datasets::routes())
        .merge(documents::routes())
        .merge(history::routes())
        .merge(integrations::routes())
        .merge(llm_proxy::routes())
        .merge(models::routes())
        .merge(pipelines::routes())
        .merge(profile::routes())
        .merge(prompts::routes())
        .merge(rate_limits::routes())
        .merge(team::routes())
        .merge(teams::routes())
        .layer(Extension(config.clone()))
        .layer(Extension(pool.clone()));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
