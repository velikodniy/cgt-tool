import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";
import * as decimal from "./decimal.mjs";

function element() {
  return {
    children: [],
    classList: { add() {}, remove() {} },
    addEventListener() {},
    setAttribute() {},
    append(...children) {
      this.children.push(...children);
    },
    appendChild(child) {
      this.children.push(child);
    },
    get innerHTML() {
      return (this.textContent || "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
    },
    set innerHTML(_value) {
      assert.fail("Toast content must not use innerHTML");
    },
  };
}

async function loadApp() {
  const elements = new Map();
  const document = {
    createElement: element,
    getElementById(id) {
      if (!elements.has(id)) elements.set(id, element());
      return elements.get(id);
    },
  };
  const source = await readFile(new URL("app.js", import.meta.url), "utf8");
  const context = vm.createContext({
    document,
    window: {},
    console,
    setTimeout() {},
    init: async () => {},
    calculate_tax() {},
    parse_transactions() {},
    validate_dsl() {},
    ...decimal,
  });
  vm.runInContext(source.replace(/^import .*;$/gm, ""), context);
  await Promise.resolve();
  document.getElementById("toastContainer").children.length = 0;
  return { context, document };
}

test("parser errors remain text in toast messages", async () => {
  const { context, document } = await loadApp();
  const message = 'Parsing error: <img src=x onerror="document.body.dataset.injected=1"> & text';
  context.showToast(message, "error");

  const [toast] = document.getElementById("toastContainer").children;
  assert.equal(toast.className, "toast error");
  assert.equal(toast.children.length, 2);
  assert.equal(toast.children[0].textContent, "✕");
  assert.equal(toast.children[1].textContent, message);
});

test("income-only year cards show income and tax paid", async () => {
  const { context } = await loadApp();
  context.incomeYear = {
    period: "2024/25", disposals: [], gross_proceeds: "0", total_allowable_cost: "0",
    total_gain: "0", total_loss: "0", net_gain: "0", exempt_amount: "3000",
    taxable_gain: "0", dividend_income: "42.50", dividend_tax_paid: "6.75",
  };
  const html = vm.runInContext("ReportBuilder.taxYear(incomeYear)", context);
  assert.ok(html.includes("Dividend Income"));
  assert.ok(html.includes("£42.50"));
  assert.ok(html.includes("Dividend Tax Paid"));
  assert.ok(html.includes("£6.75"));
});

test("report cards preserve large pennies and fractional unit costs", async () => {
  const { context } = await loadApp();
  context.disposal = {
    ticker: "TEST",
    date: "2024-06-01",
    quantity: "0.015",
    gross_proceeds: "9007199254740993.29",
    proceeds: "9007199254740993.29",
    matches: [{
      rule: "Section104",
      quantity: "0.015",
      allowable_cost: "0.30",
      gain_or_loss: "9007199254740992.99",
    }],
  };
  const html = vm.runInContext("ReportBuilder.disposalCard(disposal, 0)", context);
  assert.ok(html.includes("£9,007,199,254,740,992.99"));
  assert.ok(html.includes("0.015 shares @ £20.00"));
});

test("holding cards derive averages from exact report costs", async () => {
  const { context } = await loadApp();
  const report = JSON.parse(await readFile(new URL("../tests/renderers/holding_precision.json", import.meta.url), "utf8"));
  context.holdings = report.holdings;
  assert.equal(context.holdings[0].total_cost, "10.0005");
  assert.equal(Object.hasOwn(context.holdings[0], "average_cost"), false);
  const html = vm.runInContext("ReportBuilder.holdings(holdings)", context);
  assert.ok(html.includes("£10.00"));
  assert.ok(html.includes("£100.01"));
  assert.ok(!html.includes("£100.00"));
});
