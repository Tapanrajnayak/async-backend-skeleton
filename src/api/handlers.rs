use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::api::responses::ApiResponse;
use crate::domain::models::{CreateTransactionRequest, ListFilters, UpdateStatusRequest};
use crate::domain::service::TransactionService;
use crate::error::AppError;
use crate::storage::Storage;

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn create_transaction<S: Storage>(
    State(svc): State<TransactionService<S>>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (txn, created) = svc.create(req).await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(ApiResponse::new(txn))))
}

pub async fn get_transaction<S: Storage>(
    State(svc): State<TransactionService<S>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let txn = svc.get(id).await?;
    Ok(Json(ApiResponse::new(txn)))
}

pub async fn list_transactions<S: Storage>(
    State(svc): State<TransactionService<S>>,
    Query(filters): Query<ListFilters>,
) -> Result<impl IntoResponse, AppError> {
    let txns = svc.list(filters).await?;
    Ok(Json(ApiResponse::new(txns)))
}

pub async fn update_transaction_status<S: Storage>(
    State(svc): State<TransactionService<S>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<impl IntoResponse, AppError> {
    let txn = svc.update_status(id, req).await?;
    Ok(Json(ApiResponse::new(txn)))
}
