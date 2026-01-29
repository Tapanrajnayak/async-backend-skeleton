pub mod memory;

use crate::domain::models::{Currency, Transaction, TransactionStatus};
use crate::error::AppError;
use std::future::Future;
use uuid::Uuid;

pub trait Storage: Send + Sync + 'static {
    fn insert(&self, txn: Transaction) -> impl Future<Output = Result<(), AppError>> + Send;

    fn get(&self, id: Uuid) -> impl Future<Output = Result<Option<Transaction>, AppError>> + Send;

    fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> impl Future<Output = Result<Option<Transaction>, AppError>> + Send;

    fn list(
        &self,
        status: Option<TransactionStatus>,
        currency: Option<Currency>,
    ) -> impl Future<Output = Result<Vec<Transaction>, AppError>> + Send;

    fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> impl Future<Output = Result<Transaction, AppError>> + Send;
}
