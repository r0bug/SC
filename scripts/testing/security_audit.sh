#!/bin/bash
# Security audit for SagensContact

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test_results/security"

mkdir -p "$RESULTS_DIR"

echo "🔒 SagensContact Security Audit"
echo "================================"
echo ""

cd "$PROJECT_ROOT"

# 1. Cargo Audit - Check for known vulnerabilities
echo "1️⃣  Running cargo audit (dependency vulnerabilities)..."
if ! command -v cargo-audit &> /dev/null; then
    echo "  Installing cargo-audit..."
    cargo install cargo-audit
fi

cargo audit --deny warnings > "$RESULTS_DIR/cargo_audit.txt" 2>&1 && \
    echo "  ✅ No known vulnerabilities found" || \
    echo "  ⚠️  Vulnerabilities detected - see $RESULTS_DIR/cargo_audit.txt"
echo ""

# 2. Cargo Geiger - Check for unsafe code
echo "2️⃣  Running cargo geiger (unsafe code detection)..."
if ! command -v cargo-geiger &> /dev/null; then
    echo "  Installing cargo-geiger..."
    cargo install cargo-geiger
fi

cargo geiger --all-features > "$RESULTS_DIR/unsafe_code.txt" 2>&1 && \
    echo "  ✅ Unsafe code analysis complete" || \
    echo "  ℹ️  See $RESULTS_DIR/unsafe_code.txt"
echo ""

# 3. SQL Injection Check - Grep for potentially unsafe SQL
echo "3️⃣  Checking for SQL injection vulnerabilities..."
echo "Searching for potentially unsafe SQL patterns..." > "$RESULTS_DIR/sql_injection_check.txt"

# Check for string concatenation in SQL
grep -rn "format!" crates/ --include="*.rs" | grep -i "select\|insert\|update\|delete" >> "$RESULTS_DIR/sql_injection_check.txt" 2>&1 || true

# Check that all queries use sqlx parameterized queries
grep -rn "query!" crates/ --include="*.rs" > "$RESULTS_DIR/sqlx_queries.txt" 2>&1 || true
grep -rn "query_as!" crates/ --include="*.rs" >> "$RESULTS_DIR/sqlx_queries.txt" 2>&1 || true

if [ ! -s "$RESULTS_DIR/sql_injection_check.txt" ]; then
    echo "  ✅ No SQL injection patterns detected"
else
    echo "  ⚠️  Review SQL patterns in $RESULTS_DIR/sql_injection_check.txt"
fi
echo ""

# 4. XSS Prevention Check
echo "4️⃣  Checking for XSS vulnerabilities..."
echo "Searching for potentially unsafe HTML rendering..." > "$RESULTS_DIR/xss_check.txt"

# Check for innerHTML usage in frontend
grep -rn "innerHTML" apps/web/src/ --include="*.svelte" --include="*.ts" >> "$RESULTS_DIR/xss_check.txt" 2>&1 || true
grep -rn "@html" apps/web/src/ --include="*.svelte" >> "$RESULTS_DIR/xss_check.txt" 2>&1 || true

if [ ! -s "$RESULTS_DIR/xss_check.txt" ]; then
    echo "  ✅ No XSS patterns detected (SvelteKit auto-escapes)"
else
    echo "  ⚠️  Review HTML rendering in $RESULTS_DIR/xss_check.txt"
fi
echo ""

# 5. CORS Configuration Review
echo "5️⃣  Checking CORS configuration..."
grep -rn "cors" crates/ --include="*.rs" -A 5 > "$RESULTS_DIR/cors_config.txt" 2>&1 || true
echo "  ℹ️  CORS configuration saved to $RESULTS_DIR/cors_config.txt"
echo ""

# 6. Secrets Check
echo "6️⃣  Checking for hardcoded secrets..."
echo "Searching for potential secrets..." > "$RESULTS_DIR/secrets_check.txt"

# Common patterns
grep -rn "password.*=" crates/ apps/ --include="*.rs" --include="*.ts" --include="*.js" | \
    grep -v "password:" | grep -v "// " | grep -v "Password" >> "$RESULTS_DIR/secrets_check.txt" 2>&1 || true

