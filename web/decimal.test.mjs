import assert from "node:assert/strict";
import test from "node:test";
import { formatCurrency, formatPence, sumPence, formatUnitCost, isPositive } from "./decimal.mjs";

test("large money values retain every penny", () => {
  assert.equal(formatCurrency("9007199254740992.99"), "£9,007,199,254,740,992.99");
  assert.equal(formatCurrency("-9007199254740992.99"), "-£9,007,199,254,740,992.99");
});

test("signed money sums stay exact through cancellation", () => {
  const total = sumPence(["9007199254740992.99", "-9007199254740992.98", "-0.02"]);
  assert.equal(total, -1n);
  assert.equal(formatPence(total), "-£0.01");
});

test("unit costs divide exact fractional share counts", () => {
  assert.equal(formatUnitCost("0.30", "0.015"), "£20.00");
  assert.equal(formatUnitCost("1.00", "0.0000001"), "£10,000,000.00");
  assert.equal(formatUnitCost("1.00", "0"), "£0.00");
  assert.equal(isPositive("0.0000000000000000000000000001"), true);
  assert.equal(isPositive("0.00"), false);
});

test("money and division use midpoint rounding away from zero", () => {
  assert.equal(formatCurrency("1.005"), "£1.01");
  assert.equal(formatCurrency("-1.005"), "-£1.01");
  assert.equal(formatCurrency("-0.004"), "£0.00");
  assert.equal(formatUnitCost("0.01", "2"), "£0.01");
  assert.equal(formatUnitCost("-0.01", "2"), "-£0.01");
});
