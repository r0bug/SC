#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePool;
    use std::sync::Arc;
    use sync_service::auth::{AuthService, LoginRequest, SignupRequest};

    async fn setup_test_db() -> Arc<SqlitePool> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Create basic tables for testing
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                api_token TEXT,
                email_verified BOOLEAN DEFAULT 0,
                active BOOLEAN DEFAULT 1,
                preferences TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_login_at TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS resource_acls (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                grants TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create resource_acls table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS acl_grants (
                id TEXT PRIMARY KEY,
                acl_id TEXT NOT NULL,
                grantee_id TEXT NOT NULL,
                permissions TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (acl_id) REFERENCES resource_acls(id)
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create acl_grants table");

        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_auth_service_signup_login() {
        std::env::set_var(
            "JWT_SECRET",
            "test-secret-key-for-integration-tests-only-32chars!",
        );
        let pool = setup_test_db().await;
        let auth_service = AuthService::new(pool);

        // Test signup
        let signup_req = SignupRequest {
            email: "test@example.com".to_string(),
            password: "S3cur3P@ssXyz99".to_string(),
            name: "Test User".to_string(),
        };

        let signup_result = auth_service.signup(signup_req).await;
        assert!(
            signup_result.is_ok(),
            "Signup failed: {:?}",
            signup_result.err()
        );
        let signup_response = signup_result.unwrap();
        assert!(!signup_response.token.is_empty());

        // Test login
        let login_req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "S3cur3P@ssXyz99".to_string(),
        };

        let login_result = auth_service.login(login_req).await;
        assert!(login_result.is_ok());
        let login_response = login_result.unwrap();
        assert!(!login_response.token.is_empty());

        // Test invalid login with non-existent email
        let invalid_login = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "WrongPassword".to_string(),
        };

        let invalid_result = auth_service.login(invalid_login).await;
        assert!(invalid_result.is_err());
    }

    #[tokio::test]
    async fn test_acl_service_permissions() {
        use core_domain::{Permission, ShareEntityType};
        use sync_service::acl::AclService;
        use uuid::Uuid;

        let pool = setup_test_db().await;
        let acl_service = AclService::new(pool);

        let owner_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let entity_type = ShareEntityType::Contact;

        // Create ACL
        let create_result = acl_service
            .create_acl(&owner_id, entity_type, &entity_id)
            .await;
        assert!(
            create_result.is_ok(),
            "ACL creation failed: {:?}",
            create_result.err()
        );

        // Owner should have all permissions
        let owner_can_read = acl_service
            .can_read(&owner_id, entity_type, &entity_id)
            .await
            .unwrap();
        assert!(owner_can_read);

        let owner_can_delete = acl_service
            .can_delete(&owner_id, entity_type, &entity_id)
            .await
            .unwrap();
        assert!(owner_can_delete);

        // Other user should not have permissions
        let user_can_read = acl_service
            .can_read(&user_id, entity_type, &entity_id)
            .await
            .unwrap();
        assert!(!user_can_read);

        // Grant read permission
        let grant_result = acl_service
            .grant_permission(
                &owner_id,
                &user_id,
                entity_type,
                &entity_id,
                vec![Permission::Read],
            )
            .await;
        assert!(grant_result.is_ok());

        // Now user should have read permission
        let user_can_read_after = acl_service
            .can_read(&user_id, entity_type, &entity_id)
            .await
            .unwrap();
        assert!(user_can_read_after);

        // But not write permission
        let user_can_write = acl_service
            .can_write(&user_id, entity_type, &entity_id)
            .await
            .unwrap();
        assert!(!user_can_write);
    }

    #[tokio::test]
    async fn test_websocket_broadcaster() {
        use sync_service::websocket::{BroadcastEvent, WebSocketBroadcaster};
        use uuid::Uuid;

        let broadcaster = WebSocketBroadcaster::new();
        let user_id = Uuid::new_v4();
        let contact_id = Uuid::new_v4();

        // Test broadcast without errors
        let event = BroadcastEvent::ContactCreated {
            id: contact_id,
            user_id,
        };

        broadcaster.broadcast(event).await;
        // No clients connected, so this should just complete without error
    }
}
