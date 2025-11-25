/// System-level constants for SagensContact
use uuid::Uuid;
use std::str::FromStr;

/// System user ID - used for system-generated entities and legacy data
/// This user cannot log in and is created automatically during migration
pub const SYSTEM_USER_ID_STR: &str = "00000000-0000-0000-0000-000000000001";

/// Get the system user ID as a Uuid
pub fn system_user_id() -> Uuid {
    Uuid::from_str(SYSTEM_USER_ID_STR)
        .expect("Invalid system user UUID constant")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_user_id() {
        let id = system_user_id();
        assert_eq!(id.to_string(), SYSTEM_USER_ID_STR);
        assert_ne!(id, Uuid::nil());
    }
}
