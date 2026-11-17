use anyhow::Context;
use cms_backend::{config::Config, db, routes, services, state::AppState};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,cms_backend=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env().context("invalid configuration")?;
    let addr = format!("{}:{}", config.server_host, config.server_port);

    let pool = db::create_pool(&config)
        .await
        .context("failed to connect to PostgreSQL (is `docker compose up -d` running?)")?;

    // Migrations must succeed before the server accepts any request.
    db::migrations::run(&pool)
        .await
        .context("database migration failed; refusing to start")?;

    let state = AppState::new(pool.clone(), config);

    if let (Some(email), Some(pw)) = (&state.config.admin_email, &state.config.admin_password) {
        let created = services::auth::bootstrap_admin(&state.pool, email, pw)
            .await
            .context("failed to bootstrap initial admin")?;
        if created {
            tracing::info!("initial admin account created");
        }
    }

    let app = routes::router(state);

    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    pool.close().await;
    tracing::info!("shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %e, "failed to listen for Ctrl+C");
        return;
    }
    tracing::info!("Ctrl+C received, shutting down");
}
