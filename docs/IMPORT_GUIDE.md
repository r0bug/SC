# SagensContact Import Guide

## Overview

SagensContact provides a robust, extensible import system that allows you to import contacts from various sources including SMS backups, email exports, contact files, and social network data.

## Quick Start

```bash
# Import a file (auto-detects format)
sagenscontact import --file contacts.csv

# Dry run to preview without importing
sagenscontact import --file contacts.csv --dry-run

# List available connectors
sagenscontact import --list-connectors

# Use a specific connector
sagenscontact import --file data.xml --connector sms
```

## Architecture

### Plugin-Based Connectors

SagensContact uses a plugin-style architecture where each import source is handled by a dedicated connector:

- **SMS Connector** - Android SMS Backup & Restore XML, iOS SMS CSV
- **Email Connector** - Gmail Takeout MBOX, Outlook CSV
- **Google Contacts** - Google Contacts CSV export
- **Apple Contacts** - vCard (.vcf) files
- **LinkedIn** - LinkedIn connections export (Beta)
- **Twitter/X, Facebook, Instagram** - Planned/Stub implementations

Each connector:
1. Validates the input file
2. Parses the data into a common format
3. Suggests field mappings
4. Provides warnings about data quality issues

### Deduplication Engine

The import system includes intelligent deduplication with multiple strategies:

- **Skip** - Ignore duplicate entries
- **Update** - Update existing records with new data
- **Merge** - Intelligently combine fields from both records
- **Keep Both** - Import duplicates as separate contacts
- **Ask** - Flag for manual review

Matching criteria:
- Email address (exact match)
- Phone number (normalized)
- Full name (fuzzy matching)
- Email OR Phone
- Custom field combinations

## Supported Formats

### SMS Messages

**Android SMS Backup & Restore XML**

```xml
<?xml version='1.0' encoding='UTF-8'?>
<smses count="2">
  <sms address="+1234567890" contact_name="John Doe"
       date="1609459200000" type="1" body="Hello!" />
</smses>
```

- **File Extension:** `.xml`
- **Detection:** Looks for `<smses>` or `<sms` tags
- **Fields Extracted:**
  - Phone number
  - Contact name (if available)
  - Message history (stored in notes)
  - Message dates and type (sent/received)

**iOS SMS CSV Export**

```csv
phone_number,contact_name,message,date,is_from_me
+1234567890,Jane Smith,Test message,2024-01-01,false
```

- **File Extension:** `.csv`
- **Detection:** CSV with phone/message columns
- **Fields Extracted:**
  - Phone number
  - Contact name
  - Message content (in notes)
  - Sent/received indicator

### Email

**Gmail Takeout MBOX**

- **File Extension:** `.mbox`
- **What's Extracted:**
  - Email addresses (from/to)
  - Display names
  - Subject lines (in notes)
  - Message dates
- **Privacy:** Only extracts contact info, not message bodies
- **How to Export:**
  1. Go to Google Takeout (takeout.google.com)
  2. Select "Mail"
  3. Choose MBOX format
  4. Download and extract

**Outlook CSV**

```csv
First Name,Last Name,E-mail Address,Business Phone,Company
John,Doe,john@example.com,555-1234,Acme Corp
```

- **File Extension:** `.csv`
- **Detection:** Looks for "E-mail Address" or "Business Phone" columns
- **Fields Mapped:**
  - First/Last Name
  - Email
  - Phone (Business, Mobile, Home)
  - Company → Organization
  - Job Title → Title
  - Notes/Comments

### Contact Files

**Google Contacts CSV**

- **File Extension:** `.csv`
- **Detection:** Looks for "Given Name" or "E-mail 1 - Value" columns
- **Special Handling:**
  - Supports multiple email/phone fields
  - Organization and title extraction
  - Birthday and address fields
  - Notes preservation

**Apple Contacts vCard**

```vcard
BEGIN:VCARD
VERSION:3.0
N:Doe;John;;;
EMAIL:john@example.com
TEL:555-1234
ORG:Acme Corp
END:VCARD
```

- **File Extensions:** `.vcf`, `.vcard`
- **Fields Supported:**
  - N (structured name)
  - FN (full name)
  - EMAIL
  - TEL (phone)
  - ORG (organization)
  - TITLE
  - NOTE
  - ADR (address)
  - URL
  - BDAY (birthday)

**Generic CSV**

- Auto-detects common column names:
  - First/Last/Given/Family name
  - Email/E-mail
  - Phone/Mobile/Cell
  - Company/Organization
  - Title/Job
  - Note/Comment
- Falls back for any CSV that doesn't match specific formats

### Social Networks

**LinkedIn (Beta)**

```csv
First Name,Last Name,Email Address,Company,Position,Connected On
John,Doe,john@example.com,Acme Corp,Engineer,01-Jan-2024
```

- **File:** `Connections.csv` from LinkedIn data export
- **How to Export:**
  1. LinkedIn Settings → Data Privacy
  2. Download your data
  3. Extract and locate `Connections.csv`

**Twitter/X (Planned)**

- **Files:** `following.js`, `followers.js`
- **Status:** Stub implementation
- **Workaround:** Convert JS to JSON, use generic JSON import

**Facebook (Planned)**

- **File:** `friends/friends.json`
- **Note:** Only names and dates available (no email/phone for privacy)

