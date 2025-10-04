#!/usr/bin/env python3
"""Generate large test datasets for performance testing"""

import csv
import random
import sys
from pathlib import Path

# Data pools
FIRST_NAMES = [
    "John", "Jane", "Michael", "Sarah", "David", "Emily", "Robert", "Lisa",
    "James", "Mary", "William", "Patricia", "Richard", "Jennifer", "Thomas",
    "Linda", "Charles", "Barbara", "Joseph", "Elizabeth", "Christopher", "Susan",
    "Daniel", "Jessica", "Matthew", "Karen", "Anthony", "Nancy", "Mark", "Betty",
    "Donald", "Helen", "Steven", "Sandra", "Paul", "Donna", "Andrew", "Carol",
    "Joshua", "Ruth", "Kenneth", "Sharon", "Kevin", "Michelle", "Brian", "Laura",
    "George", "Kimberly", "Timothy", "Amy"
]

LAST_NAMES = [
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson",
    "Thomas", "Taylor", "Moore", "Jackson", "Martin", "Lee", "Walker", "Hall",
    "Allen", "Young", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill",
    "Flores", "Green", "Adams", "Nelson", "Baker", "Rivera", "Campbell", "Mitchell",
    "Carter", "Roberts", "Turner", "Phillips", "Evans", "Collins"
]

COMPANIES = [
    "Tech Corp", "Innovation Labs", "Global Solutions", "Digital Services",
    "Cloud Systems", "Data Analytics Inc", "Software House", "Consulting Group",
    "Enterprise Solutions", "Startup XYZ", "Venture Partners", "Creative Agency",
    "Marketing Pro", "Finance Group", "Healthcare Systems", "Education Platform",
    "E-commerce Giants", "Media Networks", "Retail Chain", "Manufacturing Co",
    "AI Research", "Robotics Inc", "Biotech Labs", "Green Energy", "Logistics Pro"
]

TITLES = [
    "Software Engineer", "Product Manager", "Data Scientist", "UX Designer",
    "DevOps Engineer", "CEO", "CTO", "CFO", "VP Engineering", "Director of Sales",
    "Marketing Manager", "Business Analyst", "Project Manager", "Solutions Architect",
    "Senior Developer", "Lead Designer", "Account Manager", "Operations Manager",
    "HR Manager", "Customer Success", "QA Engineer", "Security Analyst"
]

DOMAINS = [
    "gmail.com", "yahoo.com", "outlook.com", "hotmail.com", "company.com",
    "test.com", "example.com", "work.io", "business.net", "email.com",
    "corporate.org", "enterprise.co", "tech.dev"
]

def generate_contact():
    """Generate a single random contact"""
    first = random.choice(FIRST_NAMES)
    last = random.choice(LAST_NAMES)
    email = f"{first.lower()}.{last.lower()}{random.randint(1, 9999)}@{random.choice(DOMAINS)}"
    phone = f"+1-555-{random.randint(1000, 9999)}"
    company = random.choice(COMPANIES)
    title = random.choice(TITLES)

    return {
        'Given Name': first,
        'Family Name': last,
        'E-mail 1 - Value': email,
        'Phone 1 - Value': phone,
        'Organization 1 - Name': company,
        'Title': title
    }

def generate_csv(filename, count, duplicate_rate=0.0):
    """Generate CSV file with contacts"""
    print(f"📝 Generating {count:,} contacts...")

    fieldnames = [
        'Given Name', 'Family Name', 'E-mail 1 - Value',
        'Phone 1 - Value', 'Organization 1 - Name', 'Title'
    ]

    duplicate_pool = []

    with open(filename, 'w', newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()

        for i in range(1, count + 1):
            # Generate duplicate if rate > 0 and pool exists
            if duplicate_pool and random.random() < duplicate_rate:
                contact = random.choice(duplicate_pool)
            else:
                contact = generate_contact()
                # Add to pool (keep max 1000)
                if len(duplicate_pool) < 1000:
                    duplicate_pool.append(contact)

            writer.writerow(contact)

            if i % 10000 == 0:
                print(f"  ✓ Generated {i:,} rows...")

    size_mb = Path(filename).stat().st_size / (1024 * 1024)
    print(f"  ✅ Complete: {filename} ({size_mb:.1f}MB)")

def main():
    script_dir = Path(__file__).parent
    output_dir = script_dir.parent.parent / "sample_data" / "load_tests"
    output_dir.mkdir(parents=True, exist_ok=True)

    mode = sys.argv[1] if len(sys.argv) > 1 else "all"

    print("📊 SagensContact Test Data Generator")
    print("=" * 50)

    if mode == "10k":
        generate_csv(output_dir / "contacts_10k.csv", 10_000)

    elif mode == "100k":
        generate_csv(output_dir / "contacts_100k.csv", 100_000)

    elif mode == "50k-dupes":
        generate_csv(
            output_dir / "contacts_50k_with_duplicates.csv",
            50_000,
            duplicate_rate=0.2
        )

    elif mode == "1m":
        print("⚠️  This may take several minutes...")
        generate_csv(output_dir / "contacts_1m.csv", 1_000_000)

    elif mode == "all":
        generate_csv(output_dir / "contacts_10k.csv", 10_000)
        generate_csv(output_dir / "contacts_100k.csv", 100_000)
        generate_csv(
            output_dir / "contacts_50k_with_duplicates.csv",
            50_000,
            duplicate_rate=0.2
        )
        print("\n✅ Test data generation complete!")
        print("\nGenerated files:")
        for f in sorted(output_dir.glob("*.csv")):
            size_mb = f.stat().st_size / (1024 * 1024)
            print(f"  {f.name}: {size_mb:.1f}MB")
        print("\nTo generate the 1M stress test file, run:")
        print(f"  {sys.argv[0]} 1m")

    else:
        print(f"Usage: {sys.argv[0]} {{10k|100k|50k-dupes|1m|all}}")
        print()
        print("  10k         Generate 10,000 contacts")
        print("  100k        Generate 100,000 contacts")
        print("  50k-dupes   Generate 50,000 contacts with 20% duplicates")
        print("  1m          Generate 1,000,000 contacts (stress test)")
        print("  all         Generate all except 1m (default)")
        sys.exit(1)

if __name__ == "__main__":
    main()
