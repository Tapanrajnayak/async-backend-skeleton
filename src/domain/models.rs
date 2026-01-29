use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

impl TransactionStatus {
    /// Returns whether transitioning from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Pending, Self::Completed)
                | (Self::Pending, Self::Failed)
                | (Self::Pending, Self::Cancelled)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Usd,
    Eur,
    Gbp,
    Jpy,
    Cad,
    Aud,
    Chf,
}

impl Currency {
    pub const ALLOWED: &[&str] = &["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF"];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub idempotency_key: String,
    pub amount: f64,
    pub currency: Currency,
    pub description: String,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub idempotency_key: String,
    pub amount: f64,
    pub currency: Currency,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: TransactionStatus,
}

#[derive(Debug, Deserialize)]
pub struct ListFilters {
    pub status: Option<TransactionStatus>,
    pub currency: Option<Currency>,
}
