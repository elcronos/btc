pub struct AuthValidator {
    allowed_telegram_ids: Vec<i64>,
    allowed_slack_users: Vec<String>,
}

impl AuthValidator {
    pub fn new(allowed_telegram_ids: Vec<i64>, allowed_slack_users: Vec<String>) -> Self {
        Self {
            allowed_telegram_ids,
            allowed_slack_users,
        }
    }

    pub fn validate_telegram(&self, chat_id: i64) -> bool {
        self.allowed_telegram_ids.contains(&chat_id)
    }

    pub fn validate_slack(&self, user_id: &str) -> bool {
        self.allowed_slack_users.iter().any(|id| id == user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_telegram_accepts_allowed() {
        let validator = AuthValidator::new(vec![123, 456], vec![]);
        assert!(validator.validate_telegram(123));
        assert!(validator.validate_telegram(456));
    }

    #[test]
    fn test_validate_telegram_rejects_unknown() {
        let validator = AuthValidator::new(vec![123], vec![]);
        assert!(!validator.validate_telegram(999));
    }

    #[test]
    fn test_validate_slack_accepts_allowed() {
        let validator = AuthValidator::new(vec![], vec!["U001".to_string(), "U002".to_string()]);
        assert!(validator.validate_slack("U001"));
        assert!(validator.validate_slack("U002"));
    }

    #[test]
    fn test_validate_slack_rejects_unknown() {
        let validator = AuthValidator::new(vec![], vec!["U001".to_string()]);
        assert!(!validator.validate_slack("U999"));
    }
}
