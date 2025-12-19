#!/usr/bin/env python3
"""
Convert cgt-tool DSL (.cgt) to mattjgalloway/cgtcalc format.

cgtcalc format: TYPE DD/MM/YYYY TICKER AMOUNT PRICE EXPENSES
Where TYPE is BUY, SELL, DIVIDEND, CAPRETURN, SPLIT, or UNSPLIT.

Note: cgtcalc uses £ for all values, no currency support.

Usage: python convert-to-cgtcalc.py input.cgt > output.txt
"""

import re
import sys
from datetime import datetime
from pathlib import Path


def convert_date(date_str: str) -> str:
    """Convert YYYY-MM-DD to DD/MM/YYYY."""
    dt = datetime.strptime(date_str, "%Y-%m-%d")
    return dt.strftime("%d/%m/%Y")


def parse_cgt_line(line: str) -> dict | None:
    """Parse a single CGT DSL line into a transaction dict."""
    line = line.strip()

    # Skip comments and empty lines
    if not line or line.startswith("#"):
        return None

    # Parse BUY: DATE BUY TICKER AMOUNT @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]
    buy_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+BUY\s+(\S+)\s+(\S+)\s+@\s+(\S+)(?:\s+(\w{3}))?(?:\s+FEES\s+(\S+)(?:\s+(\w{3}))?)?",
        line,
    )
    if buy_match:
        date, ticker, amount, price, currency, fees, fees_currency = buy_match.groups()
        return {
            "type": "BUY",
            "date": date,
            "ticker": ticker,
            "amount": amount,
            "price": price,
            "fees": fees or "0",
        }

    # Parse SELL: DATE SELL TICKER AMOUNT @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]
    sell_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+SELL\s+(\S+)\s+(\S+)\s+@\s+(\S+)(?:\s+(\w{3}))?(?:\s+FEES\s+(\S+)(?:\s+(\w{3}))?)?",
        line,
    )
    if sell_match:
        date, ticker, amount, price, currency, fees, fees_currency = sell_match.groups()
        return {
            "type": "SELL",
            "date": date,
            "ticker": ticker,
            "amount": amount,
            "price": price,
            "fees": fees or "0",
        }

    # Parse DIVIDEND: DATE DIVIDEND TICKER AMOUNT TOTAL VALUE [CURRENCY] [TAX AMOUNT [CURRENCY]]
    div_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+DIVIDEND\s+(\S+)\s+(\S+)\s+TOTAL\s+(\S+)(?:\s+(\w{3}))?(?:\s+TAX\s+(\S+))?",
        line,
    )
    if div_match:
        date, ticker, amount, value, currency, tax = div_match.groups()
        return {
            "type": "DIVIDEND",
            "date": date,
            "ticker": ticker,
            "amount": amount,
            "value": value,
        }

    # Parse CAPRETURN: DATE CAPRETURN TICKER AMOUNT TOTAL VALUE [CURRENCY] [FEES AMOUNT]
    cap_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+CAPRETURN\s+(\S+)\s+(\S+)\s+TOTAL\s+(\S+)",
        line,
    )
    if cap_match:
        date, ticker, amount, value = cap_match.groups()
        return {
            "type": "CAPRETURN",
            "date": date,
            "ticker": ticker,
            "amount": amount,
            "value": value,
        }

    # Parse SPLIT: DATE SPLIT TICKER RATIO N
    split_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+SPLIT\s+(\S+)\s+RATIO\s+(\S+)",
        line,
    )
    if split_match:
        date, ticker, ratio = split_match.groups()
        return {
            "type": "SPLIT",
            "date": date,
            "ticker": ticker,
            "ratio": ratio,
        }

    # Parse UNSPLIT: DATE UNSPLIT TICKER RATIO N
    unsplit_match = re.match(
        r"(\d{4}-\d{2}-\d{2})\s+UNSPLIT\s+(\S+)\s+RATIO\s+(\S+)",
        line,
    )
    if unsplit_match:
        date, ticker, ratio = unsplit_match.groups()
        return {
            "type": "UNSPLIT",
            "date": date,
            "ticker": ticker,
            "ratio": ratio,
        }

    return None


def convert_to_cgtcalc(input_path: Path) -> list[str]:
    """Convert a .cgt file to cgtcalc format lines."""
    lines = []

    with open(input_path) as f:
        for line in f:
            tx = parse_cgt_line(line)
            if tx:
                tx_type = tx["type"]
                date = convert_date(tx["date"])
                ticker = tx["ticker"]

                if tx_type in ("BUY", "SELL"):
                    # BUY/SELL DD/MM/YYYY TICKER AMOUNT PRICE EXPENSES
                    cgt_line = f"{tx_type} {date} {ticker} {tx['amount']} {tx['price']} {tx['fees']}"
                elif tx_type == "DIVIDEND":
                    # DIVIDEND DD/MM/YYYY TICKER AMOUNT VALUE
                    cgt_line = f"DIVIDEND {date} {ticker} {tx['amount']} {tx['value']}"
                elif tx_type == "CAPRETURN":
                    # CAPRETURN DD/MM/YYYY TICKER AMOUNT VALUE
                    cgt_line = f"CAPRETURN {date} {ticker} {tx['amount']} {tx['value']}"
                elif tx_type == "SPLIT":
                    # SPLIT DD/MM/YYYY TICKER MULTIPLIER
                    cgt_line = f"SPLIT {date} {ticker} {tx['ratio']}"
                elif tx_type == "UNSPLIT":
                    # UNSPLIT DD/MM/YYYY TICKER MULTIPLIER
                    cgt_line = f"UNSPLIT {date} {ticker} {tx['ratio']}"
                else:
                    continue

                lines.append(cgt_line)

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
