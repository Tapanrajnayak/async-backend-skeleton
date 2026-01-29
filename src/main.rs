use async_backend_skeleton::api::build_router;
use async_backend_skeleton::domain::service::TransactionService;
use async_backend_skeleton::storage::memory::InMemoryStorage;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .json()
        .init();

    let storage = InMemoryStorage::new();
    let service = TransactionService::new(storage);
    let app = build_router(service).layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::error!("Port {} is already in use. Set a different port with PORT=<number>.", port);
            std::process::exit(1);
        }
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
