use crate::domain::models::{
    CreateTransactionRequest, ListFilters, Transaction, TransactionStatus, UpdateStatusRequest,
};
use crate::domain::validation::validate_create_request;
use crate::error::AppError;
use crate::storage::Storage;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionService<S: Storage> {
    storage: S,
}

impl<S: Storage> TransactionService<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Create a transaction. Returns `(transaction, created)` where `created` is false on
    /// idempotent replay.
    pub async fn create(
        &self,
        req: CreateTransactionRequest,
    ) -> Result<(Transaction, bool), AppError> {
        validate_create_request(&req)?;

        // Check idempotency
        if let Some(existing) = self.storage.find_by_idempotency_key(&req.idempotency_key).await? {
            return Ok((existing, false));
        }

        let now = Utc::now();
        let txn = Transaction {
            id: Uuid::new_v4(),
            idempotency_key: req.idempotency_key,
            amount: req.amount,
            currency: req.currency,
            description: req.description,
            status: TransactionStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        self.storage.insert(txn.clone()).await?;
        Ok((txn, true))
    }

    pub async fn get(&self, id: Uuid) -> Result<Transaction, AppError> {
        self.storage
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(id.to_string()))
    }

    pub async fn list(&self, filters: ListFilters) -> Result<Vec<Transaction>, AppError> {
        self.storage.list(filters.status, filters.currency).await
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        req: UpdateStatusRequest,
    ) -> Result<Transaction, AppError> {
        self.storage.update_status(id, req.status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Currency;
    use crate::storage::memory::InMemoryStorage;

    fn make_service() -> TransactionService<InMemoryStorage> {
        TransactionService::new(InMemoryStorage::new())
    }

    fn create_req(key: &str) -> CreateTransactionRequest {
        CreateTransactionRequest {
            idempotency_key: key.into(),
            amount: 250.0,
            currency: Currency::Usd,
            description: "Wire transfer".into(),
        }
    }

    #[tokio::test]
    async fn create_and_get() {
        let svc = make_service();
        let (txn, created) = svc.create(create_req("k1")).await.unwrap();
        assert!(created);
        assert_eq!(txn.status, TransactionStatus::Pending);

        let fetched = svc.get(txn.id).await.unwrap();
        assert_eq!(fetched.id, txn.id);
    }

    #[tokio::test]
    async fn idempotent_create() {
        let svc = make_service();
        let (first, created1) = svc.create(create_req("dup")).await.unwrap();
        assert!(created1);

        let (second, created2) = svc.create(create_req("dup")).await.unwrap();
        assert!(!created2);
        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn valid_state_transition() {
        let svc = make_service();
        let (txn, _) = svc.create(create_req("t1")).await.unwrap();

        let updated = svc
            .update_status(txn.id, UpdateStatusRequest { status: TransactionStatus::Completed })
            .await
            .unwrap();
        assert_eq!(updated.status, TransactionStatus::Completed);
    }

    #[tokio::test]
    async fn invalid_state_transition() {
        let svc = make_service();
        let (txn, _) = svc.create(create_req("t2")).await.unwrap();

        svc.update_status(txn.id, UpdateStatusRequest { status: TransactionStatus::Completed })
            .await
            .unwrap();

        let result = svc
            .update_status(txn.id, UpdateStatusRequest { status: TransactionStatus::Pending })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_with_filters() {
        let svc = make_service();
        svc.create(create_req("a")).await.unwrap();
        svc.create(create_req("b")).await.unwrap();

        let all = svc.list(ListFilters { status: None, currency: None }).await.unwrap();
        assert_eq!(all.len(), 2);

        let pending = svc
            .list(ListFilters { status: Some(TransactionStatus::Pending), currency: None })
            .await
            .unwrap();
        assert_eq!(pending.len(), 2);

        let completed = svc
            .list(ListFilters { status: Some(TransactionStatus::Completed), currency: None })
            .await
            .unwrap();
        assert!(completed.is_empty());
    }

    #[tokio::test]
    async fn get_not_found() {
        let svc = make_service();
        let result = svc.get(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
