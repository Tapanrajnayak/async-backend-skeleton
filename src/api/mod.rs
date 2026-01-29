pub mod handlers;
pub mod responses;

use axum::routing::{get, patch, post};
use axum::Router;

use crate::domain::service::TransactionService;
use crate::storage::Storage;

pub fn build_router<S: Storage + Clone>(service: TransactionService<S>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route(
            "/api/v1/transactions",
            post(handlers::create_transaction::<S>).get(handlers::list_transactions::<S>),
        )
        .route(
            "/api/v1/transactions/{id}",
            get(handlers::get_transaction::<S>),
        )
        .route(
            "/api/v1/transactions/{id}/status",
            patch(handlers::update_transaction_status::<S>),
        )
        .with_state(service)
}
