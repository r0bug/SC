# SagensContact - Weeks 1-4 Sprint Plan

## Executive Summary

This document outlines the 4-week sprint to polish SagensContact Alpha and prepare for beta release. The plan focuses on **UI completion, testing infrastructure, and performance validation**.

---

## ✅ Current State (Completed)

### Import Subsystem
- ✅ **9 Connectors** (SMS, Email, Google/Apple/Generic contacts, LinkedIn + 3 stubs)
- ✅ **Background Job API** (`/api/import/*` routes)
- ✅ **Audit System** (5 database tables for logging/rollback)
- ✅ **Deduplication Engine** (5 strategies, 4 match criteria)
- ✅ **CLI Tool** (`sagenscontact import`)
- ✅ **Sample Data** (Google CSV, Android SMS, LinkedIn)
- ✅ **13 Tests** passing
- ✅ **Documentation** (50+ pages)

### Existing UI
- ✅ **Import page exists** (`apps/web/src/routes/import/+page.svelte`)
- ✅ **File upload** with validation
- ✅ **Preview functionality** with field mapping
- ✅ **Progress tracking** (simulated)
- ✅ **Error handling** with toast notifications
- ⚠️ **Uses mocked API** (needs integration with real backend)

---

## 📅 Week 1-2: UI Polish

### Objectives
1. Connect import UI to real API routes
2. Add drag-and-drop functionality
3. Build job tracking dashboard
4. Create import history page
5. Mobile responsive fixes

### Tasks

#### 1.1 Update Import API Integration (3 days)
**File:** `apps/web/src/lib/api/api.ts`

```typescript
// Add real API methods
export const importApi = {
  async getConnectors(): Promise<Connector[]> {
    return await fetch('/api/import/connectors').then(r => r.json());
  },

  async previewFile(file: File): Promise<PreviewResponse> {
    const formData = new FormData();
    formData.append('file', file);
    return await fetch('/api/import/preview?limit=10', {
      method: 'POST',
      body: formData
    }).then(r => r.json());
  },

  async executeImport(file: File, config: ImportConfig): Promise<JobResponse> {
    const formData = new FormData();
    formData.append('file', file);
    // Send config as JSON in separate request or multipart
    return await fetch('/api/import/execute', {
      method: 'POST',
      body: formData
    }).then(r => r.json());
  },

  async getJobStatus(jobId: string): Promise<ImportJob> {
    return await fetch(`/api/import/jobs/${jobId}`).then(r => r.json());
  },

  async listJobs(): Promise<ImportJob[]> {
    return await fetch('/api/import/jobs').then(r => r.json());
  },

  async cancelJob(jobId: string): Promise<void> {
    await fetch(`/api/import/jobs/${jobId}/cancel`, { method: 'POST' });
  }
};
```

#### 1.2 Add Drag-and-Drop (1 day)
**File:** `apps/web/src/routes/import/+page.svelte`

```svelte
<script>
  let dragActive = false;

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragActive = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
    const files = e.dataTransfer?.files;
    if (files?.[0]) {
      handleFile(files[0]);
    }
  }
</script>

<div
  class:drag-active={dragActive}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}>
  <!-- Upload UI -->
</div>
```

#### 1.3 Job Tracking Dashboard (2 days)
**New File:** `apps/web/src/routes/import/jobs/+page.svelte`

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { importApi } from '$lib/api/api';

  let jobs: ImportJob[] = [];
  let pollInterval: number;

  onMount(async () => {
    await loadJobs();
    // Poll every 2 seconds for active jobs
    pollInterval = setInterval(async () => {
      const hasActive = jobs.some(j =>
        j.status === 'pending' || j.status === 'importing'
      );
      if (hasActive) {
        await loadJobs();
      }
    }, 2000);
  });

  onDestroy(() => {
    clearInterval(pollInterval);
  });

  async function loadJobs() {
    jobs = await importApi.listJobs();
  }

  async function cancelJob(jobId: string) {
    await importApi.cancelJob(jobId);
    await loadJobs();
  }
