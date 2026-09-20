mod config;
mod db;
mod errors;
mod models;
mod routes;

use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Context;
use config::Config;
use routes::AppState;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "actix_supabase_api=info,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env().context("failed to load application configuration")?;
    let pool = db::create_pool(&config.database_url, config.database_max_connections)
        .await
        .context("failed to connect to PostgreSQL")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    let state = web::Data::new(AppState { pool });
    let bind_address = format!("{}:{}", config.host, config.port);

    info!(%bind_address, "starting HTTP server");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .configure(routes::configure)
    })
    .bind(&bind_address)
    .with_context(|| format!("failed to bind HTTP server to {bind_address}"))?
    .run()
    .await
    .context("HTTP server stopped unexpectedly")
}
