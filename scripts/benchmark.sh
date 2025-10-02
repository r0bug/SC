#!/bin/bash
# SagensContact Performance Benchmark Script

set -e

# Configuration
API_URL="${API_URL:-http://localhost:3002}"
CONCURRENT_USERS="${CONCURRENT_USERS:-10}"
DURATION="${DURATION:-30}"
OUTPUT_DIR="./benchmark-results"

# Create output directory
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$OUTPUT_DIR/benchmark_$TIMESTAMP.md"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper function to format numbers
format_time() {
    local time=$1
    if (( $(echo "$time < 1" | bc -l) )); then
        echo "$(echo "$time * 1000" | bc -l | xargs printf "%.0f") ms"
    else
        echo "$(echo "$time" | bc -l | xargs printf "%.2f") s"
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  SagensContact Performance Benchmark"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Target: $API_URL"
echo "Concurrent Users: $CONCURRENT_USERS"
echo "Duration: ${DURATION}s"
echo ""

# Check if required tools are installed
if ! command -v curl >/dev/null 2>&1; then
    echo -e "${RED}Error: curl is required but not installed.${NC}"
    exit 1
fi

if ! command -v bc >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: bc is not installed. Calculations may be limited.${NC}"
fi

# Start markdown report
cat > "$REPORT_FILE" <<EOF
# SagensContact Performance Benchmark Report

**Date:** $(date)
**Target URL:** $API_URL
**Concurrent Users:** $CONCURRENT_USERS
**Test Duration:** ${DURATION}s

---

## Test Results

EOF

# Benchmark function
benchmark_endpoint() {
    local name=$1
    local url=$2
    local method=${3:-GET}
    local data=${4:-}

    echo -n "Testing: $name... "

    local total_time=0
    local success_count=0
    local error_count=0
    local iterations=5

    for i in $(seq 1 $iterations); do
        if [ -n "$data" ]; then
            RESPONSE=$(curl -s -o /dev/null -w "%{http_code}\n%{time_total}" -X $method \
                -H "Content-Type: application/json" \
                -d "$data" "$url" 2>/dev/null || echo "000\n0")
        else
            RESPONSE=$(curl -s -o /dev/null -w "%{http_code}\n%{time_total}" -X $method "$url" 2>/dev/null || echo "000\n0")
        fi

        HTTP_CODE=$(echo "$RESPONSE" | head -1)
        TIME=$(echo "$RESPONSE" | tail -1)

        if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "401" ] || [ "$HTTP_CODE" = "404" ]; then
            success_count=$((success_count + 1))
            total_time=$(echo "$total_time + $TIME" | bc -l 2>/dev/null || echo "$total_time")
        else
            error_count=$((error_count + 1))
        fi

        # Brief delay between requests
        sleep 0.1
    done

    if [ "$success_count" -gt 0 ]; then
        avg_time=$(echo "$total_time / $success_count" | bc -l 2>/dev/null || echo "0")
        formatted_time=$(format_time "$avg_time" 2>/dev/null || echo "$avg_time")
        echo -e "${GREEN}✓${NC} Avg: $formatted_time (${success_count}/${iterations} successful)"

        # Add to report
        cat >> "$REPORT_FILE" <<EOF
### $name
- **Endpoint:** \`$method $url\`
- **Average Response Time:** $formatted_time
- **Success Rate:** ${success_count}/${iterations} ($(echo "scale=1; $success_count * 100 / $iterations" | bc)%)
- **Errors:** $error_count

EOF
    else
        echo -e "${RED}✗${NC} Failed (${error_count}/${iterations} errors)"

        cat >> "$REPORT_FILE" <<EOF
### $name
- **Endpoint:** \`$method $url\`
- **Status:** ${RED}FAILED${NC}
- **Errors:** $error_count

EOF
    fi
}

# Benchmark function for concurrent requests
benchmark_concurrent() {
    local name=$1
    local url=$2
    local count=$3

    echo -n "Testing concurrent: $name ($count requests)... "

    local start_time=$(date +%s.%N)

    # Run requests in parallel
    for i in $(seq 1 $count); do
        curl -s -o /dev/null "$url" &
    done

    # Wait for all background jobs
    wait

    local end_time=$(date +%s.%N)
    local total_time=$(echo "$end_time - $start_time" | bc -l)
    local formatted_time=$(format_time "$total_time")
    local rps=$(echo "scale=2; $count / $total_time" | bc -l)

    echo -e "${GREEN}✓${NC} Total: $formatted_time, RPS: $rps"

    cat >> "$REPORT_FILE" <<EOF
### $name (Concurrent Test)
- **Endpoint:** \`$url\`
- **Total Requests:** $count
- **Total Time:** $formatted_time
- **Requests per Second:** $rps

EOF
}

echo "Running Benchmarks..."
echo ""

# Health Check
echo -e "${BLUE}[Health Checks]${NC}"
benchmark_endpoint "Health Check" "$API_URL/health"
benchmark_endpoint "Worker Health" "$API_URL/health/worker"
echo ""

# API Endpoints
echo -e "${BLUE}[API Endpoints]${NC}"
benchmark_endpoint "List Contacts (limit=50)" "$API_URL/api/contacts?limit=50&offset=0"
benchmark_endpoint "List Groups" "$API_URL/api/groups"
benchmark_endpoint "List Projects" "$API_URL/api/projects"
benchmark_endpoint "List Tags" "$API_URL/api/tags"
benchmark_endpoint "Dashboard Summary" "$API_URL/api/dashboard"
echo ""

# Database-heavy operations
echo -e "${BLUE}[Search Operations]${NC}"
benchmark_endpoint "Search Contacts" "$API_URL/api/contacts/search" "POST" '{"query":"test","filters":{}}'
benchmark_endpoint "Search History" "$API_URL/api/search/history"
echo ""

# AI Operations
echo -e "${BLUE}[AI Services]${NC}"
benchmark_endpoint "Get AI Insights" "$API_URL/api/ai/insights"
echo ""

# Concurrent Load Tests
echo -e "${BLUE}[Concurrent Load Tests]${NC}"
benchmark_concurrent "Health Check" "$API_URL/health" 50
benchmark_concurrent "List Contacts" "$API_URL/api/contacts?limit=10" 20
echo ""

# Add system info to report
cat >> "$REPORT_FILE" <<EOF

---

## System Information

**OS:** $(uname -s)
**Kernel:** $(uname -r)
**Architecture:** $(uname -m)

## Performance Summary

EOF

# Calculate overall stats
cat >> "$REPORT_FILE" <<EOF
The benchmark tests have been completed. Review the individual endpoint results above for detailed performance metrics.

### Recommendations

1. **Response Times < 100ms:** Excellent
2. **Response Times 100-500ms:** Good
3. **Response Times 500ms-1s:** Acceptable
4. **Response Times > 1s:** Needs optimization

### Next Steps

- If any endpoints show poor performance, consider:
  - Adding database indexes
  - Implementing caching strategies
  - Optimizing database queries
  - Scaling horizontally with more workers

---

*Generated by SagensContact Benchmark Tool*
EOF

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Benchmark Complete!${NC}"
echo ""
echo "Results saved to: $REPORT_FILE"
echo ""
echo "To view the report:"
echo "  cat $REPORT_FILE"
echo ""
echo "To run more intensive tests:"
echo "  CONCURRENT_USERS=50 DURATION=60 $0"
echo ""
