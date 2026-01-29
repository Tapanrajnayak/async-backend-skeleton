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

    let addr = "0.0.0.0:3000";
    tracing::info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
