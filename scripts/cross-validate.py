#!/usr/bin/env python3
"""Cross-validate cgt-tool calculations against external UK CGT calculators.

Compares per-tax-year gains and losses from:
- cgt-tool (this project)
- KapJI/capital-gains-calculator (Python, via uvx)
- mattjgalloway/cgtcalc (Swift, built from source)

Reports any discrepancy greater than £1 per tax year.

Usage:
  python cross-validate.py tests/inputs/Simple.cgt
  python cross-validate.py tests/inputs/*.cgt

Prerequisites:
- cgt-tool: cargo build --release
- cgt-calc: pip install uv (invoked as `uvx cgt-calc`)
- cgtcalc: git clone https://github.com/mattjgalloway/cgtcalc && cd cgtcalc && swift build -c release
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from collections.abc import Iterator
from contextlib import contextmanager
from dataclasses import dataclass
from decimal import Decimal
from enum import StrEnum
from pathlib import Path

from cgt_parse import find_cgt_tool

SCRIPT_DIR = Path(__file__).resolve().parent

# Discrepancy reporting threshold (£1 per tax year).
THRESHOLD = Decimal("1.00")

# Operations not supported by cgt-calc (KapJI) that would cause false diffs.
# ACCUMULATION adjusts cost basis (no CGT impact); cgt-calc has no equivalent.
UNSUPPORTED_OPS = {"SPLIT", "UNSPLIT", "CAPRETURN", "ACCUMULATION"}

# Fixtures whose cgtcalc (mattjgalloway) discrepancy is expected, not a
# regression. Each has a same-day or Bed & Breakfast disposal followed by a
# corporate action dated AFTER it. cgtcalc applies that later action as an
# "offset" onto the already-disposed shares (its output literally prints
# "offset of £15.81" — the post-disposal CAPRETURN+ACCUMULATION net), i.e. the
# legacy retroactive-cost-leak. cgt-tool follows HMRC CG51560 (matched shares
# leave the pool) and TCGA92/S110(8)(d) (a distribution adjusts only shares
# still held), so the later action lands in the carried-forward holding, not
# the disposed leg. Adjudicated against HMRC (docs adjudication 2026-06-18,
# re-verified 2026-06-21); cgt-tool is authoritative. SyntheticComplex is not
# listed here because it is multi-currency and so skipped for cgtcalc.
KNOWN_CGTCALC_DIVERGENCES = {
    "WithAssetEventsBB",
    "WithAssetEventsSameDay",
}

# Matches an UNSUPPORTED_OPS token as a whole word anywhere on a DSL line.
_UNSUPPORTED_OP_RE = re.compile(r"\b(" + "|".join(sorted(UNSUPPORTED_OPS)) + r")\b")
# Matches the optional 3-letter currency code after an "@ <price>" token.
_CURRENCY_RE = re.compile(r"@\s*[\d.]+\s+([A-Z]{3})\b")
# A RAW CSV row begins with an ISO date: "YYYY-MM-DD,...".
_RAW_DATE_RE = re.compile(r"(\d{4})-(\d{2})-(\d{2}),")
# cgt-calc prints "Capital gain: £X.XX" / "Capital loss: £X.XX".
_CGT_CALC_GAIN_RE = r"Capital gain:\s*[£$]?([\d,]+\.?\d*)"
_CGT_CALC_LOSS_RE = r"Capital loss:\s*[£$]?([\d,]+\.?\d*)"
# cgtcalc per-tax-year summary, e.g.:
#   2019/2020: Disposals = 1, proceeds = 7768, allowable costs = 7503,
#   total gains = 265, total losses = 0
_CGTCALC_SUMMARY_RE = re.compile(
    r"(\d{4})/(\d{4}):.*?Disposals\s*=\s*(\d+).*?"
    r"total gains?\s*=\s*([\d,.-]+).*?total losses?\s*=\s*([\d,.-]+)"
)


class Status(StrEnum):
    """Outcome of comparing one external calculator against cgt-tool."""

    OK = "ok"
    DIFF = "diff"
    SKIP = "skip"
    ERROR = "error"
    KNOWN = "known"


@dataclass
class TaxYearResult:
    period: str
    gain: Decimal
    loss: Decimal
    # Disposals cgtcalc reported for the year. cgtcalc rounds each disposal's
    # gain/loss down to whole pounds, so the comparison tolerance is scaled by
    # this count (0 = unknown/other calculators, keeping the base threshold).
    disposals: int = 0


@dataclass
class CalculatorResult:
    name: str
    tax_years: list[TaxYearResult]
    error: str | None = None
    # True when `error` is an expected limitation (reported as SKIP, not ERROR).
    skipped: bool = False


@dataclass
class Discrepancy:
    period: str
    detail: str
    amount: Decimal


class ConversionFailed(Exception):
    """A converter script exited non-zero."""


class ConversionEmpty(Exception):
    """A converter produced no transactions to compare."""


def run(cmd: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    """Run a subprocess, capturing its text output."""
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)


def significant_lines(cgt_file: Path) -> list[str]:
    """Stripped DSL lines, excluding blanks and comments; [] if unreadable."""
    try:
        text = cgt_file.read_text()
    except OSError:
        return []
    return [
        stripped
        for line in text.splitlines()
        if (stripped := line.strip()) and not stripped.startswith("#")
    ]


def has_unsupported_ops(cgt_file: Path) -> bool:
    """True if the file uses operations cgt-calc cannot model."""
    return any(_UNSUPPORTED_OP_RE.search(line) for line in significant_lines(cgt_file))


def has_foreign_currency(cgt_file: Path) -> bool:
    """True if any disposal is priced in a non-GBP currency.

    cgtcalc is GBP-only and convert-to-cgtcalc.py strips the currency code, so a
    foreign-currency fixture would be compared at face value against cgt-tool's
    HMRC-FX-converted figures — a guaranteed false discrepancy.
    """
    for line in significant_lines(cgt_file):
        match = _CURRENCY_RE.search(line)
        if match and match.group(1) != "GBP":
            return True
    return False


@contextmanager
def temp_text_file(content: str, suffix: str = ".txt") -> Iterator[Path]:
    """Write `content` to a temporary file, yielding its path and cleaning up."""
    with tempfile.NamedTemporaryFile("w", suffix=suffix, delete=False) as handle:
        handle.write(content)
        path = Path(handle.name)
    try:
        yield path
    finally:
        path.unlink(missing_ok=True)


def convert(cgt_file: Path, converter: str) -> str:
    """Run a sibling converter script and return its stdout."""
    result = run(
        [sys.executable, str(SCRIPT_DIR / converter), str(cgt_file)], timeout=30
    )
    if result.returncode != 0:
        raise ConversionFailed(result.stderr)
    if not result.stdout.strip():
        raise ConversionEmpty
    return result.stdout


def first_decimal(text: str, pattern: str) -> Decimal:
    """First regex capture in `text` as a Decimal (commas stripped); 0 if none."""
    match = re.search(pattern, text)
    return Decimal(match.group(1).replace(",", "")) if match else Decimal("0")


def find_cgtcalc() -> Path | None:
    """Locate a cgtcalc binary, preferring a copy in scripts/."""
    candidates = [
        SCRIPT_DIR / "cgtcalc",
        Path("/tmp/cgtcalc/.build/release/cgtcalc"),
        Path.home() / "cgtcalc" / ".build" / "release" / "cgtcalc",
    ]
    return next((path for path in candidates if path.exists()), None)


def tax_years_in_raw(raw_csv: str) -> list[int]:
    """UK tax-year start years present in RAW CSV rows (April 6 boundary)."""
    years: set[int] = set()
    for line in raw_csv.splitlines():
        match = _RAW_DATE_RE.match(line)
        if not match:
            continue
        year, month, day = (int(group) for group in match.groups())
        if month < 4 or (month == 4 and day < 6):
            year -= 1
        years.add(year)
    return sorted(years)


def run_cgt_tool(cgt_file: Path) -> CalculatorResult:
    """Run this project's cgt-tool and read its JSON report."""
    try:
        result = run(find_cgt_tool() + ["report", str(cgt_file), "--format", "json"])
        if result.returncode != 0:
            return CalculatorResult("cgt-tool", [], error=result.stderr.strip())
        data = json.loads(result.stdout)
        years = [
            TaxYearResult(
                period=year["period"],
                gain=Decimal(year.get("total_gain", "0")),
                loss=Decimal(year.get("total_loss", "0")),
            )
            for year in data.get("tax_years", [])
        ]
        return CalculatorResult("cgt-tool", years)
    except Exception as exc:
        return CalculatorResult("cgt-tool", [], error=str(exc))


