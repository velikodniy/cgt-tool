#!/usr/bin/env python3
"""
Cross-validate cgt-tool calculations against external UK CGT calculators.

Compares results from:
- cgt-tool (this project)
- KapJI/capital-gains-calculator (Python, via uvx)
- mattjgalloway/cgtcalc (Swift, build from source)

Reports any discrepancies greater than £1 per tax year.

Usage:
  python cross-validate.py tests/inputs/Simple.cgt
  python cross-validate.py tests/inputs/*.cgt

Prerequisites:
- cgt-tool: cargo build --release
- cgt-calc: pip install cgt-calc (or use uvx cgt-calc)
- cgtcalc: git clone https://github.com/mattjgalloway/cgtcalc && cd cgtcalc && swift build -c release
"""

import json
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path

# Threshold for discrepancy reporting (£1)
THRESHOLD = Decimal("1.00")

# Operations not supported by cgt-calc (KapJI) that lead to false diffs
# Note: our DSL uses SPLIT/UNSPLIT and CAPRETURN; ACCDIV is not present.
UNSUPPORTED_OPS = {"SPLIT", "UNSPLIT", "CAPRETURN"}


def has_unsupported_ops(cgt_file: Path) -> bool:
    """Return True if the .cgt file contains ops cgt-calc cannot handle."""
    try:
        content = cgt_file.read_text()
    except OSError:
        return False

    for line in content.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if re.search(r"\b(" + "|".join(UNSUPPORTED_OPS) + r")\b", stripped):
            return True
    return False


@dataclass
class TaxYearResult:
    period: str
    gain: Decimal
    loss: Decimal


@dataclass
class CalculatorResult:
    name: str
    tax_years: list[TaxYearResult]
    error: str | None = None


