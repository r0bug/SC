# Performance Benchmarking Guide

This document describes the performance benchmarking tools and practices for SagensContact.

## Overview

SagensContact includes two types of benchmarking tools:

1. **Shell Script Benchmarks** (`scripts/benchmark.sh`) - HTTP endpoint performance testing
2. **Rust Criterion Benchmarks** - Micro-benchmarks for critical code paths

---

## 1. Shell Script Benchmarks

### Location
`/scripts/benchmark.sh`

### Purpose
Load testing and performance profiling of the HTTP API endpoints.

### Requirements
- `curl` - HTTP client
- `bc` - Command-line calculator (optional, for calculations)
- Running SagensContact sync service

### Usage

#### Basic Usage
```bash
./scripts/benchmark.sh
```

#### Custom Configuration
```bash
# Test with 50 concurrent users for 60 seconds
CONCURRENT_USERS=50 DURATION=60 ./scripts/benchmark.sh

# Test against a different API endpoint
API_URL=https://api.example.com ./scripts/benchmark.sh

# Combine options
API_URL=http://localhost:3002 CONCURRENT_USERS=25 DURATION=45 ./scripts/benchmark.sh
```

### Configuration Options

| Variable | Default | Description |
|----------|---------|-------------|
| `API_URL` | `http://localhost:3002` | Target API base URL |
| `CONCURRENT_USERS` | `10` | Number of concurrent users to simulate |
| `DURATION` | `30` | Test duration in seconds |
| `OUTPUT_DIR` | `./benchmark-results` | Directory for benchmark reports |

### What It Tests

#### 1. Health Checks
- Service health endpoint
- Worker health status
- Average response time
- Success rate

#### 2. API Endpoints
- **List Contacts** (with pagination)
- **List Groups**
- **List Projects**
- **List Tags**
- **Dashboard Summary**

#### 3. Search Operations
- Contact search (POST with query)
- Search history retrieval

#### 4. AI Services
- AI insights generation
- Response caching performance

#### 5. Concurrent Load Tests
- 50 simultaneous health checks
- 20 simultaneous contact list requests

### Output Format

The script generates a detailed Markdown report with:

```markdown
# SagensContact Performance Benchmark Report

**Date:** 2025-10-01 15:30:00
**Target URL:** http://localhost:3002
**Concurrent Users:** 10
**Test Duration:** 30s

---

## Test Results

### Health Check
- **Endpoint:** `GET /health`
- **Average Response Time:** 15 ms
- **Success Rate:** 5/5 (100%)
- **Errors:** 0

### List Contacts (limit=50)
- **Endpoint:** `GET /api/contacts?limit=50&offset=0`
- **Average Response Time:** 125 ms
- **Success Rate:** 5/5 (100%)
- **Errors:** 0

...
```

### Interpreting Results

**Response Time Guidelines:**
- ✅ **< 100ms**: Excellent
- ✅ **100-500ms**: Good
- ⚠️ **500ms-1s**: Acceptable
- ❌ **> 1s**: Needs optimization

**Success Rate:**
- ✅ **100%**: All requests succeeded
- ⚠️ **80-99%**: Some failures, investigate logs
- ❌ **< 80%**: Significant issues, immediate attention needed

### Troubleshooting

**Problem:** "Connection refused"
- **Solution:** Ensure the sync service is running on the specified port

**Problem:** "401 Unauthorized" for all endpoints
- **Solution:** This is expected for authenticated endpoints. The test focuses on response time, not authentication.

**Problem:** Very slow response times
- **Possible causes:**
  - Database not optimized (missing indexes)
  - Too many records without pagination
  - Network latency
  - Server resource constraints

---

## 2. Rust Criterion Benchmarks

### Location
- **API Benchmarks:** `/crates/sync_service/benches/api_bench.rs`
- **Database Benchmarks:** `/crates/local_store/benches/db_bench.rs`

### Purpose
Micro-benchmarks to measure performance of critical code paths and identify optimization opportunities.

### Requirements
- Rust toolchain
- Criterion dependency (already configured)

### Usage

