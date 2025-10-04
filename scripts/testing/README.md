# SagensContact Testing Suite

Quick reference for running all Week 3-4 tests.

---

## 📋 Quick Start

```bash
# Generate all test data (10k, 50k, 100k)
python3 scripts/testing/generate_test_data.py all

# Run security audit
./scripts/testing/security_audit.sh

# View test data
ls -lh sample_data/load_tests/
```

---

## 🧪 Test Data Generation

### Generate All Standard Datasets
```bash
python3 scripts/testing/generate_test_data.py all
```

Output:
- `sample_data/load_tests/contacts_10k.csv` (0.8MB)
- `sample_data/load_tests/contacts_100k.csv` (8.2MB)
- `sample_data/load_tests/contacts_50k_with_duplicates.csv` (4.1MB)

### Generate Individual Datasets
```bash
# 10k contacts
python3 scripts/testing/generate_test_data.py 10k

# 100k contacts
python3 scripts/testing/generate_test_data.py 100k

# 50k with 20% duplicates
python3 scripts/testing/generate_test_data.py 50k-dupes

# 1M contacts (stress test - takes ~2 minutes)
python3 scripts/testing/generate_test_data.py 1m
```

---

## 🔒 Security Audit

### Run Full Security Audit
```bash
chmod +x scripts/testing/security_audit.sh
./scripts/testing/security_audit.sh
```

### View Results
```bash
# Summary
cat test_results/security/SUMMARY.txt

# Individual reports
ls test_results/security/
cat test_results/security/cargo_audit.txt
cat test_results/security/unsafe_code.txt
cat test_results/security/sql_injection_check.txt
```

### Manual Checks
```bash
# Check for vulnerabilities
cargo audit

# Check for unsafe code
cargo geiger --all-features

# Check dependencies
cargo tree
```

---

## 📊 Load Testing

### Prerequisites
```bash
# Build CLI tool
cargo build --release --bin cli_client

# Verify CLI exists
ls target/release/cli_client
```

### Run Load Tests
```bash
chmod +x scripts/testing/run_load_tests.sh
./scripts/testing/run_load_tests.sh
```

### View Results
```bash
# All results
cat test_results/*_results.txt

# Specific test
cat test_results/100k_import_results.txt
```

### Manual Import Tests
```bash
# 10k import
time target/release/cli_client import --file sample_data/load_tests/contacts_10k.csv

# 100k import with memory tracking
/usr/bin/time -v target/release/cli_client import --file sample_data/load_tests/contacts_100k.csv

# Dry run (no database changes)
target/release/cli_client import --file sample_data/load_tests/contacts_10k.csv --dry-run
```

---

## 🌐 Browser Testing

### Local Testing
1. Start the application:
   ```bash
   # Terminal 1: Start backend
   cd /home/robug/Projects/sagenscontact/alpha
   cargo run --release --bin sync_service

   # Terminal 2: Start frontend
   cd apps/web
   npm run dev
   ```

2. Open browsers:
   - Chrome: http://localhost:3001/import
   - Firefox: http://localhost:3001/import
   - Safari: http://localhost:3001/import
   - Edge: http://localhost:3001/import

3. Test each page:
   - `/import` - Import wizard
   - `/import/jobs` - Job tracking
   - `/import/history` - Import history

### Manual Test Checklist
- [ ] File upload works
- [ ] Drag-and-drop works
- [ ] Preview displays correctly
- [ ] Import executes
- [ ] Progress updates in real-time
- [ ] Job dashboard shows active jobs
- [ ] History page loads
- [ ] Filters work
- [ ] Mobile layout works
- [ ] Error handling works

---

## ⚡ Performance Benchmarks

### Rust Benchmarks

#### Run All Benchmarks
```bash
# Run import service benchmarks
cargo bench --package import_service

# Run sync service benchmarks
cargo bench --package sync_service
```

#### View Results
```bash
# Open in browser
open target/criterion/report/index.html

# Command line summary
cargo bench --package import_service -- --baseline
```

