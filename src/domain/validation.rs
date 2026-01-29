use crate::domain::models::CreateTransactionRequest;
use crate::error::AppError;

const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_IDEMPOTENCY_KEY_LENGTH: usize = 128;

pub fn validate_create_request(req: &CreateTransactionRequest) -> Result<(), AppError> {
    if req.amount <= 0.0 {
        return Err(AppError::Validation(
            "Amount must be greater than zero".into(),
        ));
    }

    if !req.amount.is_finite() {
        return Err(AppError::Validation("Amount must be a finite number".into()));
    }

    if req.description.trim().is_empty() {
        return Err(AppError::Validation(
            "Description must not be empty".into(),
        ));
    }

    if req.description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(AppError::Validation(format!(
            "Description must not exceed {} characters",
            MAX_DESCRIPTION_LENGTH
        )));
    }

    if req.idempotency_key.trim().is_empty() {
        return Err(AppError::Validation(
            "Idempotency key must not be empty".into(),
        ));
    }

    if req.idempotency_key.len() > MAX_IDEMPOTENCY_KEY_LENGTH {
        return Err(AppError::Validation(format!(
            "Idempotency key must not exceed {} characters",
            MAX_IDEMPOTENCY_KEY_LENGTH
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Currency;

    fn valid_request() -> CreateTransactionRequest {
        CreateTransactionRequest {
            idempotency_key: "key-123".into(),
            amount: 100.0,
            currency: Currency::Usd,
            description: "Test payment".into(),
        }
    }

    #[test]
    fn valid_request_passes() {
        assert!(validate_create_request(&valid_request()).is_ok());
    }

    #[test]
    fn zero_amount_rejected() {
        let mut req = valid_request();
        req.amount = 0.0;
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn negative_amount_rejected() {
        let mut req = valid_request();
        req.amount = -50.0;
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn infinite_amount_rejected() {
        let mut req = valid_request();
        req.amount = f64::INFINITY;
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn empty_description_rejected() {
        let mut req = valid_request();
        req.description = "   ".into();
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn long_description_rejected() {
        let mut req = valid_request();
        req.description = "x".repeat(501);
        assert!(validate_create_request(&req).is_err());
    }

    #[test]
    fn empty_idempotency_key_rejected() {
        let mut req = valid_request();
        req.idempotency_key = "".into();
        assert!(validate_create_request(&req).is_err());
    }
}
