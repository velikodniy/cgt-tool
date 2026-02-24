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

import re
import sys
from pathlib import Path


def parse_cgt_line(line: str) -> dict | None:
    """Parse a single CGT DSL line into a transaction dict."""
    line = line.strip()

    # Skip comments and empty lines
    if not line or line.startswith("#"):
        return None

    # Parse BUY: DATE BUY TICKER AMOUNT @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]
    # Currency is 3 uppercase letters (ISO 4217) that are NOT "FEES"
    buy_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+BUY\s+(\S+)\s+(\S+)\s+@\s+(\S+)(?:\s+([A-Z]{3})(?!\S))?(?:\s+FEES\s+(\S+)(?:\s+([A-Z]{3}))?)?",
        line,
    )
    if buy_match:
        date, ticker, amount, price, currency, fees, fees_currency = buy_match.groups()
        # If currency matched but is actually part of FEES keyword, ignore it
        if currency == "FEE":
            currency = None
        return {
            "action": "BUY",
            "date": date,
            "symbol": ticker,
            "quantity": amount,
            "price": price,
            "currency": currency or "GBP",
            "fees": fees or "0",
        }

    # Parse SELL: DATE SELL TICKER AMOUNT @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]
    sell_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+SELL\s+(\S+)\s+(\S+)\s+@\s+(\S+)(?:\s+([A-Z]{3})(?!\S))?(?:\s+FEES\s+(\S+)(?:\s+([A-Z]{3}))?)?",
        line,
    )
    if sell_match:
        date, ticker, amount, price, currency, fees, fees_currency = sell_match.groups()
        if currency == "FEE":
            currency = None
        return {
            "action": "SELL",
            "date": date,
            "symbol": ticker,
            "quantity": amount,
            "price": price,
            "currency": currency or "GBP",
            "fees": fees or "0",
        }

    # Skip other transaction types (DIVIDEND, SPLIT, etc.)
    return None


def convert_to_raw(input_path: Path) -> list[str]:
    """Convert a .cgt file to RAW CSV format lines."""
    lines = []

    with open(input_path) as f:
        for line in f:
            tx = parse_cgt_line(line)
            if tx:
                # RAW CSV format: date,action,symbol,quantity,price,fees,currency
                csv_line = (
                    f"{tx['date']},"
                    f"{tx['action']},"
                    f"{tx['symbol']},"
                    f"{tx['quantity']},"
                    f"{tx['price']},"
                    f"{tx['fees']},"
                    f"{tx['currency']}"
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
