use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use uuid::Uuid;

// Simulate database query building
fn build_contact_query(filters: &HashMap<String, String>) -> String {
    let mut query = String::from("SELECT * FROM contacts WHERE 1=1");

    for (key, value) in filters {
        query.push_str(&format!(" AND {} = '{}'", key, value));
    }

    query
}

// Benchmark query building
fn bench_query_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_building");

    for filter_count in [0, 1, 5, 10].iter() {
        let mut filters = HashMap::new();
        for i in 0..*filter_count {
            filters.insert(format!("field_{}", i), format!("value_{}", i));
        }

        group.bench_with_input(
            BenchmarkId::new("build_query", filter_count),
            &filters,
            |b, filters| b.iter(|| build_contact_query(black_box(filters))),
        );
    }

    group.finish();
}

// Simulate pagination calculations
fn calculate_pagination(total_items: usize, page_size: usize, page: usize) -> (usize, usize) {
    let offset = page * page_size;
    let limit = page_size.min(total_items.saturating_sub(offset));
    (offset, limit)
}

// Benchmark pagination calculations
fn bench_pagination(c: &mut Criterion) {
    let mut group = c.benchmark_group("pagination");

    for total in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("calculate", total), total, |b, &total| {
            b.iter(|| calculate_pagination(black_box(total), black_box(50), black_box(0)))
        });
    }

    group.finish();
}

// Simulate ID validation
fn validate_uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok()
}

// Benchmark UUID validation
fn bench_uuid_validation(c: &mut Criterion) {
    let valid_uuid = Uuid::new_v4().to_string();
    let invalid_uuid = "not-a-uuid";

    c.bench_function("validate_valid_uuid", |b| {
        b.iter(|| validate_uuid(black_box(&valid_uuid)))
    });

    c.bench_function("validate_invalid_uuid", |b| {
        b.iter(|| validate_uuid(black_box(invalid_uuid)))
    });
}

// Simulate tag intersection (finding contacts with multiple tags)
fn find_contacts_with_all_tags(
    contact_tags: &HashMap<String, Vec<String>>,
    required_tags: &[String],
) -> Vec<String> {
    contact_tags
        .iter()
        .filter(|(_, tags)| required_tags.iter().all(|rt| tags.contains(rt)))
        .map(|(id, _)| id.clone())
        .collect()
}

// Benchmark tag operations
fn bench_tag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_operations");

    for contact_count in [100, 500, 1000].iter() {
        let mut contact_tags = HashMap::new();
        for i in 0..*contact_count {
            let tags = vec![
                format!("tag_{}", i % 5),
                format!("tag_{}", i % 10),
                format!("tag_{}", i % 20),
            ];
            contact_tags.insert(Uuid::new_v4().to_string(), tags);
        }

        let required_tags = vec!["tag_0".to_string(), "tag_1".to_string()];

        group.bench_with_input(
            BenchmarkId::new("find_with_all_tags", contact_count),
            &contact_tags,
            |b, contact_tags| {
                b.iter(|| {
                    find_contacts_with_all_tags(black_box(contact_tags), black_box(&required_tags))
                })
            },
        );
    }

    group.finish();
}

// Simulate text search (case-insensitive substring match)
fn search_contacts_by_name(contacts: &HashMap<String, String>, query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    contacts
        .iter()
        .filter(|(_, name)| name.to_lowercase().contains(&query_lower))
        .map(|(id, _)| id.clone())
        .collect()
}

// Benchmark search operations
fn bench_search_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_operations");

    for contact_count in [100, 500, 1000].iter() {
        let mut contacts = HashMap::new();
        for i in 0..*contact_count {
            contacts.insert(Uuid::new_v4().to_string(), format!("Person Name {}", i));
        }

        group.bench_with_input(
            BenchmarkId::new("search_by_name", contact_count),
            &contacts,
            |b, contacts| {
                b.iter(|| search_contacts_by_name(black_box(contacts), black_box("Person")))
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_query_building,
    bench_pagination,
    bench_uuid_validation,
    bench_tag_operations,
    bench_search_operations
);
criterion_main!(benches);
