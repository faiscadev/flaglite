mod config;
mod db;
mod error;
mod handlers;
mod models;
mod auth;

use axum::{
    routing::{get, patch, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "flaglite")]
#[command(about = "FlagLite - Lightweight feature flag service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Run database migrations
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "flaglite=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = config::Config::from_env()?;

    match cli.command {
        Commands::Serve { port, host } => {
            let pool = db::create_pool(&config.database_url).await?;
            
            // Run migrations on startup
            db::run_migrations(&pool).await?;
            
            let app_state = models::AppState {
                pool,
                jwt_secret: config.jwt_secret,
            };

            let app = create_router(app_state);

            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            tracing::info!("🚀 FlagLite API listening on {}", addr);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
        Commands::Migrate => {
            let pool = db::create_pool(&config.database_url).await?;
            db::run_migrations(&pool).await?;
            tracing::info!("✅ Migrations completed successfully");
        }
    }

    Ok(())
}

fn create_router(state: models::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(|| async { "OK" }))
        // Auth routes
        .route("/v1/auth/signup", post(handlers::auth::signup))
        .route("/v1/auth/login", post(handlers::auth::login))
        .route("/v1/auth/me", get(handlers::auth::me))
        // Flag routes
        .route("/v1/flags", get(handlers::flags::list_flags))
        .route("/v1/flags", post(handlers::flags::create_flag))
        .route("/v1/flags/:key", get(handlers::flags::evaluate_flag))
        .route(
            "/v1/flags/:key/environments/:env",
            patch(handlers::flags::update_flag_value),
        )
        .route("/v1/flags/:key/toggle", post(handlers::flags::toggle_flag))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