grep -rn "api_key\|apiKey\|API_KEY" crates/ apps/ --include="*.rs" --include="*.ts" --include="*.js" >> "$RESULTS_DIR/secrets_check.txt" 2>&1 || true

grep -rn "secret.*=" crates/ apps/ --include="*.rs" --include="*.ts" | \
    grep -v "SECRET" | grep -v "jwt" >> "$RESULTS_DIR/secrets_check.txt" 2>&1 || true

if [ ! -s "$RESULTS_DIR/secrets_check.txt" ]; then
    echo "  ✅ No hardcoded secrets detected"
else
    echo "  ⚠️  Review potential secrets in $RESULTS_DIR/secrets_check.txt"
fi
echo ""

# 7. Dependency License Check
echo "7️⃣  Checking dependency licenses..."
cargo tree --prefix none | sort -u > "$RESULTS_DIR/dependencies.txt"
echo "  ℹ️  Dependencies list saved to $RESULTS_DIR/dependencies.txt"
echo ""

# 8. Security Headers Check (if server is running)
echo "8️⃣  Checking security headers..."
if curl -s http://localhost:3002/health > /dev/null 2>&1; then
    curl -I http://localhost:3002/health > "$RESULTS_DIR/security_headers.txt" 2>&1
    echo "  ℹ️  Security headers saved to $RESULTS_DIR/security_headers.txt"

    # Check for important headers
    if grep -q "X-Frame-Options" "$RESULTS_DIR/security_headers.txt"; then
        echo "  ✅ X-Frame-Options present"
    else
        echo "  ⚠️  X-Frame-Options missing"
    fi

    if grep -q "X-Content-Type-Options" "$RESULTS_DIR/security_headers.txt"; then
        echo "  ✅ X-Content-Type-Options present"
    else
        echo "  ⚠️  X-Content-Type-Options missing"
    fi

    if grep -q "Content-Security-Policy" "$RESULTS_DIR/security_headers.txt"; then
        echo "  ✅ Content-Security-Policy present"
    else
        echo "  ⚠️  Content-Security-Policy missing"
    fi
else
    echo "  ⚠️  Server not running on localhost:3002 - skipping header check"
fi
echo ""

# Summary
echo "📋 Security Audit Summary"
echo "========================"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""
echo "✅ Completed Checks:"
echo "  - Dependency vulnerabilities (cargo-audit)"
echo "  - Unsafe code detection (cargo-geiger)"
echo "  - SQL injection patterns"
echo "  - XSS prevention"
echo "  - CORS configuration"
echo "  - Hardcoded secrets"
echo "  - Dependency licenses"
echo "  - Security headers"
echo ""
echo "📄 Review all reports in: $RESULTS_DIR"
echo ""

# Create summary
{
    echo "SagensContact Security Audit Summary"
    echo "Date: $(date)"
    echo "======================================"
    echo ""
    echo "1. Cargo Audit:"
    tail -5 "$RESULTS_DIR/cargo_audit.txt" 2>/dev/null || echo "  See cargo_audit.txt"
    echo ""
    echo "2. Unsafe Code:"
    grep -i "unsafe" "$RESULTS_DIR/unsafe_code.txt" | head -10 || echo "  See unsafe_code.txt"
    echo ""
    echo "3. SQL Injection:"
    wc -l "$RESULTS_DIR/sql_injection_check.txt" | awk '{print "  " $1 " potential issues found"}'
    echo ""
    echo "4. XSS:"
    wc -l "$RESULTS_DIR/xss_check.txt" | awk '{print "  " $1 " patterns found"}'
    echo ""
    echo "5. Secrets:"
    wc -l "$RESULTS_DIR/secrets_check.txt" | awk '{print "  " $1 " patterns found"}'
} > "$RESULTS_DIR/SUMMARY.txt"

echo "✅ Summary saved to: $RESULTS_DIR/SUMMARY.txt"
