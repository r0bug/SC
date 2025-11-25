/// Multi-User Isolation Tests
/// Verifies that users can only access their own data and cannot see each other's data

use chrono::Utc;
use core_domain::{Contact, Tag, User, system_user_id};
use local_store::{ContactRepository, TagRepository, UserRepository};
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

async fn setup_test_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Run migrations
    sqlx::query(local_store::migrations::SCHEMA)
        .execute(&pool)
        .await
        .unwrap();

    pool
}

async fn create_test_user(pool: &sqlx::Pool<sqlx::Sqlite>, email: &str, name: &str) -> User {
    let user = User {
        id: Uuid::new_v4(),
        email: email.to_string(),
        name: name.to_string(),
        password_hash: "hashed_password".to_string(),
        api_token: Some(format!("token_{}", email)),
        email_verified: true,
        active: true,
        preferences: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
    };

    let repo = UserRepository::new(pool);
    repo.create(&user).await.unwrap();
    user
}

#[tokio::test]
async fn test_contact_isolation_between_users() {
    let pool = setup_test_db().await;

    // Create two users
    let user_a = create_test_user(&pool, "usera@example.com", "User A").await;
    let user_b = create_test_user(&pool, "userb@example.com", "User B").await;

    let contact_repo = ContactRepository::new(&pool);

    // User A creates a contact
    let contact_a = Contact {
        id: Uuid::new_v4(),
        first_name: "Alice".to_string(),
        last_name: Some("Anderson".to_string()),
        email: Some("alice@example.com".to_string()),
        phone: Some("+15555551111".to_string()),
        organization: None,
        title: None,
        notes: None,
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: user_a.id,
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    };

    contact_repo.create(&contact_a).await.unwrap();

    // User B creates a contact
    let contact_b = Contact {
        id: Uuid::new_v4(),
        first_name: "Bob".to_string(),
        last_name: Some("Brown".to_string()),
        email: Some("bob@example.com".to_string()),
        phone: Some("+15555552222".to_string()),
        organization: None,
        title: None,
        notes: None,
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: user_b.id,
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    };

    contact_repo.create(&contact_b).await.unwrap();

    // User A should only see their own contact
    let user_a_contacts = contact_repo.list(10, 0, user_a.id).await.unwrap();
    assert_eq!(user_a_contacts.len(), 1);
    assert_eq!(user_a_contacts[0].first_name, "Alice");

    // User B should only see their own contact
    let user_b_contacts = contact_repo.list(10, 0, user_b.id).await.unwrap();
    assert_eq!(user_b_contacts.len(), 1);
    assert_eq!(user_b_contacts[0].first_name, "Bob");

    // Search should also be isolated
    let user_a_search = contact_repo.search("alice", user_a.id).await.unwrap();
    assert_eq!(user_a_search.len(), 1);

    let user_b_search = contact_repo.search("alice", user_b.id).await.unwrap();
    assert_eq!(user_b_search.len(), 0); // User B cannot find User A's contact

    let user_b_search = contact_repo.search("bob", user_b.id).await.unwrap();
    assert_eq!(user_b_search.len(), 1);

    let user_a_search = contact_repo.search("bob", user_a.id).await.unwrap();
    assert_eq!(user_a_search.len(), 0); // User A cannot find User B's contact
}

#[tokio::test]
async fn test_tag_isolation_between_users() {
    let pool = setup_test_db().await;

    // Create two users
    let user_a = create_test_user(&pool, "usera@example.com", "User A").await;
    let user_b = create_test_user(&pool, "userb@example.com", "User B").await;

    let tag_repo = TagRepository::new(&pool);

    // User A creates a tag
    let tag_a = Tag {
        id: Uuid::new_v4(),
        name: "Important".to_string(),
        color: Some("#FF0000".to_string()),
        created_at: Utc::now(),
        created_by: user_a.id,
    };

    tag_repo.create(&tag_a).await.unwrap();

    // User B creates a tag
    let tag_b = Tag {
        id: Uuid::new_v4(),
        name: "Work".to_string(),
        color: Some("#0000FF".to_string()),
        created_at: Utc::now(),
        created_by: user_b.id,
    };

    tag_repo.create(&tag_b).await.unwrap();

    // User A should only see their own tag
    let user_a_tags = tag_repo.list(user_a.id).await.unwrap();
    assert_eq!(user_a_tags.len(), 1);
    assert_eq!(user_a_tags[0].name, "Important");

    // User B should only see their own tag
    let user_b_tags = tag_repo.list(user_b.id).await.unwrap();
    assert_eq!(user_b_tags.len(), 1);
    assert_eq!(user_b_tags[0].name, "Work");
}

#[tokio::test]
async fn test_system_user_exists() {
    let pool = setup_test_db().await;

    // Create system user as per migration
    let system_user = User {
        id: system_user_id(),
        email: "system@sagenscontact.local".to_string(),
        name: "System".to_string(),
        password_hash: "".to_string(),
        api_token: None,
        email_verified: true,
        active: true,
        preferences: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
    };

    let user_repo = UserRepository::new(&pool);
    user_repo.create(&system_user).await.unwrap();

    // Verify system user can be retrieved
    let retrieved = user_repo.get_by_id(system_user_id()).await.unwrap();
    assert_eq!(retrieved.email, "system@sagenscontact.local");
    assert_eq!(retrieved.name, "System");

    // Verify system user ID is not nil
    assert_ne!(system_user_id(), Uuid::nil());
}

#[tokio::test]
async fn test_system_user_owns_legacy_data() {
    let pool = setup_test_db().await;

    // Create system user
    let system_user = User {
        id: system_user_id(),
        email: "system@sagenscontact.local".to_string(),
        name: "System".to_string(),
        password_hash: "".to_string(),
        api_token: None,
        email_verified: true,
        active: true,
        preferences: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
    };

    let user_repo = UserRepository::new(&pool);
    user_repo.create(&system_user).await.unwrap();

    // Create a contact owned by system user (simulating legacy/imported data)
    let contact_repo = ContactRepository::new(&pool);
    let legacy_contact = Contact {
        id: Uuid::new_v4(),
        first_name: "Legacy".to_string(),
        last_name: Some("Contact".to_string()),
        email: Some("legacy@example.com".to_string()),
        phone: None,
        organization: None,
        title: None,
        notes: None,
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: system_user_id(),
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    };

    contact_repo.create(&legacy_contact).await.unwrap();

    // System user should see the contact
    let system_contacts = contact_repo.list(10, 0, system_user_id()).await.unwrap();
    assert_eq!(system_contacts.len(), 1);
    assert_eq!(system_contacts[0].first_name, "Legacy");

    // Regular users should not see system-owned contacts
    let regular_user = create_test_user(&pool, "regular@example.com", "Regular User").await;
    let regular_contacts = contact_repo.list(10, 0, regular_user.id).await.unwrap();
    assert_eq!(regular_contacts.len(), 0);
}
