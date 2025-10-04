#!/bin/bash
# Generate large test datasets for performance testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/sample_data/load_tests"

mkdir -p "$OUTPUT_DIR"

echo "📊 SagensContact Test Data Generator"
echo "======================================"

# Function to generate random names
generate_name() {
    local first_names=("John" "Jane" "Michael" "Sarah" "David" "Emily" "Robert" "Lisa" "James" "Mary" "William" "Patricia" "Richard" "Jennifer" "Thomas" "Linda" "Charles" "Barbara" "Joseph" "Elizabeth" "Christopher" "Susan" "Daniel" "Jessica" "Matthew" "Karen" "Anthony" "Nancy" "Mark" "Betty" "Donald" "Helen" "Steven" "Sandra" "Paul" "Donna" "Andrew" "Carol" "Joshua" "Ruth" "Kenneth" "Sharon" "Kevin" "Michelle" "Brian" "Laura" "George" "Sarah" "Timothy" "Kimberly")

    local last_names=("Smith" "Johnson" "Williams" "Brown" "Jones" "Garcia" "Miller" "Davis" "Rodriguez" "Martinez" "Hernandez" "Lopez" "Gonzalez" "Wilson" "Anderson" "Thomas" "Taylor" "Moore" "Jackson" "Martin" "Lee" "Walker" "Hall" "Allen" "Young" "King" "Wright" "Scott" "Torres" "Nguyen" "Hill" "Flores" "Green" "Adams" "Nelson" "Baker" "Hall" "Rivera" "Campbell" "Mitchell" "Carter" "Roberts")

    local first_idx=$((RANDOM % ${#first_names[@]}))
    local last_idx=$((RANDOM % ${#last_names[@]}))

    echo "${first_names[$first_idx]},${last_names[$last_idx]}"
}

# Function to generate random email
generate_email() {
    local name="$1"
    local domains=("gmail.com" "yahoo.com" "outlook.com" "hotmail.com" "company.com" "test.com" "example.com" "work.io" "business.net" "email.com")
    local domain_idx=$((RANDOM % ${#domains[@]}))

    # Clean name and convert to lowercase
    local clean_name=$(echo "$name" | tr ',' '.' | tr '[:upper:]' '[:lower:]')

    echo "${clean_name}${RANDOM}@${domains[$domain_idx]}"
}

# Function to generate random phone
generate_phone() {
    echo "+1-555-$(printf '%04d' $((RANDOM % 10000)))"
}

# Function to generate random company
generate_company() {
    local companies=("Tech Corp" "Innovation Labs" "Global Solutions" "Digital Services" "Cloud Systems" "Data Analytics Inc" "Software House" "Consulting Group" "Enterprise Solutions" "Startup XYZ" "Venture Partners" "Creative Agency" "Marketing Pro" "Finance Group" "Healthcare Systems" "Education Platform" "E-commerce Giants" "Media Networks" "Retail Chain" "Manufacturing Co")

    local idx=$((RANDOM % ${#companies[@]}))
    echo "${companies[$idx]}"
}

# Function to generate random title
generate_title() {
    local titles=("Software Engineer" "Product Manager" "Data Scientist" "UX Designer" "DevOps Engineer" "CEO" "CTO" "CFO" "VP Engineering" "Director of Sales" "Marketing Manager" "Business Analyst" "Project Manager" "Solutions Architect" "Senior Developer" "Lead Designer" "Account Manager" "Operations Manager" "HR Manager" "Customer Success")

    local idx=$((RANDOM % ${#titles[@]}))
    echo "${titles[$idx]}"
}

# Generate 10k contacts CSV
generate_10k() {
    local output="$OUTPUT_DIR/contacts_10k.csv"
    echo "📝 Generating 10,000 contacts..."

    echo "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name,Title" > "$output"

    for i in {1..10000}; do
        name=$(generate_name)
        IFS=',' read -r first last <<< "$name"
        email=$(generate_email "$name")
        phone=$(generate_phone)
        company=$(generate_company)
        title=$(generate_title)

        echo "$first,$last,$email,$phone,$company,$title" >> "$output"

        if [ $((i % 1000)) -eq 0 ]; then
            echo "  ✓ Generated $i rows..."
        fi
    done

    echo "  ✅ Complete: $output ($(du -h "$output" | cut -f1))"
}

# Generate 100k contacts CSV
generate_100k() {
    local output="$OUTPUT_DIR/contacts_100k.csv"
    echo "📝 Generating 100,000 contacts..."

    echo "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name,Title" > "$output"

    for i in {1..100000}; do
        name=$(generate_name)
        IFS=',' read -r first last <<< "$name"
        email=$(generate_email "$name")
        phone=$(generate_phone)
        company=$(generate_company)
        title=$(generate_title)

        echo "$first,$last,$email,$phone,$company,$title" >> "$output"

        if [ $((i % 10000)) -eq 0 ]; then
            echo "  ✓ Generated $i rows..."
        fi
    done

    echo "  ✅ Complete: $output ($(du -h "$output" | cut -f1))"
}

# Generate 50k contacts with duplicates (for dedupe testing)
generate_50k_duplicates() {
    local output="$OUTPUT_DIR/contacts_50k_with_duplicates.csv"
    echo "📝 Generating 50,000 contacts (with 20% duplicates)..."

    echo "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name,Title" > "$output"

    # Store some contacts for duplication
    declare -a duplicate_pool

    for i in {1..50000}; do
        # 20% chance to use a duplicate
        if [ ${#duplicate_pool[@]} -gt 0 ] && [ $((RANDOM % 100)) -lt 20 ]; then
            # Pick a random contact from pool
            local dup_idx=$((RANDOM % ${#duplicate_pool[@]}))
            echo "${duplicate_pool[$dup_idx]}" >> "$output"
        else
            name=$(generate_name)
            IFS=',' read -r first last <<< "$name"
            email=$(generate_email "$name")
            phone=$(generate_phone)
            company=$(generate_company)
            title=$(generate_title)

            local contact="$first,$last,$email,$phone,$company,$title"
            echo "$contact" >> "$output"

            # Add to duplicate pool (keep max 1000)
            if [ ${#duplicate_pool[@]} -lt 1000 ]; then
                duplicate_pool+=("$contact")
            fi
        fi

        if [ $((i % 5000)) -eq 0 ]; then
            echo "  ✓ Generated $i rows..."
        fi
    done

    echo "  ✅ Complete: $output ($(du -h "$output" | cut -f1))"
}

# Generate stress test data (1M rows)
generate_1m() {
    local output="$OUTPUT_DIR/contacts_1m.csv"
    echo "📝 Generating 1,000,000 contacts (stress test)..."
    echo "⚠️  This may take several minutes..."

    echo "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name,Title" > "$output"

    for i in {1..1000000}; do
        name=$(generate_name)
        IFS=',' read -r first last <<< "$name"
        email=$(generate_email "$name")
        phone=$(generate_phone)
        company=$(generate_company)
        title=$(generate_title)

        echo "$first,$last,$email,$phone,$company,$title" >> "$output"

        if [ $((i % 50000)) -eq 0 ]; then
            echo "  ✓ Generated $i rows..."
        fi
    done

    echo "  ✅ Complete: $output ($(du -h "$output" | cut -f1))"
}

# Main menu
case "${1:-all}" in
    10k)
        generate_10k
        ;;
    100k)
        generate_100k
        ;;
    50k-dupes)
        generate_50k_duplicates
        ;;
    1m)
        generate_1m
        ;;
    all)
        generate_10k
        generate_100k
        generate_50k_duplicates
        echo ""
        echo "✅ Test data generation complete!"
        echo ""
        echo "Generated files:"
        ls -lh "$OUTPUT_DIR"
        echo ""
        echo "To generate the 1M stress test file, run:"
        echo "  $0 1m"
        ;;
    *)
        echo "Usage: $0 {10k|100k|50k-dupes|1m|all}"
        echo ""
        echo "  10k         Generate 10,000 contacts"
        echo "  100k        Generate 100,000 contacts"
        echo "  50k-dupes   Generate 50,000 contacts with 20% duplicates"
        echo "  1m          Generate 1,000,000 contacts (stress test)"
        echo "  all         Generate all except 1m (default)"
        exit 1
        ;;
esac