### Frontend Performance

#### Lighthouse (Chrome DevTools)
1. Open Chrome DevTools (F12)
2. Go to Lighthouse tab
3. Select "Performance" + "Best practices" + "Accessibility"
4. Click "Analyze page load"
5. Review scores (target: >90 for all)

#### WebPageTest
1. Visit https://www.webpagetest.org/
2. Enter URL: http://localhost:3001/import
3. Run test
4. Review metrics:
   - FCP (First Contentful Paint)
   - LCP (Largest Contentful Paint)
   - TTI (Time to Interactive)

---

## 🔍 Database Analysis

### Check Database Size
```bash
# Database file size
du -h data/contacts.db

# Table sizes (SQLite)
sqlite3 data/contacts.db "SELECT name, COUNT(*) FROM sqlite_master WHERE type='table' GROUP BY name;"

# Contact count
sqlite3 data/contacts.db "SELECT COUNT(*) FROM contacts;"
```

### Query Performance
```bash
# Explain query plan
sqlite3 data/contacts.db "EXPLAIN QUERY PLAN SELECT * FROM contacts WHERE email LIKE '%test%' LIMIT 10;"

# Check indexes
sqlite3 data/contacts.db ".indexes"

# Table info
sqlite3 data/contacts.db ".schema contacts"
```

---

## 📈 Test Results

### Expected Performance

| Metric | Target | Notes |
|--------|--------|-------|
| Import Speed | >1000/sec | 100k in <2min |
| Memory Usage | <500MB | Peak during 100k import |
| DB Size (100k) | ~50MB | Contacts only |
| Search (p95) | <100ms | Full-text search |
| API (p95) | <100ms | All endpoints |

### Success Criteria

- ✅ Import 100k contacts in <2 minutes
- ✅ Memory stays under 500MB
- ✅ Zero critical security issues
- ✅ Works in Chrome, Firefox, Safari
- ✅ Mobile responsive (Lighthouse mobile >90)

---

## 🐛 Troubleshooting

### Test Data Generation Fails
```bash
# Check Python version (need 3.6+)
python3 --version

# Install if missing
sudo apt-get install python3
```

### Security Audit Tools Missing
```bash
# Install cargo-audit
cargo install cargo-audit

# Install cargo-geiger
cargo install cargo-geiger
```

### Load Tests Fail
```bash
# Rebuild CLI
cargo build --release --bin cli_client

# Check database permissions
ls -l data/

# Reset database
rm data/contacts.db
cargo run --bin sync_service  # Will recreate on start
```

### Browser Tests Fail
```bash
# Check if services are running
curl http://localhost:3002/health
curl http://localhost:3001/

# Restart services
pkill sync_service
pkill node
cargo run --release --bin sync_service &
cd apps/web && npm run dev &
```

---

## 📝 Documentation

- **Test Results:** `docs/WEEK_3-4_TEST_RESULTS.md`
- **Sprint Summary:** `docs/SPRINT_SUMMARY.md`
- **Security Reports:** `test_results/security/`
- **Load Test Results:** `test_results/*_results.txt`

---

## 🎯 Quick Commands Reference

```bash
# === Test Data ===
python3 scripts/testing/generate_test_data.py all

# === Security ===
./scripts/testing/security_audit.sh
cargo audit
cargo geiger

# === Load Tests ===
./scripts/testing/run_load_tests.sh
time target/release/cli_client import --file sample_data/load_tests/contacts_100k.csv

# === Benchmarks ===
cargo bench --package import_service
open target/criterion/report/index.html

# === Database ===
sqlite3 data/contacts.db "SELECT COUNT(*) FROM contacts;"
du -h data/contacts.db

# === Services ===
cargo run --release --bin sync_service
cd apps/web && npm run dev

# === Results ===
cat test_results/security/SUMMARY.txt
cat test_results/100k_import_results.txt
ls -lh sample_data/load_tests/
```

---

**Last Updated:** 2025-10-03
**Next Steps:** Execute tests and document actual results
