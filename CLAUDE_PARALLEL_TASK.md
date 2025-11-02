# Parallel Task: AI-Powered Import System - Format Parsers

## Context
You're working on the SagensContact project, a contact management system built with Rust (backend) and SvelteKit/Tauri (frontend). The project is located at `/home/robug/Projects/sagenscontact/alpha`.

## Current Status
Another Claude instance is building the **SmartImporter connector** with AI-powered format detection using the Segmind API. Your task is to build the **format-specific parsers** that the SmartImporter will use.

## Architecture Overview
- **Location**: `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/connectors/`
- **Pattern**: Each parser implements the `ImportConnector` trait
- **Existing Examples**: Check `contacts.rs`, `google_contacts.rs`, `sms.rs` for reference

## Your Tasks

### 1. Create JSON Parser (`json.rs`)
**Path**: `crates/import_service/src/connectors/json.rs`

Requirements:
- Parse JSON files (arrays of objects or nested structures)
- Handle deeply nested structures (flatten or provide path-based access)
- Support both:
  - Simple array format: `[{contact1}, {contact2}]`
  - Nested format: `{"contacts": [{}, {}], "metadata": {}}`
- Extract sample data for AI field mapping
- Return `ParseResult` with:
  - Raw records as HashMap<String, String>
  - Detected fields list
  - Sample rows (first 5)

**Key Features**:
- Use `serde_json` for parsing
- Handle missing fields gracefully
- Support JSONPath-like field access (e.g., `contacts[0].name`)
- Provide metadata about nesting structure

### 2. Create XML Parser (`xml.rs`)
**Path**: `crates/import_service/src/connectors/xml.rs`

Requirements:
- Parse general-purpose XML (not just Android SMS backup)
- Detect repeating elements (likely contact records)
- Support attributes and element values
- Handle namespaces
- Extract sample data for AI mapping

**Key Features**:
- Use `quick_xml` (already in dependencies)
- Detect record pattern automatically (e.g., `<contact>`, `<person>`, `<entry>`)
- Convert XML attributes to fields (e.g., `<contact name="John">` → `name: John`)
- Flatten nested structures with dot notation (`address.street`, `address.city`)

### 3. Create HTML Scraper (`html.rs`)
**Path**: `crates/import_service/src/connectors/html.rs`

Requirements:
- Parse HTML tables (most common format)
- Extract data from structured HTML (lists, divs with classes)
- Support common export formats (LinkedIn, Gmail contacts, etc.)
- Handle missing data gracefully

**Key Features**:
- Use `scraper` crate (add to dependencies if needed: `scraper = "0.17"`)
- Detect tables automatically
- Extract headers from `<thead>` or first `<tr>`
- Handle rowspan/colspan
- Support common patterns:
  - Tables: `<table> → <tr> → <td>`
  - Lists: `<ul> → <li>` with nested data
  - Cards: `<div class="contact">` patterns

### 4. Create Plain Text Parser (`text.rs`)
**Path**: `crates/import_service/src/connectors/text.rs`

Requirements:
- Parse structured text (tab-separated, space-separated, key-value pairs)
- Detect delimiters automatically
- Handle different formats:
  - Columnar (aligned text)
  - Key-value pairs (`Name: John\nEmail: john@email.com`)
  - Simple lists (one contact per line)

**Key Features**:
- Auto-detect delimiter (tab, multiple spaces, comma, semicolon)
- Parse key-value format (common in email signatures)
- Handle line-based records
- Provide confidence score for format detection

## Implementation Template

```rust
use crate::connector::{ImportConnector, ConnectorMetadata, ParseResult};
use anyhow::Result;
use core_domain::Contact;
use std::collections::HashMap;
use std::path::Path;

pub struct JsonImporter;

impl ImportConnector for JsonImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "json".to_string(),
            name: "JSON File".to_string(),
            description: "Import from JSON files with arrays of contact objects".to_string(),
            supported_extensions: vec!["json".to_string()],
            requires_mapping: true,
            supports_preview: true,
        }
    }

    fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        // TODO: Implement JSON parsing
        // 1. Read file
        // 2. Detect structure (array vs object with nested arrays)
        // 3. Extract records
        // 4. Convert to HashMap<String, String> format
        // 5. Return ParseResult with fields, records, samples

        todo!("Implement JSON parser")
    }

    fn map_to_contacts(
        &self,
        records: Vec<HashMap<String, String>>,
        mapping: HashMap<String, String>,
    ) -> Result<Vec<Contact>> {
        // Use the existing mapper logic from contacts.rs
        // This is already implemented in the parent trait
        Ok(vec![])
    }
}
```

## Testing
Create test files in `/home/robug/Projects/sagenscontact/alpha/sample_data/`:
- `contacts.json` - Simple JSON array
- `contacts_nested.json` - Nested structure
- `contacts.xml` - Generic XML
- `contacts.html` - HTML table
- `contacts.txt` - Plain text format

## Integration
After creating parsers, update:
1. `crates/import_service/src/connectors/mod.rs` - Export new parsers
2. `crates/import_service/src/connector.rs` - Add to `create_default_registry()`

## Dependencies (if needed)
Add to `crates/import_service/Cargo.toml`:
```toml
scraper = "0.17"  # For HTML parsing
```

## Questions/Coordination
- SmartImporter will call your parsers based on file extension or content detection
- Your parsers should return raw field data; AI will suggest the mapping
- Focus on robust parsing; don't worry about contact field mapping yet

## Files to Reference
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/connectors/contacts.rs` - CSV parser example
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/connector.rs` - ImportConnector trait
- `/home/robug/Projects/sagenscontact/alpha/crates/core_domain/src/entities.rs` - Contact struct

---

**Priority Order**: JSON → XML → HTML → Text

Start with JSON parser as it's most common and will help establish the pattern for the others.