def run_cgt_calc_year(raw_file: Path, year: int) -> TaxYearResult:
    """Run cgt-calc for a single tax year and read its gain/loss summary."""
    result = run(
        [
            "uvx",
            "cgt-calc",
            "--year",
            str(year),
            "--raw",
            str(raw_file),
            "--no-report",
            "--no-balance-check",
        ]
    )
    output = result.stdout + result.stderr
    return TaxYearResult(
        period=f"{year}/{(year + 1) % 100:02d}",
        gain=first_decimal(output, _CGT_CALC_GAIN_RE),
        loss=first_decimal(output, _CGT_CALC_LOSS_RE),
    )


def run_cgt_calc(cgt_file: Path) -> CalculatorResult:
    """Run KapJI/capital-gains-calculator, one pass per tax year."""
    if has_unsupported_ops(cgt_file):
        return CalculatorResult(
            "cgt-calc", [], error="SKIP: unsupported ops for cgt-calc", skipped=True
        )
    try:
        raw_csv = convert(cgt_file, "convert-to-raw.py")
        with temp_text_file(raw_csv) as raw_file:
            years = [
                run_cgt_calc_year(raw_file, year) for year in tax_years_in_raw(raw_csv)
            ]
        return CalculatorResult("cgt-calc", years)
    except ConversionFailed as exc:
        return CalculatorResult("cgt-calc", [], error=f"Conversion failed: {exc}")
    except ConversionEmpty:
        return CalculatorResult("cgt-calc", [], error="No transactions to convert")
    except FileNotFoundError:
        return CalculatorResult(
            "cgt-calc", [], error="uvx not found - install with: pip install uv"
        )
    except Exception as exc:
        return CalculatorResult("cgt-calc", [], error=str(exc))


