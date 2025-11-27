#!/usr/bin/env python3
"""
Reverse transaction lines in .cgt files (to convert from reverse chronological to chronological).
Preserves comments and blank lines at the beginning.
"""

import sys
from pathlib import Path

def reverse_cgt_file(filepath):
    """Reverse transaction lines in a .cgt file."""
    with open(filepath, 'r') as f:
        lines = f.readlines()

    # Separate header comments from transactions
    header_comments = []
    transactions = []

    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            if not transactions:  # Still in header
                header_comments.append(line)
            else:  # Footer comments/blanks - ignore or keep with transactions
                transactions.append(line)
        else:
            transactions.append(line)

    # Reverse the transactions
    transactions.reverse()

    # Write back
    with open(filepath, 'w') as f:
        # Write header comments
        for line in header_comments:
            f.write(line)

        # Write reversed transactions
        for line in transactions:
            f.write(line)

def main():
    test_data_dir = Path('/Users/vadim/Projects/cgt-tool/tests/data')

    if not test_data_dir.exists():
        print(f"Error: {test_data_dir} does not exist")
        sys.exit(1)

    cgt_files = list(test_data_dir.glob('*.cgt'))
    print(f"Found {len(cgt_files)} .cgt files")

    for cgt_file in cgt_files:
        print(f"Reversing {cgt_file.name}...")
        reverse_cgt_file(cgt_file)

    print("Done!")

if __name__ == '__main__':
    main()
