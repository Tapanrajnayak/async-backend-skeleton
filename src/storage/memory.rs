use crate::domain::models::{Currency, Transaction, TransactionStatus};
use crate::error::AppError;
use crate::storage::Storage;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Uuid, Transaction>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for InMemoryStorage {
    async fn insert(&self, txn: Transaction) -> Result<(), AppError> {
        let mut store = self.data.write().await;
        store.insert(txn.id, txn);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Transaction>, AppError> {
        let store = self.data.read().await;
        Ok(store.get(&id).cloned())
    }

    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<Transaction>, AppError> {
        let store = self.data.read().await;
        Ok(store.values().find(|t| t.idempotency_key == key).cloned())
    }

    async fn list(
        &self,
        status: Option<TransactionStatus>,
        currency: Option<Currency>,
    ) -> Result<Vec<Transaction>, AppError> {
        let store = self.data.read().await;
        let results = store
            .values()
            .filter(|t| status.is_none_or(|s| t.status == s))
            .filter(|t| currency.is_none_or(|c| t.currency == c))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, AppError> {
        let mut store = self.data.write().await;
        let txn = store
            .get_mut(&id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if !txn.status.can_transition_to(status) {
            return Err(AppError::InvalidStateTransition {
                from: txn.status.to_string(),
                to: status.to_string(),
            });
        }

        txn.status = status;
        txn.updated_at = Utc::now();
        Ok(txn.clone())
    }
}