def parse_cgtcalc_summary(output: str) -> list[TaxYearResult]:
    """Parse cgtcalc's per-tax-year summary lines into results.

    cgtcalc prints the period as YYYY/YYYY and rounds each disposal's gain/loss
    down to whole pounds; we normalise the period to cgt-tool's YYYY/YY form and
    keep the disposal count so the comparison tolerance can scale with it.
    """
    results = []
    for match in _CGTCALC_SUMMARY_RE.finditer(output):
        start_year, end_year, disposals, gains, losses = match.groups()
        results.append(
            TaxYearResult(
                period=f"{start_year}/{end_year[-2:]}",
                gain=Decimal(gains.replace(",", "") or "0"),
                loss=Decimal(losses.replace(",", "") or "0"),
                disposals=int(disposals),
            )
        )
    return results


def run_cgtcalc(cgt_file: Path) -> CalculatorResult:
    """Run mattjgalloway/cgtcalc and read its per-tax-year summary."""
    binary = find_cgtcalc()
    if binary is None:
        return CalculatorResult(
            "cgtcalc",
            [],
            error="cgtcalc not found. Clone and build: git clone https://github.com/mattjgalloway/cgtcalc && cd cgtcalc && swift build -c release",
            skipped=True,
        )
    if has_foreign_currency(cgt_file):
        return CalculatorResult(
            "cgtcalc",
            [],
            error="SKIP: foreign currency not supported by cgtcalc",
            skipped=True,
        )
    try:
        cgtcalc_input = convert(cgt_file, "convert-to-cgtcalc.py")
        with temp_text_file(cgtcalc_input) as input_file:
            result = run([str(binary), str(input_file)], timeout=60)

        output = result.stdout + result.stderr

        # cgtcalc aborts on inputs it cannot model (e.g. multi-currency, or a
        # CAPRETURN whose amount fails its internal validation). Surface that as
        # an explicit limitation instead of parsing zero tax years and reporting
        # a spurious match.
        if result.returncode != 0 or "Error calculating CGT" in output:
            reason = next(
                (line.strip() for line in output.splitlines() if "Error" in line),
                f"cgtcalc exited {result.returncode}",
            )
            return CalculatorResult(
                "cgtcalc", [], error=f"cgtcalc cannot process: {reason}", skipped=True
            )

        return CalculatorResult("cgtcalc", parse_cgtcalc_summary(output))
    except ConversionFailed as exc:
        return CalculatorResult("cgtcalc", [], error=f"Conversion failed: {exc}")
    except ConversionEmpty:
        return CalculatorResult(
            "cgtcalc", [], error="No transactions to convert", skipped=True
        )
    except Exception as exc:
        return CalculatorResult("cgtcalc", [], error=str(exc))


def compare_results(
    cgt_tool: CalculatorResult, other: CalculatorResult
) -> list[Discrepancy]:
    """Discrepancies between cgt-tool and another calculator, above tolerance."""
    by_period = {year.period: year for year in cgt_tool.tax_years}
    discrepancies: list[Discrepancy] = []

    for other_year in other.tax_years:
        # cgtcalc rounds each disposal's gain/loss down to whole pounds, so the
        # rounding noise grows with the disposal count; scale tolerance to suit.
        # Other calculators report disposals=0 and keep the base threshold.
        tolerance = (
            THRESHOLD * other_year.disposals if other_year.disposals else THRESHOLD
        )

        cgt_year = by_period.get(other_year.period)
        if cgt_year is None:
            if other_year.gain > tolerance or other_year.loss > tolerance:
                discrepancies.append(
                    Discrepancy(
                        other_year.period,
                        "missing in cgt-tool",
                        other_year.gain - other_year.loss,
                    )
                )
            continue

        for label, tool_value, other_value in (
            ("gain", cgt_year.gain, other_year.gain),
            ("loss", cgt_year.loss, other_year.loss),
        ):
            difference = abs(tool_value - other_value)
            if difference > tolerance:
                discrepancies.append(
                    Discrepancy(
                        other_year.period,
                        f"{label}: cgt-tool={tool_value}, {other.name}={other_value}",
                        difference,
                    )
                )

    return discrepancies


