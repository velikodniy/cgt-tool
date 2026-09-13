"""Tests for comparison failures and local binary selection."""

import importlib.util
import io
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from decimal import Decimal
from pathlib import Path
from unittest.mock import patch

import cgt_parse

SPEC = importlib.util.spec_from_file_location(
    "cross_validate", Path(__file__).with_name("cross-validate.py")
)
cv = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = cv
SPEC.loader.exec_module(cv)
FIXTURE = Path(__file__).resolve().parent.parent / "tests" / "inputs" / "Simple.cgt"


def result(name, gain="10", **kwargs):
    years = [cv.TaxYearResult("2024/25", Decimal(gain), Decimal("0"))]
    return cv.CalculatorResult(name, years, **kwargs)


class CalculatorFailureTests(unittest.TestCase):
    def run_main(self, calc, oracle, require_oracle="1"):
        output = io.StringIO()
        with (
            patch.object(sys, "argv", ["cross-validate.py", str(FIXTURE)]),
            patch.dict(os.environ, {"CGT_REQUIRE_CGTCALC": require_oracle}),
            patch.object(cv, "run_cgt_tool", return_value=result("cgt-tool")),
            patch.object(cv, "run_cgt_calc", return_value=calc),
            patch.object(cv, "run_cgtcalc", return_value=oracle),
            redirect_stdout(output),
        ):
            code = cv.main()
        return code, output.getvalue()

    def test_failed_cgt_calc_process_is_not_a_zero_result(self):
        failed = subprocess.CompletedProcess([], 1, "", "invalid input")
        with patch.object(cv, "run", return_value=failed):
            with self.assertRaisesRegex(RuntimeError, "exited 1"):
                cv.run_cgt_calc_year(FIXTURE, 2024)

    def test_missing_cgt_calc_summary_is_an_error(self):
        for output in ("", "Capital gain: £0.00"):
            with self.subTest(output=output):
                completed = subprocess.CompletedProcess([], 0, output, "")
                with patch.object(cv, "run", return_value=completed):
                    with self.assertRaisesRegex(ValueError, "gain/loss summary"):
                        cv.run_cgt_calc_year(FIXTURE, 2024)

    def test_completed_zero_summary_is_valid(self):
        completed = subprocess.CompletedProcess(
            [], 0, "Capital gain: £0.00\nCapital loss: £0.00", ""
        )
        with patch.object(cv, "run", return_value=completed):
            year = cv.run_cgt_calc_year(FIXTURE, 2024)
        self.assertEqual((year.gain, year.loss), (Decimal(0), Decimal(0)))

    def test_no_supported_transactions_is_an_applicability_skip(self):
        with (
            patch.object(cv, "has_unsupported_ops", return_value=False),
            patch.object(cv, "convert", side_effect=cv.ConversionEmpty),
        ):
            outcome = cv.run_cgt_calc(FIXTURE)
        self.assertTrue(outcome.skipped)

    def test_zero_quantity_accumulation_is_not_sent_to_cgtcalc(self):
        spec = importlib.util.spec_from_file_location(
            "convert_to_cgtcalc", Path(__file__).with_name("convert-to-cgtcalc.py")
        )
        converter = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(converter)
        transaction = {
            "action": "ACCUMULATION", "date": "2024-01-01", "ticker": "TEST",
            "amount": "0.00", "total_value": {"amount": "15", "currency": "GBP"},
        }
        with patch.object(converter, "run_parse", return_value=[transaction]):
            self.assertEqual(converter.convert_to_cgtcalc(FIXTURE), [])

    def test_failed_or_unparsed_cgtcalc_is_not_an_applicability_skip(self):
        for code, output in ((1, "fatal error"), (0, "unexpected output")):
            with self.subTest(code=code):
                completed = subprocess.CompletedProcess([], code, output, "")
                with (
                    patch.object(cv, "find_cgtcalc", return_value=Path("/tmp/cgtcalc")),
                    patch.object(cv, "has_foreign_currency", return_value=False),
                    patch.object(cv, "convert", return_value="BUY 01/01/2024 TEST 1 1 0"),
                    patch.object(cv, "run", return_value=completed),
                ):
                    outcome = cv.run_cgtcalc(FIXTURE)
                self.assertIsNotNone(outcome.error)
                self.assertFalse(outcome.skipped)

    def test_authoritative_error_fails_the_run(self):
        code, output = self.run_main(
            result("cgt-calc"), result("cgtcalc", error="summary cannot be parsed")
        )
        self.assertEqual(code, 1)
        self.assertIn("ERROR (cgtcalc)", output)
        self.assertNotIn("All validations passed", output)

    def test_optional_calculator_error_fails_the_run(self):
        code, _ = self.run_main(
            result("cgt-calc", error="process failed"), result("cgtcalc")
        )
        self.assertEqual(code, 1)

    def test_discrepancy_requires_an_authoritative_match(self):
        skipped = result("cgtcalc", error="unsupported", skipped=True)
        code, _ = self.run_main(result("cgt-calc", gain="30"), skipped, "0")
        self.assertEqual(code, 1)
        code, _ = self.run_main(result("cgt-calc", gain="30"), result("cgtcalc"))
        self.assertEqual(code, 0)

    def test_skipped_calculators_do_not_count_as_a_comparison(self):
        code, _ = self.run_main(
            result("cgt-calc", error="unsupported", skipped=True),
            result("cgtcalc", error="unsupported", skipped=True),
            "0",
        )
        self.assertEqual(code, 1)

    def test_offline_mode_still_requires_an_actual_comparison(self):
        missing = result("cgtcalc", error="not built", skipped=True, oracle_missing=True)
        code, _ = self.run_main(result("cgt-calc"), missing)
        self.assertEqual(code, 1)
        code, _ = self.run_main(result("cgt-calc"), missing, "0")
        self.assertEqual(code, 0)


class BinarySelectionTests(unittest.TestCase):
    def test_explicit_binary_override_is_one_command_token(self):
        with patch.dict(os.environ, {"CGT_TOOL_BIN": "/tmp/my build/cgt-tool"}):
            self.assertEqual(cgt_parse.find_cgt_tool(), ["/tmp/my build/cgt-tool"])

    def test_newest_build_comes_from_the_script_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            name = "cgt-tool.exe" if os.name == "nt" else "cgt-tool"
            debug = repo / "target" / "debug" / name
            release = repo / "target" / "release" / name
            for path, timestamp in ((debug, 200), (release, 100)):
                path.parent.mkdir(parents=True)
                path.write_text("test binary")
                os.utime(path, (timestamp, timestamp))
            with (
                patch.object(cgt_parse, "REPO", repo),
                patch.dict(os.environ, {"CGT_TOOL_BIN": ""}),
                patch.object(subprocess, "run", side_effect=AssertionError("PATH lookup")),
            ):
                self.assertEqual(cgt_parse.find_cgt_tool(), [str(debug)])

    def test_missing_build_uses_this_repository_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            with (
                patch.object(cgt_parse, "REPO", repo),
                patch.dict(os.environ, {"CGT_TOOL_BIN": ""}),
                patch.object(subprocess, "run", side_effect=AssertionError("PATH lookup")),
            ):
                command = cgt_parse.find_cgt_tool()
            self.assertEqual(command[:3], ["cargo", "run", "--quiet"])
            self.assertEqual(command[3:5], ["--manifest-path", str(repo / "Cargo.toml")])


if __name__ == "__main__":
    unittest.main()
