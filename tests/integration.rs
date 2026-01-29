use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use async_backend_skeleton::api::build_router;
use async_backend_skeleton::domain::service::TransactionService;
use async_backend_skeleton::storage::memory::InMemoryStorage;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> axum::Router {
    let storage = InMemoryStorage::new();
    let service = TransactionService::new(storage);
    build_router(service)
}

async fn body_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_check() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn create_and_get_transaction() {
    let app = app();

    // Create
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "idempotency_key": "txn-001",
                        "amount": 150.75,
                        "currency": "USD",
                        "description": "Invoice payment"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = body_json(create_resp.into_body()).await;
    let txn_id = create_body["data"]["id"].as_str().unwrap();
    assert_eq!(create_body["data"]["status"], "PENDING");
    assert_eq!(create_body["data"]["amount"], 150.75);

    // Get
    let get_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/transactions/{}", txn_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = body_json(get_resp.into_body()).await;
    assert_eq!(get_body["data"]["id"], txn_id);
}

#[tokio::test]
async fn idempotent_create_returns_200() {
    let app = app();
    let payload = json!({
        "idempotency_key": "idem-key",
        "amount": 50.0,
        "currency": "EUR",
        "description": "Duplicate test"
    })
    .to_string();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_amount_returns_400() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "idempotency_key": "bad",
                        "amount": -10.0,
                        "currency": "USD",
                        "description": "Negative"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn state_transition_pending_to_completed() {
    let app = app();

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "idempotency_key": "st-1",
                        "amount": 100.0,
                        "currency": "GBP",
                        "description": "State test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_body = body_json(create_resp.into_body()).await;
    let txn_id = create_body["data"]["id"].as_str().unwrap();

    let patch_resp = app
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri(format!("/api/v1/transactions/{}/status", txn_id))
                .header("content-type", "application/json")
                .body(Body::from(json!({"status": "COMPLETED"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(patch_resp.status(), StatusCode::OK);
    let patch_body = body_json(patch_resp.into_body()).await;
    assert_eq!(patch_body["data"]["status"], "COMPLETED");
}

#[tokio::test]
async fn invalid_state_transition_returns_422() {
    let app = app();

    // Create
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "idempotency_key": "st-2",
                        "amount": 100.0,
                        "currency": "USD",
                        "description": "Transition test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let create_body = body_json(create_resp.into_body()).await;
    let txn_id = create_body["data"]["id"].as_str().unwrap();

    // Complete it
    app.clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri(format!("/api/v1/transactions/{}/status", txn_id))
                .header("content-type", "application/json")
                .body(Body::from(json!({"status": "COMPLETED"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to move back to pending
    let bad_resp = app
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri(format!("/api/v1/transactions/{}/status", txn_id))
                .header("content-type", "application/json")
                .body(Body::from(json!({"status": "PENDING"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(bad_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_transactions() {
    let app = app();

    // Create two transactions
    for key in &["list-1", "list-2"] {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/api/v1/transactions")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "idempotency_key": key,
                            "amount": 10.0,
                            "currency": "USD",
                            "description": "List test"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let list_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/transactions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = body_json(list_resp.into_body()).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_nonexistent_returns_404() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/v1/transactions/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
