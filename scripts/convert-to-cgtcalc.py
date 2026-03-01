#!/usr/bin/env python3
"""
Convert cgt-tool DSL (.cgt) to mattjgalloway/cgtcalc format.

cgtcalc format: TYPE DD/MM/YYYY TICKER AMOUNT PRICE EXPENSES
Where TYPE is BUY, SELL, DIVIDEND, CAPRETURN, SPLIT, or UNSPLIT.

Note: cgtcalc uses £ for all values, no currency support.

Usage: python convert-to-cgtcalc.py input.cgt > output.txt
"""

import sys
from datetime import datetime
from pathlib import Path

from cgt_parse import run_parse


def convert_date(date_str: str) -> str:
    """Convert YYYY-MM-DD to DD/MM/YYYY."""
    dt = datetime.strptime(date_str, "%Y-%m-%d")
    return dt.strftime("%d/%m/%Y")


def convert_to_cgtcalc(input_path: Path) -> list[str]:
    """Convert a .cgt file to cgtcalc format lines."""
    transactions = run_parse(input_path)
    lines = []

    for tx in transactions:
        action = tx["action"]
        date = convert_date(tx["date"])
        ticker = tx["ticker"]

        if action in ("BUY", "SELL"):
            line = f"{action} {date} {ticker} {tx['amount']} {tx['price']['amount']} {tx['fees']['amount']}"
        elif action == "ACCUMULATION":
            # Maps to cgtcalc DIVIDEND (accumulation fund cost basis adjustment)
            line = f"DIVIDEND {date} {ticker} {tx['amount']} {tx['total_value']['amount']}"
        elif action == "CAPRETURN":
            line = f"CAPRETURN {date} {ticker} {tx['amount']} {tx['total_value']['amount']}"
        elif action in ("SPLIT", "UNSPLIT"):
            line = f"{action} {date} {ticker} {tx['ratio']}"
        elif action == "DIVIDEND":
            # Cash dividends have no CGT impact; cgtcalc has no equivalent
            continue
        else:
            continue

        lines.append(line)

    return lines


def main():
    if len(sys.argv) < 2:
        print("Usage: python convert-to-cgtcalc.py input.cgt", file=sys.stderr)
        sys.exit(1)

    input_path = Path(sys.argv[1])
    if not input_path.exists():
        print(f"Error: File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    lines = convert_to_cgtcalc(input_path)
    for line in lines:
        print(line)


if __name__ == "__main__":
    main()
