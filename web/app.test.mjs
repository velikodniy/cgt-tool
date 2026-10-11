import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";
import { createRequire } from "node:module";

const wasm = process.env.CGT_WASM_TEST_PKG
  ? createRequire(import.meta.url)(process.env.CGT_WASM_TEST_PKG)
  : null;

function element(acceptHtml = false) {
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
      if (acceptHtml && this.html !== undefined) return this.html;
      return (this.textContent || "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
    },
    set innerHTML(value) {
      if (acceptHtml) { this.html = value; return; }
      assert.fail("Toast content must not use innerHTML");
    },
  };
}

async function loadApp(bindings = {}) {
  const elements = new Map();
  const document = {
    createElement: () => element(),
    getElementById(id) {
      if (!elements.has(id)) elements.set(id, element(id === "output"));
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
    calculate_display() {},
    parse_transactions() {},
    validate_dsl() {},
    ...bindings,
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
    period: "2024/25", disposals: [], gross_proceeds: "£0.00", total_allowable_cost: "£0.00",
    total_gain: "£0.00", total_loss: "£0.00", net_gain: "£0.00", exempt_amount: "£3,000.00",
    taxable_gain: "£0.00", dividend: { income: "£42.50", tax_paid: "£6.75" },
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
    date: { iso: "2024-06-01", uk: "01/06/2024" },
    quantity: "0.015",
    gross_proceeds: "£9,007,199,254,740,993.29",
    proceeds: "£9,007,199,254,740,993.29",
    allowable_cost: "£0.30",
    result: { kind: "Gain", signed: "£9,007,199,254,740,992.99", magnitude: "£9,007,199,254,740,992.99" },
    matches: [{
      rule: "Section104",
      rule_label: "Section 104",
      unit_cost: { money: "£20.00", compact: "£20" },
      quantity: "0.015",
      allowable_cost: "£0.30",
      gain_or_loss: "£9,007,199,254,740,992.99",
    }],
  };
  const html = vm.runInContext("ReportBuilder.disposalCard(disposal, 0)", context);
  assert.ok(html.includes("£9,007,199,254,740,992.99"));
  assert.ok(html.includes("0.015 shares @ £20.00"));
});

test("holding cards render prepared averages while calculation costs stay exact", async () => {
  const { context } = await loadApp();
  const report = JSON.parse(await readFile(new URL("../tests/renderers/holding_precision.json", import.meta.url), "utf8"));
  context.holdings = report.holdings;
  assert.equal(context.holdings[0].total_cost, "10.0005");
  assert.equal(Object.hasOwn(context.holdings[0], "average_cost"), false);
  context.holdings = [{
    ticker: report.holdings[0].ticker, quantity: report.holdings[0].quantity,
    total_cost: "£10.00", unit_cost: { money: "£100.01", compact: "£100.01" },
  }];
  const html = vm.runInContext("ReportBuilder.holdings(holdings)", context);
  assert.ok(html.includes("£10.00"));
  assert.ok(html.includes("£100.01"));
  assert.ok(!html.includes("£100.00"));
});


test("report action consumes WASM display data and JSON action retains exact data", {
  skip: wasm ? false : "Set CGT_WASM_TEST_PKG to a wasm-pack Node.js build",
}, async () => {
  const { context, document } = await loadApp(wasm);
  const input = await readFile(new URL("../tests/renderers/holding_precision.cgt", import.meta.url), "utf8") + `
2024-01-01 BUY UNIT 100 @ 1.001
2024-06-01 SELL UNIT 100 @ 1.005
2024-06-01 DIVIDEND UNIT TOTAL 42.5 TAX 6.75
`;
  document.getElementById("input").value = input;
  document.getElementById("taxYear").value = "2024";
  document.getElementById("yearCheckbox").checked = true;
  context.generateReport();
  const html = document.getElementById("output").innerHTML;
  assert.ok(html.includes("£100.01"), html);
  assert.ok(html.includes("£100.50"), html);
  assert.ok(html.includes("£42.50"), html);
  const display = JSON.parse(wasm.calculate_display(input, 2024));
  for (const note of display.notes) assert.ok(html.includes(note.text), note.text);
  assert.equal(display.tax_years[0].disposals[0].sale_unit_price, "£1.005");
  assert.ok(!html.includes("SA108 Box"));
  context.calculateTax();
  const calculation = JSON.parse(document.getElementById("output").textContent);
  const holding = calculation.holdings.find((row) => row.ticker === "FRACTION");
  assert.equal(holding.total_cost, "10.0005");
  assert.equal(Object.hasOwn(holding, "unit_cost"), false);
  assert.equal(Object.hasOwn(calculation, "notes"), false);
});

test("prepared report text remains escaped at the HTML boundary", async () => {
  const { context } = await loadApp();
  context.notes = [{ text: '<img src=x onerror="alert(1)"> & text' }];
  const html = vm.runInContext("ReportBuilder.notes(notes)", context);
  assert.ok(html.includes('&lt;img src=x onerror="alert(1)"&gt; &amp; text'));
  assert.ok(!html.includes("<img"));
});
