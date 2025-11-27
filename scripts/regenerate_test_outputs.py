#!/usr/bin/env python3
"""
Regenerate test output JSON files by running the CLI on each .cgt file.
"""

import os
import sys
import subprocess
import json
from pathlib import Path

def run_cli_report(cgt_file, year):
    """Run the cgt-cli report command and return the JSON output."""
    cli_path = Path('/Users/vadim/Projects/cgt-tool/target/release/cgt-cli')

    if not cli_path.exists():
        print(f"Error: CLI not found at {cli_path}")
        sys.exit(1)

    result = subprocess.run(
        [str(cli_path), 'report', str(cgt_file), '--year', str(year)],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error running CLI for {cgt_file}")
        print(f"STDERR: {result.stderr}")
        return None

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON output for {cgt_file}: {e}")
        print(f"Output: {result.stdout}")
        return None

def determine_tax_year(cgt_file):
    """Determine the tax year from the .cgt file by looking at transaction dates."""
    with open(cgt_file, 'r') as f:
        lines = f.readlines()

    years = set()
    for line in lines:
        line = line.strip()
        if line and not line.startswith('#'):
            parts = line.split()
            if len(parts) > 0:
                date_str = parts[0]
                try:
                    year = int(date_str.split('-')[0])
                    years.add(year)
                except (ValueError, IndexError):
                    pass

    if not years:
        return 2018  # Default

    # Use the earliest year for tax year calculation
    # Tax year starts in April, so a transaction in Jan-March belongs to previous tax year
    min_year = min(years)
    max_year = max(years)

    # For simplicity, use the year before the minimum transaction year
    # This ensures we capture all transactions
    return min_year - 1 if min_year > 2000 else max_year

def main():
    test_data_dir = Path('/Users/vadim/Projects/cgt-tool/tests/data')

    if not test_data_dir.exists():
        print(f"Error: Test data directory {test_data_dir} does not exist")
        sys.exit(1)

    cgt_files = list(test_data_dir.glob('*.cgt'))
    print(f"Found {len(cgt_files)} .cgt files")

    for cgt_file in cgt_files:
        # Skip unsorted_transactions as it's for testing sorting
        if cgt_file.stem == 'unsorted_transactions':
            print(f"Skipping {cgt_file.name} (test file)")
            continue

        # Determine tax year
        tax_year = determine_tax_year(cgt_file)

        # Check if there's an existing JSON to extract the tax year from
        json_file = cgt_file.with_suffix('.json')
        if json_file.exists():
            try:
                with open(json_file, 'r') as f:
                    existing_data = json.load(f)
                    if 'tax_year' in existing_data:
                        tax_year = existing_data['tax_year']
            except:
                pass

        print(f"Processing {cgt_file.name} for tax year {tax_year}...")

        report = run_cli_report(cgt_file, tax_year)
        if report:
            output_file = cgt_file.with_suffix('.json')
            with open(output_file, 'w') as f:
                json.dump(report, f, indent=2)
            print(f"  -> Generated {output_file.name}")
        else:
            print(f"  -> FAILED to generate output for {cgt_file.name}")

    print("Done!")

if __name__ == '__main__':
    main()
