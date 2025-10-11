use chrono::Utc;
use core_domain::entities::{Contact, Project, User};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use uuid::Uuid;

// Benchmark contact serialization/deserialization
fn bench_contact_serialization(c: &mut Criterion) {
    let contact = Contact {
        id: Uuid::new_v4().to_string(),
        first_name: "John".to_string(),
        last_name: Some("Doe".to_string()),
        email: Some("john.doe@example.com".to_string()),
        phone: Some("+1234567890".to_string()),
        organization: Some("Acme Corp".to_string()),
        title: Some("Software Engineer".to_string()),
        notes: Some("Important contact".to_string()),
        social_handles: vec![],
        tags: vec!["vip".to_string(), "engineering".to_string()],
        projects: vec![],
        groups: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Uuid::new_v4().to_string(),
        version: 1,
        last_synced_at: None,
        metadata: serde_json::Map::new(),
    };

    c.bench_function("serialize_contact", |b| {
        b.iter(|| serde_json::to_string(black_box(&contact)).unwrap())
    });

    let json_str = serde_json::to_string(&contact).unwrap();
    c.bench_function("deserialize_contact", |b| {
        b.iter(|| serde_json::from_str::<Contact>(black_box(&json_str)).unwrap())
    });
}

// Benchmark JSON value operations
fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_operations");

    for size in [10, 100, 1000].iter() {
        let contacts: Vec<Contact> = (0..*size)
            .map(|i| Contact {
                id: Uuid::new_v4().to_string(),
                first_name: format!("User{}", i),
                last_name: Some("Test".to_string()),
                email: Some(format!("user{}@example.com", i)),
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
                created_by: Uuid::new_v4().to_string(),
                version: 1,
                last_synced_at: None,
                metadata: serde_json::Map::new(),
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("serialize_array", size),
            &contacts,
            |b, contacts| b.iter(|| serde_json::to_string(black_box(contacts)).unwrap()),
        );

        let json_str = serde_json::to_string(&contacts).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize_array", size),
            &json_str,
            |b, json_str| {
                b.iter(|| serde_json::from_str::<Vec<Contact>>(black_box(json_str)).unwrap())
            },
        );
    }

    group.finish();
}

// Benchmark UUID generation and string conversion
fn bench_uuid_operations(c: &mut Criterion) {
    c.bench_function("uuid_generation", |b| b.iter(|| black_box(Uuid::new_v4())));

    c.bench_function("uuid_to_string", |b| {
        let uuid = Uuid::new_v4();
        b.iter(|| black_box(uuid.to_string()))
    });

    let uuid_str = Uuid::new_v4().to_string();
    c.bench_function("uuid_from_string", |b| {
        b.iter(|| Uuid::parse_str(black_box(&uuid_str)).unwrap())
    });
}

// Benchmark tag filtering operations (common in contact management)
fn bench_tag_filtering(c: &mut Criterion) {
    let contacts: Vec<Contact> = (0..1000)
        .map(|i| Contact {
            id: Uuid::new_v4().to_string(),
            first_name: format!("User{}", i),
            last_name: Some("Test".to_string()),
            email: Some(format!("user{}@example.com", i)),
            phone: None,
            organization: None,
            title: None,
            notes: None,
            social_handles: vec![],
            tags: vec![
                if i % 2 == 0 { "vip" } else { "regular" }.to_string(),
                if i % 3 == 0 { "engineering" } else { "sales" }.to_string(),
            ],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4().to_string(),
            version: 1,
            last_synced_at: None,
            metadata: serde_json::Map::new(),
        })
        .collect();

    c.bench_function("filter_contacts_by_tag", |b| {
        b.iter(|| {
            black_box(&contacts)
                .iter()
                .filter(|c| c.tags.contains(&"vip".to_string()))
                .count()
        })
    });

    c.bench_function("filter_contacts_by_multiple_tags", |b| {
        b.iter(|| {
            black_box(&contacts)
                .iter()
                .filter(|c| {
                    c.tags.contains(&"vip".to_string())
                        && c.tags.contains(&"engineering".to_string())
                })
                .count()
        })
    });
}

// Benchmark data cloning (common when caching)
fn bench_data_cloning(c: &mut Criterion) {
    let contact = Contact {
        id: Uuid::new_v4().to_string(),
        first_name: "John".to_string(),
        last_name: Some("Doe".to_string()),
        email: Some("john.doe@example.com".to_string()),
        phone: Some("+1234567890".to_string()),
        organization: Some("Acme Corp".to_string()),
        title: Some("Software Engineer".to_string()),
        notes: Some("Important contact with lots of notes and information".to_string()),
        social_handles: vec![],
        tags: vec![
            "vip".to_string(),
            "engineering".to_string(),
            "manager".to_string(),
        ],
        projects: vec![Uuid::new_v4().to_string(), Uuid::new_v4().to_string()],
        groups: vec![Uuid::new_v4().to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Uuid::new_v4().to_string(),
        version: 1,
        last_synced_at: None,
        metadata: serde_json::Map::new(),
    };

    c.bench_function("clone_contact", |b| b.iter(|| black_box(&contact).clone()));
}

criterion_group!(
    benches,
    bench_contact_serialization,
    bench_json_operations,
    bench_uuid_operations,
    bench_tag_filtering,
    bench_data_cloning
);
criterion_main!(benches);