**Instagram (Planned)**

- **Files:** `followers_1.json`, `following.json`
- **Contains:** Usernames and profile links only

## CLI Usage

### Basic Import

```bash
# Auto-detect and import
sagenscontact import --file contacts.csv

# Preview first (recommended)
sagenscontact import --file contacts.csv --dry-run
```

### Deduplication Options

```bash
# Skip duplicates (default)
sagenscontact import --file contacts.csv --dedupe-strategy skip

# Update existing contacts with new data
sagenscontact import --file contacts.csv --dedupe-strategy update

# Merge data from both records
sagenscontact import --file contacts.csv --dedupe-strategy merge

# Keep all as separate contacts
sagenscontact import --file contacts.csv --dedupe-strategy keep-both
```

### Matching Criteria

```bash
# Match by email (default)
sagenscontact import --file contacts.csv --match-by email

# Match by phone
sagenscontact import --file contacts.csv --match-by phone

# Match by name (fuzzy)
sagenscontact import --file contacts.csv --match-by name

# Match by email OR phone
sagenscontact import --file contacts.csv --match-by email-or-phone
```

### Advanced Options

```bash
# Use specific connector
sagenscontact import --file data.xml --connector sms

# Limit preview rows
sagenscontact import --file contacts.csv --preview 10

# Show detailed warnings
sagenscontact import --file contacts.csv --verbose
```

## Web UI Import Wizard

1. **Upload File**
   - Drag & drop or browse
   - Format auto-detection
   - File validation

2. **Preview & Configure**
   - View sample data
   - Review suggested mappings
   - Configure deduplication
   - See warnings

3. **Field Mapping** (if needed)
   - Map source columns to contact fields
   - Set transforms (normalize phone, lowercase email)
   - Mark required fields
   - Set default values

4. **Import Progress**
   - Real-time progress bar
   - Live statistics
   - Error tracking
   - Background processing for large files

5. **Review Results**
   - Import summary
   - Duplicate handling report
   - Failed records list
   - Option to rollback

## Validation & Transformation

### Built-in Validation

- Email format validation
- Phone number normalization
- Required field checks
- Duplicate detection
- Character encoding handling

### Data Transforms

Available transforms:
- `lowercase` - Convert to lowercase
- `uppercase` - Convert to uppercase
- `trim` - Remove whitespace
- `phone_format` - Normalize phone numbers
- `email_normalize` - Lowercase and trim emails
- `date_format` - Convert date formats

Example mapping with transform:

```json
{
  "source_column": "EMAIL",
  "target_field": "email",
  "transform": "email_normalize",
  "required": true
}
```

## Mapping Templates

Save and reuse field mappings for repeated imports:

```bash
# Create template during import
sagenscontact import --file contacts.csv --save-template "my-csv-format"

# Use saved template
sagenscontact import --file new-contacts.csv --template "my-csv-format"

# List templates
sagenscontact import --list-templates

# Delete template
sagenscontact import --delete-template "my-csv-format"
```

## Troubleshooting

### Common Issues

**"No suitable connector found"**
- Check file extension
- Verify file format
- Try specifying connector with `--connector`

**"Duplicate detection not working"**
- Check match criteria
- Verify fields exist in data
- Try different matching strategy

**"Import failed with validation errors"**
- Run with `--dry-run` first
- Check required fields
- Review warnings in output

### Import Best Practices

1. **Always dry-run first**
   ```bash
   sagenscontact import --file contacts.csv --dry-run
   ```

2. **Start with a small sample**
   - Test with 10-20 records first
   - Verify mappings and results
   - Then import full dataset

3. **Back up existing data**
   ```bash
   sagenscontact export --format json --output backup.json
   ```

4. **Use templates for repeated imports**
   - Saves time
   - Ensures consistency
   - Reduces errors

5. **Review deduplication settings**
   - Choose appropriate strategy
   - Test with known duplicates
   - Verify merge behavior

## Privacy & Security

- **Local Processing:** All imports processed locally, no cloud upload
- **Selective Import:** Choose which fields to import
- **Message Content:** Can strip SMS/email content, keep only contact info
- **Data Minimization:** Import only necessary fields
- **Rollback Support:** Undo imports if needed

## API Integration

For programmatic imports:

```bash
# REST API
POST /api/import/preview
POST /api/import/execute
GET /api/import/connectors
GET /api/import/templates
```

See [API Documentation](API.md) for details.

## Extending the System

### Creating Custom Connectors

Implement the `ImportConnector` trait:

```rust
use import_service::{ImportConnector, ParseResult};

pub struct MyCustomConnector;

#[async_trait]
impl ImportConnector for MyCustomConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "my-connector".to_string(),
            name: "My Custom Format".to_string(),
            // ...
        }
    }

    async fn parse(&self, file_path: &Path)
        -> Result<ParseResult, ImportError> {
        // Your parsing logic
    }
}
```

Register your connector:

```rust
let mut registry = create_default_registry();
registry.register(Box::new(MyCustomConnector::new()));
```

## Support

- **Issues:** https://github.com/r0bug/SC/issues
- **Email:** john@robug.com
- **Documentation:** See `docs/` directory

---

**Version:** 0.1.0-alpha
**Last Updated:** 2024-10-02
