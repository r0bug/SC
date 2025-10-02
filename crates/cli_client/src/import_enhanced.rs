use anyhow::{Context, Result};
use import_service::{
    create_default_registry, DeduplicationConfig, DeduplicationEngine, DuplicateStrategy,
    ImportConnector, MatchCriteria,
};
use local_store::{ContactRepository, LocalStore};
use std::path::Path;
use uuid::Uuid;

pub struct ImportOptions {
    pub dry_run: bool,
    pub dedupe_strategy: DuplicateStrategy,
    pub dedupe_criteria: MatchCriteria,
    pub connector_id: Option<String>,
    pub preview_limit: usize,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            dedupe_strategy: DuplicateStrategy::Skip,
            dedupe_criteria: MatchCriteria::EmailOrPhone,
            connector_id: None,
            preview_limit: 5,
        }
    }
}

pub async fn import_file_enhanced(
    file_path: &str,
    store: &LocalStore,
    options: ImportOptions,
) -> Result<()> {
    let path = Path::new(file_path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }

    println!("\n📦 SagensContact Import Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", file_path);

    // Get connector registry
    let registry = create_default_registry();

    // Find appropriate connector
    let connector = if let Some(id) = &options.connector_id {
        registry
            .get(id)
            .with_context(|| format!("Connector '{}' not found", id))?
    } else {
        registry
            .find_connector(path)
            .context("No suitable connector found for this file")?
    };

    let metadata = connector.metadata();
    println!("Connector: {} v{}", metadata.name, metadata.version);
    println!("Format: {:?}", metadata.format);
    println!();

    // Validate file
    println!("🔍 Validating file...");
    connector
        .validate_file(path)
        .await
        .context("File validation failed")?;
    println!("✅ File is valid");

    // Get preview
    println!("\n📊 Preview (first {} rows):", options.preview_limit);
    let preview = connector
        .get_preview(path, options.preview_limit)
        .await
        .context("Failed to preview file")?;

    if !preview.warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &preview.warnings {
            println!("  - {}", warning);
        }
    }

    if preview.rows.is_empty() {
        println!("❌ No data found in file");
        return Ok(());
    }

    for (idx, row) in preview.rows.iter().enumerate() {
        println!("\n  Row {}:", idx + 1);
        for (key, value) in row {
            if !key.starts_with('_') && !value.is_empty() {
                println!("    {}: {}", key, value);
            }
        }
    }

    // Parse full file
    println!("\n⏳ Parsing file...");
    let parse_result = connector
        .parse(path)
        .await
        .context("Failed to parse file")?;

    println!("✅ Parsed {} rows", parse_result.rows.len());

    if let Some(total) = parse_result.metadata.get("total_contacts") {
        println!("📇 Total contacts: {}", total);
    }

    // Apply deduplication
    if !options.dry_run && !matches!(options.dedupe_strategy, DuplicateStrategy::KeepBoth) {
        println!("\n🔄 Checking for duplicates...");
        let dedup_config = DeduplicationConfig {
            strategy: options.dedupe_strategy.clone(),
            match_criteria: options.dedupe_criteria.clone(),
            ..Default::default()
        };

        let dedup_engine = DeduplicationEngine::new(dedup_config);
        let duplicates = dedup_engine.find_duplicates(&parse_result.rows)?;

        if !duplicates.is_empty() {
            println!("   Found {} duplicate(s)", duplicates.len());
            for dup in duplicates.iter().take(5) {
                println!(
                    "   - Row {} matches row {} (score: {:.2}, on: {})",
                    dup.duplicate_index, dup.original_index, dup.match_score, dup.matched_on
                );
            }

            match options.dedupe_strategy {
                DuplicateStrategy::Skip => println!("   Strategy: Skipping duplicates"),
                DuplicateStrategy::Update => println!("   Strategy: Updating existing records"),
                DuplicateStrategy::Merge => println!("   Strategy: Merging data"),
                _ => {}
            }
        } else {
            println!("   No duplicates found");
        }
    }

    if options.dry_run {
        println!("\n✅ Dry run complete - no changes made");
        println!("   Run without --dry-run to import");
        return Ok(());
    }

    // Confirm import
    println!("\n❓ Import {} contacts? [y/N]: ", parse_result.rows.len());
    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    // Perform import
    println!("\n⏳ Importing contacts...");
    let repo = ContactRepository::new(store.pool());
    let mut imported = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for row in parse_result.rows {
        // Skip if marked as needing review
        if row.get("_needs_review") == Some(&"true".to_string()) {
            skipped += 1;
            continue;
        }

        match create_contact_from_row(&row) {
            Ok(contact) => match repo.create(&contact).await {
                Ok(_) => {
                    tracing::info!(
                        "Imported: {} {}",
                        contact.first_name,
                        contact.last_name.as_deref().unwrap_or("")
                    );
                    imported += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to save contact: {}", e);
                    failed += 1;
                }
            },
            Err(e) => {
                tracing::warn!("Failed to create contact from row: {}", e);
                failed += 1;
            }
        }

        // Show progress every 100 contacts
        if (imported + failed) % 100 == 0 {
            println!("   Processed: {} / {}", imported + failed, parse_result.rows.len());
        }
    }

    println!("\n✅ Import complete!");
    println!("   Imported: {}", imported);
    if skipped > 0 {
        println!("   Skipped: {} (marked for review)", skipped);
    }
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    Ok(())
}

fn create_contact_from_row(row: &std::collections::HashMap<String, String>) -> Result<core_domain::Contact> {
    let placeholder_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;

    Ok(core_domain::Contact {
        id: Uuid::new_v4(),
        first_name: row
            .get("first_name")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string()),
        last_name: row.get("last_name").cloned(),
        email: row.get("email").cloned(),
        phone: row.get("phone").cloned(),
        organization: row.get("organization").cloned(),
        title: row.get("title").cloned(),
        notes: row.get("notes").cloned(),
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: placeholder_user_id,
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!(row.get("source").map(|s| serde_json::json!({"source": s})).unwrap_or_default()),
    })
}

pub fn list_connectors() {
    let registry = create_default_registry();
    let connectors = registry.list_connectors();

    println!("\n📦 Available Import Connectors");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for connector in connectors {
        println!("\n🔌 {} (id: {})", connector.name, connector.id);
        println!("   Description: {}", connector.description);
        println!("   Format: {:?}", connector.format);
        println!(
            "   File types: {}",
            connector.supported_extensions.join(", ")
        );
        println!("   Version: {}", connector.version);
    }

    println!("\n💡 Usage: sagenscontact import --file <path> [--connector <id>]");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(!options.dry_run);
        assert_eq!(options.preview_limit, 5);
    }
}