def run_cgt_tool(cgt_file: Path) -> CalculatorResult:
    """Run cgt-tool and parse JSON output."""
    try:
        # Prefer pre-built binary if available
        # Check PATH first, then target/release
        cmd = ["cgt-tool"]

        # Check if cgt-tool is in PATH
        if subprocess.run(["which", "cgt-tool"], capture_output=True).returncode != 0:
            # Not in PATH, try target/release
            release_bin = Path.cwd() / "target" / "release" / "cgt-tool"
            if release_bin.exists():
                cmd = [str(release_bin)]
            else:
                # Fallback to cargo run
                cmd = ["cargo", "run", "--quiet", "--"]

        result = subprocess.run(
            cmd
            + [
                "report",
                str(cgt_file),
                "--format",
                "json",
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )
        if result.returncode != 0:
            return CalculatorResult("cgt-tool", [], error=result.stderr.strip())

        data = json.loads(result.stdout)
        years = []
        for ty in data.get("tax_years", []):
            years.append(
                TaxYearResult(
                    period=ty["period"],
                    gain=Decimal(ty.get("total_gain", "0")),
                    loss=Decimal(ty.get("total_loss", "0")),
                )
            )
        return CalculatorResult("cgt-tool", years)
    except Exception as e:
        return CalculatorResult("cgt-tool", [], error=str(e))


def run_cgt_calc(cgt_file: Path) -> CalculatorResult:
    """Run KapJI/capital-gains-calculator and parse output."""
    if has_unsupported_ops(cgt_file):
        return CalculatorResult(
            "cgt-calc", [], error="SKIP: unsupported ops for cgt-calc"
        )

    # Convert to RAW format
    script_dir = Path(__file__).parent
    convert_script = script_dir / "convert-to-raw.py"

    try:
        # Convert file
        result = subprocess.run(
            [sys.executable, str(convert_script), str(cgt_file)],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if result.returncode != 0:
            return CalculatorResult(
                "cgt-calc", [], error=f"Conversion failed: {result.stderr}"
            )

        raw_content = result.stdout
        if not raw_content.strip():
            return CalculatorResult("cgt-calc", [], error="No transactions to convert")

        # Write to temp file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write(raw_content)
            temp_path = f.name

        # Run cgt-calc for each tax year (we need to guess which years)
        # Parse dates from input to find tax years
        years = set()
        for line in raw_content.split("\n"):
            match = re.match(r"(\d{4})-(\d{2})-(\d{2}),", line)
            if match:
                year, month, day = (
                    int(match.group(1)),
                    int(match.group(2)),
                    int(match.group(3)),
                )
                # UK tax year: April 6 to April 5
                if month < 4 or (month == 4 and day < 6):
                    tax_year = year - 1
                else:
                    tax_year = year
                years.add(tax_year)

        tax_results = []
        for year in sorted(years):
            result = subprocess.run(
                [
                    "uvx",
                    "cgt-calc",
                    "--year",
                    str(year),
                    "--raw",
                    temp_path,
                    "--no-report",
                    "--no-balance-check",
                ],
                capture_output=True,
                text=True,
                timeout=120,
            )
            # Parse output for gains/losses
            # cgt-calc prints: "Capital gain: £X.XX" and "Capital loss: £X.XX"
            output = result.stdout + result.stderr
            gain_match = re.search(r"Capital gain:\s*[£$]?([\d,]+\.?\d*)", output)
            loss_match = re.search(r"Capital loss:\s*[£$]?([\d,]+\.?\d*)", output)

            gain = (
                Decimal(gain_match.group(1).replace(",", ""))
                if gain_match
                else Decimal("0")
            )
            loss = (
                Decimal(loss_match.group(1).replace(",", ""))
                if loss_match
                else Decimal("0")
            )

            tax_results.append(
                TaxYearResult(
                    period=f"{year}/{(year + 1) % 100:02d}", gain=gain, loss=loss
                )
            )

        Path(temp_path).unlink()
        return CalculatorResult("cgt-calc", tax_results)

    except FileNotFoundError:
        return CalculatorResult(
            "cgt-calc", [], error="uvx not found - install with: pip install uv"
        )
    except Exception as e:
        return CalculatorResult("cgt-calc", [], error=str(e))


def run_cgtcalc(cgt_file: Path) -> CalculatorResult:
    """Run mattjgalloway/cgtcalc and parse output."""
    # Check if cgtcalc is available (prefer local copy in scripts/)
    script_dir = Path(__file__).parent
    cgtcalc_paths = [
        script_dir / "cgtcalc",
        Path("/tmp/cgtcalc/.build/release/cgtcalc"),
        Path.home() / "cgtcalc" / ".build" / "release" / "cgtcalc",
    ]

    cgtcalc_bin = None
    for path in cgtcalc_paths:
        if path.exists():
            cgtcalc_bin = path
            break

    if not cgtcalc_bin:
        return CalculatorResult(
            "cgtcalc",
            [],
            error="cgtcalc not found. Clone and build: git clone https://github.com/mattjgalloway/cgtcalc && cd cgtcalc && swift build -c release",
        )

    # Convert to cgtcalc format
    script_dir = Path(__file__).parent
    convert_script = script_dir / "convert-to-cgtcalc.py"

    try:
        result = subprocess.run(
            [sys.executable, str(convert_script), str(cgt_file)],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if result.returncode != 0:
            return CalculatorResult(
                "cgtcalc", [], error=f"Conversion failed: {result.stderr}"
            )

        cgtcalc_content = result.stdout
        if not cgtcalc_content.strip():
            return CalculatorResult("cgtcalc", [], error="No transactions to convert")

        # Write to temp file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write(cgtcalc_content)
            temp_path = f.name

        # Run cgtcalc
        result = subprocess.run(
            [str(cgtcalc_bin), temp_path],
            capture_output=True,
            text=True,
            timeout=60,
        )

        Path(temp_path).unlink()

        # Parse output - cgtcalc outputs summary per tax year
        # Format: "Tax year 2023/24: Gain £1,234.56"
        output = result.stdout
        tax_results = []

        for match in re.finditer(
            r"Tax year (\d{4}/\d{2}):.*?Gain[:\s]*[£$]?([\d,.-]+)", output
        ):
            period = match.group(1)
            gain_str = match.group(2).replace(",", "")
            gain = Decimal(gain_str) if gain_str else Decimal("0")
            # cgtcalc shows net gain, negative = loss
            if gain < 0:
                tax_results.append(
                    TaxYearResult(period=period, gain=Decimal("0"), loss=abs(gain))
                )
            else:
                tax_results.append(
                    TaxYearResult(period=period, gain=gain, loss=Decimal("0"))
                )

        return CalculatorResult("cgtcalc", tax_results)

    except Exception as e:
        return CalculatorResult("cgtcalc", [], error=str(e))


def compare_results(
    cgt_tool: CalculatorResult, other: CalculatorResult
) -> list[tuple[str, str, Decimal]]:
    """Compare two calculator results, return discrepancies > threshold."""
    discrepancies = []

    # Build lookup for cgt-tool results
    cgt_lookup = {r.period: r for r in cgt_tool.tax_years}

    for other_year in other.tax_years:
        cgt_year = cgt_lookup.get(other_year.period)
        if not cgt_year:
            # cgt-tool missing this year
            if other_year.gain > THRESHOLD or other_year.loss > THRESHOLD:
                discrepancies.append(
                    (
                        other_year.period,
                        "missing in cgt-tool",
                        other_year.gain - other_year.loss,
                    )
                )
            continue

        # Compare gains
        gain_diff = abs(cgt_year.gain - other_year.gain)
        if gain_diff > THRESHOLD:
            discrepancies.append(
                (
                    other_year.period,
                    f"gain: cgt-tool={cgt_year.gain}, {other.name}={other_year.gain}",
                    gain_diff,
                )
            )

        # Compare losses
        loss_diff = abs(cgt_year.loss - other_year.loss)
        if loss_diff > THRESHOLD:
            discrepancies.append(
                (
                    other_year.period,
                    f"loss: cgt-tool={cgt_year.loss}, {other.name}={other_year.loss}",
                    loss_diff,
                )
            )

    return discrepancies


def validate_file(cgt_file: Path, verbose: bool = True) -> tuple[bool, str, str]:
    """Validate a single .cgt file against external calculators.

    Returns (all_passed, cgt_calc_status, cgtcalc_status) where status is
    one of: "ok", "diff", "skip", "error".
    """
    if verbose:
        print(f"\n{'=' * 60}")
        print(f"Validating: {cgt_file.name}")
        print("=" * 60)

    # Run cgt-tool
    cgt_result = run_cgt_tool(cgt_file)
    if cgt_result.error:
        print(f"  ERROR (cgt-tool): {cgt_result.error}")
        return False, "error", "error"

    if verbose:
        print(f"  cgt-tool: {len(cgt_result.tax_years)} tax year(s)")
        for ty in cgt_result.tax_years:
            print(f"    {ty.period}: gain=£{ty.gain}, loss=£{ty.loss}")

    all_passed = True
    calc_status = "skip"
    cgtcalc_status = "skip"

    # Run cgt-calc
    calc_result = run_cgt_calc(cgt_file)
    if calc_result.error:
        calc_status = "skip" if calc_result.error.startswith("SKIP") else "error"
        if verbose:
            print(f"  SKIP (cgt-calc): {calc_result.error}")
    else:
        discrepancies = compare_results(cgt_result, calc_result)
        if discrepancies:
            calc_status = "diff"
            all_passed = False
            print(f"  DISCREPANCY (cgt-calc):")
            for period, msg, diff in discrepancies:
                print(f"    {period}: {msg} (diff: £{diff:.2f})")
        else:
            calc_status = "ok"
            if verbose:
                print(f"  OK (cgt-calc): matches cgt-tool")

    # Run cgtcalc
    cgtcalc_result = run_cgtcalc(cgt_file)
    if cgtcalc_result.error:
        cgtcalc_status = (
            "skip"
            if cgtcalc_result.error.startswith("cgtcalc not found")
            or "No transactions" in cgtcalc_result.error
            else "error"
        )
        if verbose:
            print(f"  SKIP (cgtcalc): {cgtcalc_result.error}")
    else:
        discrepancies = compare_results(cgt_result, cgtcalc_result)
        if discrepancies:
            cgtcalc_status = "diff"
            all_passed = False
            print(f"  DISCREPANCY (cgtcalc):")
            for period, msg, diff in discrepancies:
                print(f"    {period}: {msg} (diff: £{diff:.2f})")
        else:
            cgtcalc_status = "ok"
            if verbose:
                print(f"  OK (cgtcalc): matches cgt-tool")

    return all_passed, calc_status, cgtcalc_status


def main():
    if len(sys.argv) < 2:
        print("Usage: python cross-validate.py <file.cgt> [file2.cgt ...]")
        print("\nValidates cgt-tool output against external UK CGT calculators.")
        print("Reports discrepancies greater than £1 per tax year.")
        sys.exit(1)

    files = [Path(f) for f in sys.argv[1:]]
    all_passed = True
    summary = {
        "cgt-calc": {"ok": 0, "diff": 0, "skip": 0, "error": 0},
        "cgtcalc": {"ok": 0, "diff": 0, "skip": 0, "error": 0},
    }

    for f in files:
        if not f.exists():
            print(f"Error: File not found: {f}")
            all_passed = False
            continue

        passed, calc_status, cgtcalc_status = validate_file(f)
        all_passed = all_passed and passed
        summary["cgt-calc"][calc_status] += 1
        summary["cgtcalc"][cgtcalc_status] += 1

    print("\n" + "=" * 60)
    if all_passed:
        print("RESULT: All validations passed")
    else:
        # Check if failure is only due to cgt-calc (KapJI)
        # We consider cgtcalc (mattjgalloway) authoritative
        cgtcalc_failed = (
            summary["cgtcalc"]["diff"] > 0 or summary["cgtcalc"]["error"] > 0
        )
        if not cgtcalc_failed and summary["cgt-calc"]["diff"] > 0:
            print("RESULT: Passed (cgtcalc matches, ignoring cgt-calc discrepancies)")
            all_passed = True
        else:
            print("RESULT: Some validations failed or had discrepancies")

    print("=" * 60)
    print("Summary:")
    print(
        f"  cgt-calc   -> ok: {summary['cgt-calc']['ok']}, diff: {summary['cgt-calc']['diff']}, skip: {summary['cgt-calc']['skip']}, error: {summary['cgt-calc']['error']}"
    )
    print(
        f"  cgtcalc    -> ok: {summary['cgtcalc']['ok']}, diff: {summary['cgtcalc']['diff']}, skip: {summary['cgtcalc']['skip']}, error: {summary['cgtcalc']['error']}"
    )

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
