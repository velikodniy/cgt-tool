#!/usr/bin/env python3
"""
Convert original cgtcalc input files to new .cgt format with enhanced DSL.
Preserves the original line order (stable sort - no reordering).
"""

import os
import sys
from pathlib import Path
from datetime import datetime
import re

def convert_date(date_str):
    """Convert DD/MM/YYYY to YYYY-MM-DD."""
    dt = datetime.strptime(date_str, '%d/%m/%Y')
    return dt.strftime('%Y-%m-%d')

def convert_line(line):
    """Convert a single transaction line to new DSL format."""
    line = line.strip()
    if not line or line.startswith('#'):
        return line

    parts = line.split()
    if len(parts) < 3:
        return line

    action = parts[0]
    date_original = parts[1]
    date_new = convert_date(date_original)
    ticker = parts[2]

    if action in ['BUY', 'SELL']:
        # Format: BUY/SELL DD/MM/YYYY TICKER AMOUNT PRICE EXPENSES
        if len(parts) >= 6:
            amount = parts[3]
            price = parts[4]
            expenses = parts[5] if len(parts) > 5 else '0'
            return f"{date_new} {action} {ticker} {amount} @ {price} EXPENSES {expenses}"
        elif len(parts) >= 5:
            amount = parts[3]
            price = parts[4]
            return f"{date_new} {action} {ticker} {amount} @ {price}"

    elif action == 'DIVIDEND':
        # Format: DIVIDEND DD/MM/YYYY TICKER AMOUNT TAX_AMOUNT
        # New: YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_AMOUNT
        if len(parts) >= 5:
            amount = parts[3]
            tax_amount = parts[4]
            return f"{date_new} DIVIDEND {ticker} {amount} TAX {tax_amount}"
        elif len(parts) >= 4:
            amount = parts[3]
            return f"{date_new} DIVIDEND {ticker} {amount} TAX 0"

    elif action == 'CAPRETURN':
        # Format: CAPRETURN DD/MM/YYYY TICKER AMOUNT EXPENSE_AMOUNT
        # New: YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSE_AMOUNT
        if len(parts) >= 5:
            amount = parts[3]
            expense_amount = parts[4]
            return f"{date_new} CAPRETURN {ticker} {amount} EXPENSES {expense_amount}"
        elif len(parts) >= 4:
            amount = parts[3]
            return f"{date_new} CAPRETURN {ticker} {amount} EXPENSES 0"

    elif action in ['SPLIT', 'UNSPLIT']:
        # Format: SPLIT/UNSPLIT DD/MM/YYYY TICKER RATIO_VALUE
        # New: YYYY-MM-DD SPLIT/UNSPLIT TICKER RATIO RATIO_VALUE
        if len(parts) >= 4:
            ratio_value = parts[3]
            return f"{date_new} {action} {ticker} RATIO {ratio_value}"

    return line

def convert_file(input_path, output_path):
    """Convert a cgtcalc input file to new .cgt format, preserving line order."""
    with open(input_path, 'r') as f:
        lines = f.readlines()

    # Convert lines in place, preserving original order
    with open(output_path, 'w') as f:
        for line in lines:
            converted = convert_line(line)
            f.write(converted + '\n')

def main():
    source_dir = Path('/tmp/cgtcalc-original/Tests/CGTCalcCoreTests/TestData/Examples/Inputs')
    target_dir = Path('/Users/vadim/Projects/cgt-tool/tests/data')

    if not source_dir.exists():
        print(f"Error: Source directory {source_dir} does not exist")
        sys.exit(1)

    if not target_dir.exists():
        print(f"Error: Target directory {target_dir} does not exist")
        sys.exit(1)

    # Process all .txt files in the source directory
    input_files = list(source_dir.glob('*.txt'))
    print(f"Found {len(input_files)} input files to convert")

    for input_file in input_files:
        output_file = target_dir / f"{input_file.stem}.cgt"
        print(f"Converting {input_file.name} -> {output_file.name}")
        convert_file(input_file, output_file)

    print("Done!")

if __name__ == '__main__':
    main()
