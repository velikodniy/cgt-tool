#!/usr/bin/env python3
"""
Convert cgt-tool DSL (.cgt) to KapJI/capital-gains-calculator RAW CSV format.

RAW format (CSV): date,action,symbol,quantity,price,fees,currency
Where:
  - date: YYYY-MM-DD
  - action: BUY or SELL
  - symbol: ticker/ISIN
  - quantity: number of shares
  - price: price per share
  - fees: transaction fees
  - currency: ISO currency code

Usage: python convert-to-raw.py input.cgt > output.csv
"""

import sys
from pathlib import Path

from cgt_parse import run_parse


def convert_to_raw(input_path: Path) -> list[str]:
    """Convert a .cgt file to RAW CSV format lines."""
    transactions = run_parse(input_path)
    lines = []

    for tx in transactions:
        if tx["action"] not in ("BUY", "SELL"):
            continue

        csv_line = (
            f"{tx['date']},"
            f"{tx['action']},"
            f"{tx['ticker']},"
            f"{tx['amount']},"
            f"{tx['price']['amount']},"
            f"{tx['fees']['amount']},"
            f"{tx['price']['currency']}"
        )
        lines.append(csv_line)

    return lines


def main():
    if len(sys.argv) < 2:
        print("Usage: python convert-to-raw.py input.cgt", file=sys.stderr)
        sys.exit(1)

    input_path = Path(sys.argv[1])
    if not input_path.exists():
        print(f"Error: File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    lines = convert_to_raw(input_path)
    # Sort by (date, action) with BUY before SELL on the same day.
    # cgt-calc processes transactions sequentially and requires shares
    # to exist before selling, so BUY must come first within a day.
    lines.sort(key=lambda l: (l.split(",")[0], 0 if l.split(",")[1] == "BUY" else 1))
    for line in lines:
        print(line)


if __name__ == "__main__":
    main()