def report_discrepancies(header: str, discrepancies: list[Discrepancy]) -> None:
    """Print a discrepancy header followed by one indented line per item."""
    print(header)
    for item in discrepancies:
        print(f"    {item.period}: {item.detail} (diff: £{item.amount:.2f})")


def check_calculator(
    cgt_result: CalculatorResult,
    other_result: CalculatorResult,
    *,
    fixture: Path,
    allow_known: bool,
) -> Status:
    """Compare one external calculator against cgt-tool and report the outcome."""
    name = other_result.name
    if other_result.error:
        print(f"  SKIP ({name}): {other_result.error}")
        return Status.SKIP if other_result.skipped else Status.ERROR

    discrepancies = compare_results(cgt_result, other_result)
    if not discrepancies:
        print(f"  OK ({name}): matches cgt-tool")
        return Status.OK

    if allow_known and fixture.stem in KNOWN_CGTCALC_DIVERGENCES:
        report_discrepancies(
            f"  KNOWN DIVERGENCE ({name}, adjudicated):", discrepancies
        )
        return Status.KNOWN

    report_discrepancies(f"  DISCREPANCY ({name}):", discrepancies)
    return Status.DIFF


def validate_file(cgt_file: Path) -> tuple[bool, Status, Status]:
    """Validate one .cgt file against both external calculators.

    Returns (all_passed, cgt_calc_status, cgtcalc_status). A KNOWN cgtcalc
    divergence (see KNOWN_CGTCALC_DIVERGENCES) is reported but does not fail.
    """
    print(f"\n{'=' * 60}")
    print(f"Validating: {cgt_file.name}")
    print("=" * 60)

    cgt_result = run_cgt_tool(cgt_file)
    if cgt_result.error:
        print(f"  ERROR (cgt-tool): {cgt_result.error}")
        return False, Status.ERROR, Status.ERROR

    print(f"  cgt-tool: {len(cgt_result.tax_years)} tax year(s)")
    for year in cgt_result.tax_years:
        print(f"    {year.period}: gain=£{year.gain}, loss=£{year.loss}")

    calc_status = check_calculator(
        cgt_result, run_cgt_calc(cgt_file), fixture=cgt_file, allow_known=False
    )
    cgtcalc_status = check_calculator(
        cgt_result, run_cgtcalc(cgt_file), fixture=cgt_file, allow_known=True
    )

    all_passed = Status.DIFF not in (calc_status, cgtcalc_status)
    return all_passed, calc_status, cgtcalc_status


def print_summary(summary: dict[str, dict[Status, int]]) -> None:
    """Print the per-calculator tally of outcomes."""
    print("Summary:")
    for name, counts in summary.items():
        tally = ", ".join(f"{status.value}: {counts[status]}" for status in Status)
        print(f"  {name:<11}-> {tally}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate cgt-tool output against external UK CGT calculators "
        "(reports discrepancies greater than £1 per tax year).",
    )
    parser.add_argument(
        "files", nargs="+", type=Path, help=".cgt fixture(s) to validate"
    )
    args = parser.parse_args()

    all_passed = True
    summary: dict[str, dict[Status, int]] = {
        "cgt-calc": {status: 0 for status in Status},
        "cgtcalc": {status: 0 for status in Status},
    }

    for cgt_file in args.files:
        if not cgt_file.exists():
            print(f"Error: File not found: {cgt_file}")
            all_passed = False
            continue
        passed, calc_status, cgtcalc_status = validate_file(cgt_file)
        all_passed = all_passed and passed
        summary["cgt-calc"][calc_status] += 1
        summary["cgtcalc"][cgtcalc_status] += 1

    print("\n" + "=" * 60)
    if all_passed:
        print("RESULT: All validations passed")
    else:
        # cgtcalc (mattjgalloway) is authoritative. cgt-calc (KapJI) cannot
        # model every operation, so its discrepancies alone do not fail the run.
        cgtcalc_failed = (
            summary["cgtcalc"][Status.DIFF] > 0 or summary["cgtcalc"][Status.ERROR] > 0
        )
        if not cgtcalc_failed and summary["cgt-calc"][Status.DIFF] > 0:
            print("RESULT: Passed (cgtcalc matches, ignoring cgt-calc discrepancies)")
            all_passed = True
        else:
            print("RESULT: Some validations failed or had discrepancies")

    print("=" * 60)
    print_summary(summary)

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
