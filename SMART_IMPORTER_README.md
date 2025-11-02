# SmartImporter - AI-Powered Contact Import System

## Overview

The **SmartImporter** is an AI-powered import connector that uses Segmind AI to:

1. **Detect file formats** automatically (CSV, JSON, XML, HTML, text)
2. **Analyze data structure** to understand field organization
3. **Suggest field mappings** to contact fields intelligently
4. **Provide confidence scores** for format detection

## How It Works

### Phase 1: Sample Reading
- Reads first 2000 bytes of any file
- Cuts at newline for better analysis
- Works with text-based formats

### Phase 2: AI Analysis
Sends sample to Segmind AI with structured prompt:
```
- Detect format (csv|json|xml|html|text|unknown)
- Extract field names
- Suggest mappings to contact fields
- Provide confidence score (0-1)
```

### Phase 3: Result Processing
- Parses JSON response from AI
- Falls back to text extraction if JSON parsing fails
- Returns `ParseResult` with suggested mappings

## Supported Formats

- **CSV**: Comma-separated values
- **JSON**: Contact objects or arrays
- **XML**: Generic XML structures
- **HTML**: Tables and structured content
- **Text**: Plain text with delimiters

## Usage

### Basic Usage

```rust
use ai_middleware::SegmindClient;
use import_service::connectors::SmartImporter;

// Create Segmind client (use None for mock mode)
let ai_client = SegmindClient::new(Some("your-api-key".to_string()));

// Create SmartImporter
let importer = SmartImporter::new(ai_client);

// Parse a file
let result = importer.parse(Path::new("contacts.json")).await?;

// Check detected format
println!("Detected: {:?}", result.metadata.get("detected_format"));
println!("Confidence: {}", result.metadata.get("confidence").unwrap());

// Review suggested mappings
for (source, target) in result.suggested_mappings {
    println!("  {} → {}", source, target);
}
```

### Registry Integration

```rust
use ai_middleware::SegmindClient;
use import_service::connectors::create_smart_registry;

let ai_client = SegmindClient::new(None); // Mock mode
let registry = create_smart_registry(ai_client);

// SmartImporter is registered FIRST and will handle unknown formats
let connector = registry.find_connector(Path::new("unknown_data.txt"));
```

## Target Contact Fields

The AI is instructed to map to these standard contact fields:

### Names
- `first_name`, `last_name`, `full_name`

### Contact Info
- `email`, `phone`, `mobile`, `work_phone`

### Professional
- `organization`, `title`, `department`

### Address
- `address`, `city`, `state`, `postal_code`, `country`

### Social & Web
- `website`, `linkedin_url`, `twitter_handle`

### Other
- `birthday`, `notes`, `tags`

## Mock Mode

When no Segmind API key is provided, SmartImporter runs in **mock mode**:

```rust
let ai_client = SegmindClient::new(None); // No API key = mock mode
```

Mock mode:
- Simulates 200ms API delay
- Returns generic suggestions
- Perfect for testing and development

## AI Response Format

The AI is expected to return JSON:

```json
{
  "format": "csv",
  "confidence": 0.95,
  "detected_fields": ["First Name", "Last Name", "Email", "Phone"],
  "suggested_mappings": {
    "First Name": "first_name",
    "Last Name": "last_name",
    "Email": "email",
    "Phone": "phone"
  },
  "structure_notes": "CSV file with header row, 4 columns",
  "record_separator": "newline"
}
```

## Configuration

### Custom Sample Size

```rust
let importer = SmartImporter::new(ai_client)
    .with_sample_size(5000); // Read 5000 bytes instead of 2000
```

### Confidence Threshold

Results with confidence < 0.7 include a warning:
> "Low confidence format detection (65%). Please verify the suggested mappings."

## Architecture

```
SmartImporter
    ├── read_sample() - Read first N bytes
    ├── analyze_with_ai() - Send to Segmind
    │   ├── parse_ai_response() - Extract JSON
    │   └── extract_format_from_text() - Fallback parser
    └── parse() - Main entry point
```

## Integration with Other Connectors

The SmartImporter works alongside format-specific parsers:

```
File Input
    ↓
SmartImporter (detect format)
    ↓
Format-Specific Parser (JSON/XML/HTML/Text)
    ↓
Contact Records
```

**Workflow:**
1. SmartImporter analyzes file and suggests format
2. User confirms or adjusts format selection
3. Format-specific parser extracts actual data
4. User reviews and adjusts field mappings
5. Import executes with deduplication

## Next Steps

The parallel Claude instance is building format-specific parsers:

- `json.rs` - Parse JSON arrays and nested objects
- `xml.rs` - Parse generic XML with repeating elements
- `html.rs` - Parse HTML tables and structured content
- `text.rs` - Parse delimited text and key-value pairs

Once these are complete, the full AI-powered import workflow will be:

```
1. User selects file
2. SmartImporter detects format → AI suggests mappings
3. UI shows 3-step wizard:
   - Step 1: Format confirmation (with confidence score)
   - Step 2: Field mapping adjustment
   - Step 3: Preview and import
4. Format-specific parser extracts data
5. Deduplication engine prevents duplicates
6. Batch validator checks data quality
7. Transaction commits to database
```

## Files Created/Modified

### Created
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/connectors/smart.rs` (363 lines)

### Modified
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/config.rs` - Added `Xml`, `Html`, `Text`, `Unknown` to `ImportFormat` enum
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/src/connectors/mod.rs` - Exported SmartImporter, added `create_smart_registry()`
- `/home/robug/Projects/sagenscontact/alpha/crates/import_service/Cargo.toml` - Added `ai_middleware` dependency

## Testing

```bash
# Run SmartImporter tests
cargo test -p import_service smart

# Test with sample file
cd /home/robug/Projects/sagenscontact/alpha
cargo run --bin sagenscontact -- analyze-import sample_data/contacts.csv
```

## Security Notes

- AI responses are sanitized for JSON extraction
- Falls back to safe text parsing if JSON invalid
- File size limited to prevent memory exhaustion
- Only reads first N bytes for analysis

## Performance

- **Sample read**: <10ms for most files
- **AI analysis** (mock): ~200ms
- **AI analysis** (real): ~500-2000ms (depends on Segmind API)
- **Caching**: 1 hour TTL, reduces repeated analysis

---

Built with:
- Rust async/await with Tokio
- Segmind AI API integration
- Serde JSON parsing
- Pattern matching for fallback detection
