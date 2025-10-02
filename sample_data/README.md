# Sample Data

This directory contains sample data files for testing SagensContact import functionality.

## Files

### contacts.csv
Standard CSV format with the following fields:
- first_name (required)
- last_name (optional)
- email (optional)
- phone (optional)
- organization (optional)
- title (optional)

### contacts.vcf
vCard 3.0 format with two sample contacts including:
- Standard contact fields (name, email, phone, organization)
- Social media profiles (X-SOCIALPROFILE extension)
- Notes field

### sms_export.json
Mock SMS conversation export with structure:
- conversations: Array of conversation threads
  - contact_name: Contact identifier
  - phone: Phone number
  - messages: Array of messages
    - timestamp: ISO 8601 format
    - direction: "incoming" or "outgoing"
    - text: Message content

## Usage

Import sample data using the CLI:

```bash
sagenscontact import --csv sample_data/contacts.csv
sagenscontact import --vcard sample_data/contacts.vcf
sagenscontact import --sms sample_data/sms_export.json
```

## Demo Scenario: Military Artifact Sharing

The sample data represents a scenario where contacts are collaborating on military artifact authentication and sharing:

1. **Robert Johnson** (Military) - Source contact with artifact information
2. **Emily Davis** (Museum Curator) - Receiving shared artifact documentation
3. **James Martinez** (Archivist) - Providing historical context and authentication

The SMS conversations show the workflow of information sharing and collaboration around artifact provenance.