#### Run All Benchmarks
```bash
# From project root
cargo bench

# Run specific benchmark suite
cargo bench --bench api_bench
cargo bench --bench db_bench
```

#### View HTML Reports
```bash
# Reports are generated in target/criterion/
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

### API Benchmarks (`api_bench.rs`)

#### What It Tests

1. **Contact Serialization/Deserialization**
   - JSON encoding performance
   - JSON decoding performance
   - Memory efficiency

2. **Bulk Operations**
   - Serializing arrays of 10, 100, 1000 contacts
   - Deserializing large JSON payloads
   - Scaling characteristics

3. **UUID Operations**
   - UUID v4 generation speed
   - UUID to string conversion
   - String to UUID parsing

4. **Tag Filtering**
   - Single tag lookup in 1000 contacts
   - Multiple tag intersection
   - Filter performance scaling

5. **Data Cloning**
   - Contact clone performance
   - Memory allocation patterns

#### Example Output
```
serialize_contact         time:   [2.5234 µs 2.5456 µs 2.5701 µs]
deserialize_contact       time:   [3.8921 µs 3.9245 µs 3.9589 µs]
uuid_generation          time:   [89.234 ns 90.125 ns 91.234 ns]
filter_contacts_by_tag   time:   [45.234 µs 45.987 µs 46.823 µs]
```

### Database Benchmarks (`db_bench.rs`)

#### What It Tests

1. **Query Building**
   - Dynamic SQL query construction
   - Performance with 0, 1, 5, 10 filters
   - String concatenation efficiency

2. **Pagination Calculations**
   - Offset/limit calculation speed
   - Performance with 100, 1k, 10k items
   - Edge case handling

3. **UUID Validation**
   - Valid UUID parsing
   - Invalid UUID rejection
   - Error handling overhead

4. **Tag Operations**
   - Finding contacts with all required tags
   - Set intersection performance
   - Scaling with contact count

5. **Search Operations**
   - Case-insensitive substring matching
   - Full-text search simulation
   - Performance with 100, 500, 1000 contacts

#### Example Output
```
query_building/build_query/0   time:   [125.34 ns 126.89 ns 128.67 ns]
query_building/build_query/5   time:   [456.23 ns 461.45 ns 467.89 ns]
pagination/calculate/1000      time:   [15.234 ns 15.456 ns 15.789 ns]
search_operations/search_by_name/1000  time:   [78.234 µs 79.456 µs 80.912 µs]
```

### Understanding Criterion Output

**Time Metrics:**
- **ns (nanoseconds)**: 1/1,000,000,000 second
- **µs (microseconds)**: 1/1,000,000 second
- **ms (milliseconds)**: 1/1,000 second

**Statistical Measures:**
- **Lower bound**: 2.5th percentile
- **Estimate**: Mean (average)
- **Upper bound**: 97.5th percentile

**Change Detection:**
```
change: [-5.1234% -3.2345% -1.2345%] (p = 0.00 < 0.05)
         ↑ Performance improved by 3.2% (statistically significant)

change: [+2.1234% +4.5678% +6.7890%] (p = 0.00 < 0.05)
         ↑ Performance degraded by 4.6% (statistically significant)
```

### Baseline Comparisons

To track performance over time:

```bash
# Save current results as baseline
cargo bench --bench api_bench -- --save-baseline main

# Compare against baseline after changes
cargo bench --bench api_bench -- --baseline main
```

---

## 3. Performance Optimization Workflow

### Step 1: Identify Bottlenecks

1. Run shell benchmarks to find slow endpoints
2. Run Criterion benchmarks to find slow operations
3. Review server logs for errors/warnings

### Step 2: Profile

Use Rust profiling tools:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench api_bench
```

### Step 3: Optimize

Common optimization strategies:

1. **Database Queries**
   - Add indexes
   - Use prepared statements
   - Implement query result caching
   - Optimize joins

2. **Serialization**
   - Use binary formats (MessagePack, Protobuf) instead of JSON
   - Implement lazy deserialization
   - Reduce payload sizes

3. **Memory Allocation**
   - Reuse buffers
   - Use arena allocators
   - Reduce cloning

