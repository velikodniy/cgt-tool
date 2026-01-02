#!/usr/bin/env python3

import argparse
import calendar
import sys
import urllib.error
import urllib.request
import datetime
import pathlib
from typing import Iterable, TypeAlias

DEFAULT_RATES_DIR = pathlib.Path("crates/cgt-money/resources/rates")
DEFAULT_START_YEAR = 2021
TIMEOUT = 15.0

YearMonth: TypeAlias = tuple[int, int]


def iter_months(start_year: int, now: datetime.datetime) -> Iterable[YearMonth]:
    for year in range(start_year, now.year + 1):
        for month in range(1, 13):
            if year == now.year and month > now.month:
                break
            yield year, month


def get_data_url(year: int, month: int) -> str:
    base_url = "https://www.trade-tariff.service.gov.uk/api/v2/exchange_rates/files"
    return f"{base_url}/monthly_xml_{year}-{month:02d}.xml"


def is_xml(data: bytes) -> bool:
    return data.startswith(b"<?xml")


def fetch_month(year: int, month: int, timeout: float) -> bytes | None:
    url = get_data_url(year, month)
    request = urllib.request.Request(url)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            data = response.read()
    except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError):
        return None
    return data if is_xml(data) else None


def download_rates(
    rates_dir: pathlib.Path,
    start_year: int,
    timeout: float,
    *,
    verbose: bool,
) -> tuple[list[YearMonth], int, int]:
    rates_dir.mkdir(parents=True, exist_ok=True)
    now = datetime.datetime.now(tz=datetime.UTC)

    downloaded: list[YearMonth] = []
    skipped = 0
    failed = 0

    for year, month in iter_months(start_year, now):
        target = rates_dir / f"{year}-{month:02d}.xml"
        if target.exists():
            skipped += 1
            continue

        data = fetch_month(year, month, timeout)
        if not data:
            failed += 1
            if verbose:
                print(f"failed {year}-{month:02d}", file=sys.stderr)
            continue

        target.write_bytes(data)
        downloaded.append((year, month))
        if verbose:
            print(f"downloaded {target.name}", file=sys.stderr)

    return downloaded, skipped, failed


def format_release_body(downloaded: list[YearMonth]) -> str:
    if not downloaded:
        return ""
    labels = [f"{calendar.month_name[m]} {y}" for y, m in sorted(downloaded)]
    return f"Updated HMRC FX rates for {', '.join(labels)}"


def emit_outputs(downloaded, skipped, failed):
    print(f"has-new={'true' if downloaded else 'false'}")
    print(f"release-body={format_release_body(downloaded)}")
    print(f"downloaded={len(downloaded)}")
    print(f"skipped={skipped}")
    print(f"failed={failed}")


def parse_args():
    p = argparse.ArgumentParser(description="Download HMRC FX rates")
    p.add_argument("rates_dir", nargs="?", type=pathlib.Path, default=DEFAULT_RATES_DIR)
    p.add_argument("start_year", nargs="?", type=int, default=DEFAULT_START_YEAR)
    p.add_argument("--timeout", type=float, default=TIMEOUT)
    p.add_argument("-v", "--verbose", action="store_true")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    downloaded, skipped, failed = download_rates(
        rates_dir=args.rates_dir,
        start_year=args.start_year,
        timeout=args.timeout,
        verbose=args.verbose,
    )
    emit_outputs(downloaded, skipped, failed)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