</script>

<div class="jobs-dashboard">
  <h1>Import Jobs</h1>

  <div class="jobs-grid">
    {#each jobs as job}
      <div class="job-card" class:active={job.status === 'importing'}>
        <div class="job-header">
          <h3>{job.file_name}</h3>
          <span class="status-badge status-{job.status}">
            {job.status}
          </span>
        </div>

        <div class="job-progress">
          <div class="progress-bar">
            <div class="progress-fill"
              style="width: {(job.progress.current / job.progress.total) * 100}%">
            </div>
          </div>
          <p>{job.progress.phase}</p>
          <p class="progress-text">
            {job.progress.current} / {job.progress.total}
          </p>
        </div>

        {#if job.result}
          <div class="job-results">
            <span>✓ {job.result.imported} imported</span>
            <span>⊘ {job.result.skipped} skipped</span>
            <span>✗ {job.result.failed} failed</span>
          </div>
        {/if}

        {#if job.status === 'importing'}
          <button on:click={() => cancelJob(job.id)}>
            Cancel
          </button>
        {/if}
      </div>
    {/each}
  </div>
</div>
```

#### 1.4 Import History Page (2 days)
**New File:** `apps/web/src/routes/import/history/+page.svelte`

Queries the `import_logs` table to show:
- Past imports with filters (date range, status, connector)
- Detailed results (counts, errors, warnings)
- Rollback capability
- Download error/warning reports
- Re-run with same configuration

#### 1.5 Mobile Responsive Fixes (1 day)
Update CSS with mobile-first approach:
```css
/* Base mobile styles */
.import-wizard {
  padding: 1rem;
}

/* Tablet */
@media (min-width: 768px) {
  .import-wizard {
    padding: 2rem;
  }
  .results-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .import-wizard {
    max-width: 1000px;
    margin: 0 auto;
  }
  .results-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}
```

---

## 🧪 Week 3-4: Testing & Performance

### Objectives
1. Generate and test with 100k contact dataset
2. Security audit with automated tools
3. Browser compatibility testing
4. Performance benchmarking

### Tasks

#### 3.1 Load Testing (3 days)

**Generate Test Data:**
```bash
# Create test data generator
cat > scripts/generate_test_data.sh <<'EOF'
#!/bin/bash
# Generate 100k contact CSV

OUTPUT="sample_data/contacts_100k.csv"

echo "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name" > $OUTPUT

for i in {1..100000}; do
  FIRST_NAME="User$i"
  LAST_NAME="Test$i"
  EMAIL="user${i}@test${RANDOM}.com"
  PHONE="+1-555-$(printf '%04d' $((RANDOM % 10000)))"
  ORG="Company $((i % 1000))"

  echo "$FIRST_NAME,$LAST_NAME,$EMAIL,$PHONE,$ORG" >> $OUTPUT

  if [ $((i % 10000)) -eq 0 ]; then
    echo "Generated $i rows..."
  fi
done

echo "✓ Generated $OUTPUT"
EOF

chmod +x scripts/generate_test_data.sh
./scripts/generate_test_data.sh
```

**Load Test Plan:**
1. Import 100k contacts via CLI
2. Monitor memory usage (top/htop)
3. Check database size growth
4. Measure import speed (rows/sec)
5. Test search performance after import
6. Verify deduplication at scale

**Expected Results:**
- Import rate: >1000 contacts/sec
- Memory usage: <500MB peak
- Search latency: <100ms (p95)
- Database size: ~50MB for 100k contacts

**Tools:**
```bash
# Memory profiling
/usr/bin/time -v ~/.cargo/bin/cargo run --release --bin cli_client \
  import --file sample_data/contacts_100k.csv

# Database analysis
sqlite3 data/contacts.db "SELECT
  COUNT(*) as total_contacts,
  pg_size_pretty(pg_database_size('contacts.db')) as db_size,
  COUNT(DISTINCT email) as unique_emails
FROM contacts;"

# Query performance
sqlite3 data/contacts.db "EXPLAIN QUERY PLAN
  SELECT * FROM contacts
  WHERE first_name LIKE '%John%'
  LIMIT 100;"
```

#### 3.2 Security Audit (2 days)

**Automated Tools:**
```bash
# Install security audit tools
cargo install cargo-audit cargo-geiger

# Rust dependency audit
cargo audit --deny warnings

# Unsafe code detection
cargo geiger --all-features

# SQL injection testing (manual review)
# Check all sqlx queries use parameterized statements

# XSS prevention (SvelteKit auto-escapes, verify custom HTML)
grep -r "innerHTML" apps/web/src/

# CORS configuration review
grep -r "cors" crates/sync_service/src/

# Rate limit testing
ab -n 10000 -c 100 http://localhost:3002/api/contacts
```

**Security Checklist:**
- [ ] No SQL injection vulnerabilities
- [ ] XSS prevention (escaped output)
- [ ] CSRF tokens (if using cookies)
- [ ] Rate limiting effective
- [ ] Input validation on all endpoints
- [ ] Secrets not in code/logs
- [ ] HTTPS enforced in production
- [ ] Security headers present
- [ ] Dependencies up to date
- [ ] No unsafe Rust code

#### 3.3 Browser Compatibility (1 day)

**Test Matrix:**
| Browser | Version | Platform | Priority |
|---------|---------|----------|----------|
| Chrome | Latest | Mac/Win/Linux | P0 |
| Firefox | Latest | Mac/Win/Linux | P0 |
| Safari | Latest | Mac/iOS | P0 |
| Edge | Latest | Windows | P1 |
| Chrome | -2 versions | All | P1 |

**Test Cases:**
1. Import file upload
2. Drag-and-drop
3. Progress updates (WebSocket/polling)
4. Table rendering (large datasets)
5. Form validation
6. Modal dialogs
7. Navigation
8. Mobile responsive

**Tools:**
- BrowserStack (cross-browser testing)
- Playwright (automated testing)
- WebPageTest (performance)

#### 3.4 Performance Benchmarks (2 days)

**Benchmark Suite:**
```rust
// benches/import_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use import_service::{create_default_registry, DeduplicationEngine};

fn bench_csv_parse(c: &mut Criterion) {
    let registry = create_default_registry();
    let connector = registry.find_connector(Path::new("test.csv")).unwrap();

    c.bench_function("parse_10k_csv", |b| {
        b.iter(|| {
            connector.parse(black_box(Path::new("sample_data/contacts_10k.csv")))
        })
    });
}

fn bench_deduplication(c: &mut Criterion) {
    // Benchmark dedupe strategies
    let engine = DeduplicationEngine::new(config);

    c.bench_function("dedupe_10k_contacts", |b| {
        b.iter(|| {
            engine.find_duplicates(black_box(&rows))
        })
    });
}

criterion_group!(benches, bench_csv_parse, bench_deduplication);
criterion_main!(benches);
```

**Run Benchmarks:**
```bash
cargo bench --package import_service

# Performance regression testing
cargo bench --package import_service -- --save-baseline main
# After changes:
cargo bench --package import_service -- --baseline main
```

**Metrics to Track:**
- Import throughput (contacts/sec)
- Memory usage (peak/average)
- API response times (p50, p95, p99)
- Database query latency
- Frontend render time (FCP, LCP, TTI)

---

## 📊 Success Criteria

### Week 1-2 (UI Polish)
- [ ] Import UI connected to real API
- [ ] Drag-and-drop functional
- [ ] Job tracking dashboard live
- [ ] Import history page complete
- [ ] Mobile responsive (passes lighthouse mobile test >90)

### Week 3-4 (Testing)
- [ ] 100k import completes in <2 minutes
- [ ] Memory usage <500MB for 100k contacts
- [ ] Zero critical security issues
- [ ] Works in Chrome, Firefox, Safari (latest 2 versions)
- [ ] API p95 latency <100ms
- [ ] Search works smoothly with 100k contacts

---

## 🚧 Known Gaps & Future Work

### Not in This Sprint
- ❌ Social network connectors (Twitter, Facebook, Instagram)
- ❌ Email integration (IMAP/SMTP)
- ❌ Real-time collaboration (WebSocket updates)
- ❌ Multi-tenancy / team features
- ❌ AI/ML features (smart insights, enrichment)
- ❌ Mobile apps (React Native)

### Deferred to Beta
- Incremental imports
- Mapping template persistence
- Advanced transforms (regex, scripting)
- Plugin marketplace
- CRM integrations
- E2E encryption

---

## 📝 Testing Checklist

### Manual Testing
- [ ] Upload CSV (Google Contacts format)
- [ ] Upload vCard (Apple Contacts)
- [ ] Upload SMS XML (Android)
- [ ] Upload LinkedIn CSV
- [ ] Test all deduplication strategies
- [ ] Test all match criteria
- [ ] Cancel import mid-process
- [ ] View import history
- [ ] Rollback an import
- [ ] Download error report
- [ ] Search contacts after import
- [ ] Mobile upload workflow
- [ ] Offline behavior (service worker)

### Automated Testing
```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --package sync_service --test '*'

# E2E tests (Playwright)
cd apps/web && pnpm test:e2e

# Performance tests
cargo bench --workspace

# Security audit
cargo audit && cargo geiger
```

---

## 🔄 Deployment Plan

### Pre-Deploy Checklist
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Changelog prepared
- [ ] Database migrations tested
- [ ] Backup procedures verified
- [ ] Rollback plan documented

### Deploy Steps
```bash
# 1. Run migrations
sqlite3 data/contacts.db < crates/sync_service/migrations/009_import_audit.sql

# 2. Build release
./scripts/deployment/build_all.sh

# 3. Restart services
sudo systemctl restart sagenscontact-{sync,web,worker}

# 4. Verify health
curl http://localhost:3002/health
curl http://localhost:3001/

# 5. Monitor logs
sudo journalctl -u sagenscontact-sync -f
```

### Rollback Plan
```bash
# If issues arise
sudo systemctl stop sagenscontact-{sync,web,worker}

# Restore previous version
cd /opt/sagenscontact
sudo tar xzf backups/sagenscontact-backup-$(date -d yesterday +%Y%m%d).tar.gz

# Rollback database (if needed)
sqlite3 data/contacts.db < backups/db-backup-$(date -d yesterday +%Y%m%d).sql

# Restart
sudo systemctl start sagenscontact-{sync,web,worker}
```

---

## 📈 Metrics Dashboard

Track these KPIs weekly:

**Performance:**
- Import speed (contacts/sec)
- API latency (p95)
- Search latency (p95)
- Memory usage (peak)

**Quality:**
- Test coverage %
- Bug count (critical/high/medium)
- Security issues (open/closed)

**Usage:**
- Imports per week
- Total contacts imported
- Top connectors used
- Error rate %

---

## 🎯 Deliverables

### Week 2 Demo
- [ ] Live demo: Import 10k Google Contacts
- [ ] Show job tracking in real-time
- [ ] Display import history
- [ ] Mobile responsive demo

### Week 4 Release
- [ ] Beta-ready codebase
- [ ] Performance benchmarks documented
- [ ] Security audit report
- [ ] Browser compatibility matrix
- [ ] User testing feedback incorporated
- [ ] Release notes drafted

---

**Next Steps After Week 4:**
1. Beta release announcement
2. User feedback collection
3. Bug fix sprint (week 5-6)
4. Feature prioritization for V1.0 (from IMPROVEMENT_IDEAS.md)

**Success = SagensContact ready for public beta testing by end of Week 4!** 🚀
