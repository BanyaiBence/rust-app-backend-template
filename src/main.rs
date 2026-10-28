use std::process::ExitCode;

use backend::{config::Config, state::AppState, telemetry};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            // Logging is configured from the config, so this one goes straight to stderr.
            eprintln!("configuration error: {err}");
            return ExitCode::FAILURE;
        }
    };
    telemetry::init(&config.log_format);

    match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(error = %err, "server failed");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.jwt_secret == Config::EXAMPLE_JWT_SECRET {
        tracing::warn!("JWT_SECRET is the example value from .env.example; change it");
    }

    let pool = backend::db::connect(&config).await?;
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("database migrations applied");

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");

    let state = AppState::new(pool.clone(), config);
    axum::serve(listener, backend::build_app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    pool.close().await;
    tracing::info!("shut down cleanly");
    Ok(())
}

/// Resolves on Ctrl+C or SIGTERM (sent by `docker stop`), letting in-flight requests finish.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("installing Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("installing SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutdown signal received, draining connections");
}