4. **Concurrency**
   - Add connection pooling
   - Use async/await properly
   - Implement request batching

### Step 4: Verify

1. Re-run benchmarks
2. Compare against baseline
3. Ensure improvements are statistically significant
4. Check for regressions in other areas

---

## 4. Continuous Performance Monitoring

### CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/benchmark.yml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1

      - name: Run benchmarks
        run: cargo bench --no-fail-fast

      - name: Archive benchmark results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/criterion/
```

### Performance Regression Detection

Use Criterion's baseline comparison:

```bash
# In CI, compare against main branch
git checkout main
cargo bench -- --save-baseline main
git checkout feature-branch
cargo bench -- --baseline main
```

If performance degradation > 5%, fail the build.

---

## 5. Performance Targets

### API Endpoints (P95)

| Endpoint | Target | Current |
|----------|--------|---------|
| Health Check | < 50ms | TBD |
| List Contacts (50) | < 200ms | TBD |
| Search Contacts | < 500ms | TBD |
| Create Contact | < 300ms | TBD |
| AI Insights | < 2s | TBD |

### Throughput

| Operation | Target | Current |
|-----------|--------|---------|
| Contacts/second | > 100 | TBD |
| Concurrent Users | > 50 | TBD |
| Database Queries/s | > 1000 | TBD |

### Resource Usage

| Metric | Target | Current |
|--------|--------|---------|
| Memory per Request | < 5MB | TBD |
| CPU per Request | < 10ms | TBD |
| Database Connections | < 20 | TBD |

---

## 6. Common Performance Issues

### Issue: Slow Database Queries

**Symptoms:**
- High response times for list/search endpoints
- Database CPU usage spikes

**Solutions:**
- Add indexes on frequently queried columns
- Use `EXPLAIN ANALYZE` to understand query plans
- Implement query result caching
- Use connection pooling

**Example:**
```sql
-- Add index on commonly searched fields
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_tags ON contacts USING GIN(tags);
```

### Issue: Memory Growth

**Symptoms:**
- Increasing memory usage over time
- Out of memory errors

**Solutions:**
- Review for memory leaks
- Implement proper cleanup in async tasks
- Use streaming for large responses
- Set connection pool limits

### Issue: Slow JSON Serialization

**Symptoms:**
- High CPU usage during serialization
- Slow response times for large payloads

**Solutions:**
- Use `serde` with optimizations
- Implement pagination aggressively
- Consider binary formats
- Cache serialized responses

### Issue: AI Request Latency

**Symptoms:**
- Timeout errors for AI endpoints
- High P99 latencies

**Solutions:**
- Implement request queuing
- Use prompt caching
- Add response streaming
- Set appropriate timeouts

---

## 7. Best Practices

### DO ✅
- Run benchmarks regularly (weekly minimum)
- Track performance metrics over time
- Set realistic performance targets
- Test with production-like data volumes
- Profile before optimizing
- Document performance improvements

### DON'T ❌
- Optimize without measuring first
- Ignore micro-benchmarks
- Test with only small datasets
- Skip baseline comparisons
- Assume optimization worked without verification
- Sacrifice code clarity for marginal gains

---

## 8. Resources

### Tools
- **Criterion.rs**: https://github.com/bheisler/criterion.rs
- **Flamegraph**: https://github.com/flamegraph-rs/flamegraph
- **cargo-bench**: Built into Cargo

### Documentation
- Criterion User Guide: https://bheisler.github.io/criterion.rs/book/
- Rust Performance Book: https://nnethercote.github.io/perf-book/

### Monitoring
- Prometheus metrics (already integrated)
- Grafana dashboards (TODO)
- Application logs (tracing)

---

## Summary

SagensContact provides comprehensive performance benchmarking tools:

- ✅ **Shell script** for end-to-end HTTP API testing
- ✅ **Criterion benchmarks** for micro-optimizations
- ✅ **Detailed reporting** with actionable metrics
- ✅ **Baseline comparison** for regression detection
- ✅ **CI/CD ready** for continuous monitoring

Run benchmarks regularly, track trends, and optimize based on data!
