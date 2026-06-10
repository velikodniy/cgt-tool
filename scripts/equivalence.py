#!/usr/bin/env python3
"""Numeric equivalence harness for the cgt-tool engine rewrite.

Modes:
  compare (default): run two binaries over fixtures, compare report JSON
                     numerically (tolerance 0) and classify differences.
  parse:             compare `parse` JSON output of two binaries exactly.
  sweep:             compare ONE binary's output against checked-in goldens
                     (tests/json/*.json numerically over the golden's keys,
                     tests/plain/*.txt by bytes) to detect stale goldens.

Buckets (compare mode):
  semantic        - any quantity/count/date/structure difference, and money
                    differences > 0.01.
  rounding-policy - money field differences with |delta| <= 0.01 (candidate
                    last-digit rounding differences; reviewed collectively).
  leg-granularity - per-rule sums and dates per disposal are equal but the
                    number of match legs differs (e.g. same-day aggregation).

Exit code 0 iff no differences outside --allow buckets and no errors.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from decimal import Decimal
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
INPUTS = REPO / "tests" / "inputs"
GOLDEN_JSON = REPO / "tests" / "json"
GOLDEN_PLAIN = REPO / "tests" / "plain"
ROUNDING_TOLERANCE = Decimal("0.01")

MONEY_LEAVES = {
    "total_gain", "total_loss", "net_gain", "exempt_amount",
    "dividend_income", "dividend_tax_paid", "gross_proceeds", "proceeds",
    "total_cost", "cost", "gain",
}


class ShapeError(Exception):
    """Report JSON lacks a recognizable report structure."""


def run(cmd: list[str]) -> tuple[int, str, str]:
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    return proc.returncode, proc.stdout, proc.stderr


def report_json(binary: str, fixture: Path) -> tuple[int, dict | None]:
    code, out, _err = run([binary, "report", str(fixture), "--format", "json"])
    if code != 0:
        return code, None
    return 0, json.loads(out, parse_float=Decimal, parse_int=Decimal)


def dec(value) -> Decimal:
    if isinstance(value, Decimal):
        return value
    return Decimal(str(value))


def get_first(mapping: dict, *names: str):
    """Read a field under any of its legacy/new aliases."""
    for name in names:
        if name in mapping:
            return mapping[name]
    raise ShapeError(f"none of {names} present in keys {sorted(mapping)}")


def view(report: dict) -> dict:
    """Reduce a report to a comparable numeric view.

    Per-disposal match legs are reduced to per-rule sums + leg count + the
    set of acquisition dates, so value-neutral leg regrouping is
    distinguishable from value or attribution changes. The `transactions`
    echo is ignored (covered by parse mode).
    """
    v: dict = {"years": {}, "disposals": {}, "holdings": {}}
    seq: dict[str, int] = {}
    for year in report.get("tax_years", []):
        period = str(get_first(year, "period"))
        y = {"disposal_count": int(dec(get_first(year, "disposal_count")))}
        for key in ("total_gain", "total_loss", "net_gain", "exempt_amount",
                    "dividend_income", "dividend_tax_paid"):
            if key in year:
                y[key] = dec(year[key])
        v["years"][period] = y
        for d in year.get("disposals", []):
            base = f"{period}|{d['date']}|{d['ticker']}"
            n = seq.get(base, 0)
            seq[base] = n + 1
            dk = f"{base}|{n}" if n else base
            entry = {
                "quantity": dec(get_first(d, "quantity", "qty")),
                "gross_proceeds": dec(d["gross_proceeds"]),
                "proceeds": dec(d["proceeds"]),
            }
            rules: dict = {}
            dates: dict[str, set] = {}
            for m in get_first(d, "matches", "legs"):
                rule = str(m["rule"])
                r = rules.setdefault(
                    rule,
                    {"qty": Decimal(0), "cost": Decimal(0),
                     "gain": Decimal(0), "legs": 0},
                )
                r["qty"] += dec(get_first(m, "quantity", "qty"))
                r["cost"] += dec(get_first(m, "allowable_cost", "cost"))
                r["gain"] += dec(get_first(m, "gain_or_loss", "gain"))
                r["legs"] += 1
                acq = m.get("acquisition_date", m.get("acq_date"))
                if acq:
                    dates.setdefault(rule, set()).add(str(acq))
            for rule, r in rules.items():
                r["dates"] = ",".join(sorted(dates.get(rule, set())))
            entry["rules"] = rules
            v["disposals"][dk] = entry
    for h in report.get("holdings", []):
        v["holdings"][h["ticker"]] = {
            "quantity": dec(h["quantity"]),
            "total_cost": dec(get_first(h, "total_cost", "cost")),
        }
    return v


def flatten(view_dict: dict, prefix: str = "") -> dict[str, object]:
    flat: dict[str, object] = {}
    for key, value in view_dict.items():
        path = f"{prefix}.{key}" if prefix else str(key)
        if isinstance(value, dict):
            flat.update(flatten(value, path))
        else:
            flat[path] = value
    return flat


def classify(path: str, old, new) -> str:
    leaf = path.rsplit(".", 1)[-1]
    if leaf == "legs":
        return "leg-granularity"
    if leaf in MONEY_LEAVES \
            and isinstance(old, Decimal) and isinstance(new, Decimal) \
            and abs(old - new) <= ROUNDING_TOLERANCE:
        return "rounding-policy"
    return "semantic"


def diff_views(old: dict, new: dict,
               one_sided: bool = False) -> list[tuple[str, str, object, object]]:
    """Return (bucket, path, old, new) tuples.

    one_sided: compare only keys present in `old` (golden-vs-live sweeps;
    fields the live binary added since the golden was written are not
    staleness). Missing keys are semantic either way.
    """
    out: list[tuple[str, str, object, object]] = []
    fo, fn = flatten(old), flatten(new)
    keys = sorted(fo) if one_sided else sorted(set(fo) | set(fn))
    for path in keys:
        if path not in fo:
            out.append(("semantic", path, "<absent>", fn[path]))
        elif path not in fn:
            out.append(("semantic", path, fo[path], "<absent>"))
        elif fo[path] != fn[path]:
            out.append((classify(path, fo[path], fn[path]),
                        path, fo[path], fn[path]))
    # Leg-count diffs whose sums or dates also differ are semantic.
    parents_with_value_diffs = {
        p.rsplit(".", 1)[0] for b, p, _, _ in out
        if p.rsplit(".", 1)[-1] in {"qty", "cost", "gain", "dates"}
    }
    return [(("semantic" if b == "leg-granularity"
              and p.rsplit(".", 1)[0] in parents_with_value_diffs else b),
             p, o, n)
            for b, p, o, n in out]


def fixtures(args_fixtures: list[str]) -> list[Path]:
    if args_fixtures:
        return [Path(f) for f in args_fixtures]
    return sorted(INPUTS.glob("*.cgt"))


def safe_view(report: dict, label: str) -> tuple[dict | None, str | None]:
    try:
        return view(report), None
    except (ShapeError, KeyError, TypeError) as exc:
        return None, f"{label} report shape unrecognized: {exc}"


def mode_compare(args) -> int:
    failures = 0
    totals: dict[str, int] = {}
    for fixture in fixtures(args.fixtures):
        old_code, old_rep = report_json(args.old, fixture)
        new_code, new_rep = report_json(args.new, fixture)
        if old_code != 0 or new_code != 0:
            if old_code != 0 and new_code != 0:
                print(f"BOTH-ERROR  {fixture.name}")
            else:
                print(f"SEMANTIC    {fixture.name}: exit codes "
                      f"old={old_code} new={new_code}")
                failures += 1
            continue
        old_view, old_err = safe_view(old_rep, "old")
        new_view, new_err = safe_view(new_rep, "new")
        if old_err or new_err:
            print(f"SEMANTIC    {fixture.name}: {old_err or new_err}")
            failures += 1
            continue
        diffs = diff_views(old_view, new_view)
        if not diffs:
            print(f"IDENTICAL   {fixture.name}")
            continue
        buckets = sorted({b for b, *_ in diffs})
        print(f"DIFFS       {fixture.name}: {', '.join(buckets)}")
        for bucket, path, old, new in diffs:
            totals[bucket] = totals.get(bucket, 0) + 1
            print(f"  [{bucket}] {path}: {old} -> {new}")
            if bucket not in args.allow:
                failures += 1
    print(f"\nbucket totals: {totals or 'none'}")
    return 1 if failures else 0


def mode_parse(args) -> int:
    failures = 0
    for fixture in fixtures(args.fixtures):
        old = run([args.old, "parse", str(fixture)])
        new = run([args.new, "parse", str(fixture)])
        if (old[0], new[0]) != (0, 0):
            status = "BOTH-ERROR" if old[0] and new[0] else "MISMATCH"
            print(f"{status}  {fixture.name}")
            failures += status == "MISMATCH"
            continue
        if json.loads(old[1], parse_float=Decimal) == \
                json.loads(new[1], parse_float=Decimal):
            print(f"IDENTICAL   {fixture.name}")
        else:
            print(f"MISMATCH    {fixture.name}")
            failures += 1
    return 1 if failures else 0


def mode_sweep(args) -> int:
    stale = 0
    for fixture in fixtures(args.fixtures):
        name = fixture.stem
        code, rep = report_json(args.old, fixture)
        golden_json = GOLDEN_JSON / f"{name}.json"
        golden_plain = GOLDEN_PLAIN / f"{name}.txt"
        notes = []
        if code != 0:
            print(f"ERROR       {name}: report exited {code}")
            continue
        live_view, live_err = safe_view(rep, "live")
        if golden_json.exists() and live_view is not None:
            golden = json.loads(golden_json.read_text(),
                                parse_float=Decimal, parse_int=Decimal)
            golden_view, golden_err = safe_view(golden, "golden")
            if golden_err:
                notes.append(f"json {golden_err}")
            else:
                diffs = diff_views(golden_view, live_view, one_sided=True)
                if diffs:
                    notes.append(f"json STALE ({len(diffs)} diffs)")
                    for bucket, path, old, new in diffs[:args.max_diffs]:
                        notes.append(f"    [{bucket}] {path}: {old} -> {new}")
        elif live_err:
            notes.append(f"json {live_err}")
        else:
            notes.append("json MISSING")
        if golden_plain.exists():
            pcode, pout, _ = run([args.old, "report", str(fixture),
                                  "--format", "plain"])
            if pcode != 0:
                notes.append(f"plain ERROR exit {pcode}")
            elif pout != golden_plain.read_text():
                notes.append("plain STALE (byte diff)")
        else:
            notes.append("plain MISSING")
        if notes:
            stale += 1
            print(f"STALE/GAP   {name}")
            for n in notes:
                print(f"  {n}")
        else:
            print(f"MATCH       {name}")
    print(f"\n{stale} fixture(s) with staleness or gaps")
    return 0  # sweep is informational


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=["compare", "parse", "sweep"],
                        default="compare")
    parser.add_argument("--old", required=True,
                        help="oracle binary (or the only binary, in sweep)")
    parser.add_argument("--new", help="candidate binary (compare/parse)")
    parser.add_argument("--allow", nargs="*", default=[],
                        choices=["rounding-policy", "leg-granularity"],
                        help="buckets that do not fail the run")
    parser.add_argument("--max-diffs", type=int, default=20)
    parser.add_argument("fixtures", nargs="*")
    args = parser.parse_args()
    if args.mode in {"compare", "parse"} and not args.new:
        parser.error("--new is required for compare/parse modes")
    return {"compare": mode_compare, "parse": mode_parse,
            "sweep": mode_sweep}[args.mode](args)


if __name__ == "__main__":
    sys.exit(main())
