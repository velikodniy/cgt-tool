/* CGT Calculator WASM Demo - Application Logic */

import init, { calculate_tax, parse_transactions, validate_dsl } from "./pkg/cgt_wasm.js";

const state = {
  wasmReady: false,
  isProcessing: false,
};

const TOAST_DURATION = 3000;
const TOAST_ANIMATION_DURATION = 300;

async function initWasm() {
  try {
    showLoading(true);
    await init();

    window.wasmCalculateTax = calculate_tax;
    window.wasmParseTransactions = parse_transactions;
    window.wasmValidateDSL = validate_dsl;

    state.wasmReady = true;
    showToast("WASM module loaded successfully!", "success");
  } catch (error) {
    showToast(`Failed to load WASM module: ${error.message}`, "error");
    console.error("WASM initialization error:", error);
  } finally {
    showLoading(false);
  }
}

function showLoading(show) {
  const overlay = document.getElementById("loadingOverlay");
  if (show) {
    overlay.classList.add("active");
  } else {
    overlay.classList.remove("active");
  }
}

function showToast(message, type = "info") {
  const container = document.getElementById("toastContainer");
  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  toast.setAttribute("role", "alert");

  const icons = {
    success: "✓",
    error: "✕",
    info: "ℹ",
  };

  toast.innerHTML = `
        <span class="toast-icon" aria-hidden="true">${icons[type] || icons.info}</span>
        <span class="toast-message">${message}</span>
    `;

  container.appendChild(toast);

  setTimeout(() => {
    toast.classList.add("removing");
    setTimeout(() => {
      if (toast.parentNode) {
        container.removeChild(toast);
      }
    }, TOAST_ANIMATION_DURATION);
  }, TOAST_DURATION);
}

function toggleHelp(event) {
  event.preventDefault();
  const helpBox = document.getElementById("helpBox");
  const toggle = document.getElementById("helpToggle");

  const isVisible = helpBox.classList.contains("visible");

  if (isVisible) {
    helpBox.classList.remove("visible");
    toggle.textContent = "Show Syntax Help";
    toggle.setAttribute("aria-expanded", "false");
  } else {
    helpBox.classList.add("visible");
    toggle.textContent = "Hide Syntax Help";
    toggle.setAttribute("aria-expanded", "true");
  }
}

function toggleYearInput() {
  const checkbox = document.getElementById("yearCheckbox");
  const yearInput = document.getElementById("taxYear");

  yearInput.disabled = !checkbox.checked;
  if (!checkbox.checked) {
    yearInput.value = "";
  } else if (!yearInput.value) {
    yearInput.value = "2024";
  }
}

function formatCurrency(amount) {
  if (!amount) return "£0.00";
  const num = parseFloat(amount);
  return "£" + num.toLocaleString("en-GB", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

const ReportBuilder = {
  header: () => `
        <div class="report-header">
            <h2>UK Capital Gains Tax Report</h2>
        </div>
    `,

  summaryTable: (taxYears) => {
    const rows = taxYears
      .map(
        (ty) => `
            <tr>
                <td>${escapeHtml(ty.period || "Unknown")}</td>
                <td class="number">${formatCurrency(ty.net_gain)}</td>
                <td class="number">${formatCurrency(ty.total_proceeds)}</td>
                <td class="number">${formatCurrency(ty.exemption)}</td>
                <td class="number">${formatCurrency(ty.taxable_gain)}</td>
            </tr>
        `,
      )
      .join("");

    return `
            <h3 style="margin: 20px 0 10px 0; color: var(--color-text);">Summary</h3>
            <table class="data-table">
                <thead>
                    <tr>
                        <th>Tax Year</th>
                        <th class="number">Gain</th>
                        <th class="number">Proceeds</th>
                        <th class="number">Exemption</th>
                        <th class="number">Taxable Gain</th>
                    </tr>
                </thead>
                <tbody>${rows}</tbody>
            </table>
            <div class="note info">💡 <strong>Note:</strong> Proceeds = SA108 Box 21 (gross, before sale fees)</div>
            <h3 style="margin: 30px 0 10px 0; color: var(--color-text);">Tax Year Details</h3>
        `;
  },

  matchItem: (match) => {
    const ruleName =
      match.rule === "SameDay"
        ? "Same Day"
        : match.rule === "BedAndBreakfast"
          ? "Bed & Breakfast"
          : match.rule === "Section104"
            ? "Section 104"
            : escapeHtml(match.rule);
    const badgeClass =
      match.rule === "SameDay"
        ? "same-day"
        : match.rule === "BedAndBreakfast"
          ? "bed-breakfast"
          : "section-104";
    const costPerShare = parseFloat(match.allowable_cost) / parseFloat(match.quantity);

    return `
            <div class="match-item">
                <span class="badge ${badgeClass}">${ruleName}</span>
                <span style="color: #374151;">${escapeHtml(match.quantity)} shares @ ${formatCurrency(costPerShare)}</span>
                <span style="margin-left: auto; font-weight: 600; color: var(--color-text-dark);">${formatCurrency(match.allowable_cost)}</span>
            </div>
        `;
  },

  disposalCard: (disp, index) => {
    const gainLoss = disp.matches
      ? disp.matches.reduce((sum, m) => sum + parseFloat(m.gain_or_loss || 0), 0)
      : 0;
    const gainLossClass = gainLoss >= 0 ? "positive" : "negative";
    const gainLossLabel = gainLoss >= 0 ? "GAIN" : "LOSS";
    const resultBadgeClass = gainLoss >= 0 ? "gain" : "loss";
    const totalCost = disp.matches
      ? disp.matches.reduce((sum, m) => sum + parseFloat(m.allowable_cost || 0), 0)
      : 0;

    const matchesHtml =
      disp.matches && disp.matches.length > 0
        ? `
            <div class="match-section">
                <div class="match-title">Matching Rules Applied</div>
                ${disp.matches.map(ReportBuilder.matchItem).join("")}
            </div>
            <div class="disposal-metrics">
                <div class="metric-item">
                    <span class="metric-label">Gross Proceeds</span>
                    <span class="metric-value">${formatCurrency(disp.gross_proceeds)}</span>
                </div>
                <div class="metric-item">
                    <span class="metric-label">Net Proceeds</span>
                    <span class="metric-value">${formatCurrency(disp.proceeds)}</span>
                </div>
                <div class="metric-item">
                    <span class="metric-label">Allowable Cost</span>
                    <span class="metric-value">${formatCurrency(totalCost)}</span>
                </div>
                <div class="metric-item">
                    <span class="metric-label">Gain/Loss</span>
                    <span class="metric-value ${gainLossClass}">${formatCurrency(gainLoss)}</span>
                </div>
            </div>
        `
        : "";

    return `
            <div class="disposal-card">
                <div class="disposal-header">
                    <div class="disposal-title">
                        <div class="disposal-number">${index + 1}</div>
                        <div>
                            <div class="disposal-action">SELL ${escapeHtml(disp.quantity)} <span class="disposal-ticker">${escapeHtml(disp.ticker)}</span></div>
                            <div class="disposal-date">${escapeHtml(disp.date)}</div>
                        </div>
                    </div>
                    <div class="disposal-result">
                        <span class="badge ${resultBadgeClass}">${gainLossLabel}</span>
                        <span class="${gainLossClass}">${formatCurrency(Math.abs(gainLoss))}</span>
                    </div>
                </div>
                <div class="disposal-body">
                    ${matchesHtml}
                </div>
            </div>
        `;
  },

  taxYear: (ty, isSingleYear) => {
    const yearLabel = escapeHtml(ty.period || "Unknown");
    const disposalsHtml =
      ty.disposals && ty.disposals.length > 0
        ? ty.disposals.map((disp, idx) => ReportBuilder.disposalCard(disp, idx)).join("")
        : "";

    const taxBreakdownHtml =
      ty.tax_rates && ty.tax_rates.length > 0
        ? `
            <div class="tax-breakdown">
                <h4>Tax Breakdown</h4>
                ${ty.tax_rates
                  .map(
                    (rate) => `
                    <div class="tax-rate-item">
                        <span>${escapeHtml(rate.rate_name)}: ${formatCurrency(rate.taxable)} @ ${escapeHtml(rate.rate)}%</span>
                        <strong>${formatCurrency(rate.tax)}</strong>
                    </div>
                `,
                  )
                  .join("")}
            </div>
        `
        : "";

    const taxLiabilityHtml =
      isSingleYear && ty.tax_liability !== undefined
        ? `
            <div class="tax-liability-box">
                <div class="tax-liability-label">Total Tax Liability</div>
                <div class="tax-liability-value">${formatCurrency(ty.tax_liability || "0")}</div>
            </div>
            ${
              parseFloat(ty.tax_liability || 0) === 0 &&
              parseFloat(ty.taxable_gain || 0) === 0 &&
              parseFloat(ty.net_gain || 0) > 0
                ? `
                <div class="note warning" style="margin-top: 16px;">
                    <strong>ℹ️ Why is tax liability £0.00?</strong><br>
                    The net gain (${formatCurrency(ty.net_gain)}) is fully covered by the annual exemption (${formatCurrency(ty.exemption)}),
                    so there is no taxable gain and therefore no tax to pay.
                </div>
            `
                : ""
            }
        `
        : "";

    return `
            <div class="tax-year-section">
                <div class="tax-year-header">
                    <span class="tax-year-badge">${yearLabel}</span>
                    <span class="tax-year-title">Disposals & Summary</span>
                </div>
                <div>
                    ${disposalsHtml}
                    <div class="subsection-title">Summary</div>
                    <div class="summary-grid">
                        <div class="summary-item">
                            <span class="summary-label">Total Proceeds</span>
                            <span class="summary-value">${formatCurrency(ty.total_proceeds)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Allowable Costs</span>
                            <span class="summary-value">${formatCurrency(ty.total_cost)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Total Gains</span>
                            <span class="summary-value positive">${formatCurrency(ty.total_gain)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Total Losses</span>
                            <span class="summary-value negative">${formatCurrency(ty.total_loss)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Net Gain/Loss</span>
                            <span class="summary-value">${formatCurrency(ty.net_gain)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Annual Exemption</span>
                            <span class="summary-value">${formatCurrency(ty.exemption)}</span>
                        </div>
                        <div class="summary-item">
                            <span class="summary-label">Taxable Gain</span>
                            <span class="summary-value">${formatCurrency(ty.taxable_gain)}</span>
                        </div>
                    </div>
                    ${taxBreakdownHtml}
                    ${taxLiabilityHtml}
                </div>
            </div>
        `;
  },

  holdings: (holdings) => {
    if (!holdings || holdings.length === 0) return "";

    const rows = holdings
      .map((h) => {
        const totalCost = parseFloat(h.total_cost);
        const quantity = parseFloat(h.quantity);
        const avgCost = totalCost / quantity;
        return `
                <tr>
                    <td><strong>${escapeHtml(h.ticker)}</strong></td>
                    <td class="number">${escapeHtml(h.quantity)}</td>
                    <td class="number">${formatCurrency(totalCost)}</td>
                    <td class="number">${formatCurrency(avgCost)}</td>
                </tr>
            `;
      })
      .join("");

    return `
            <div class="holdings-section">
                <div class="holdings-header">
                    <span class="holdings-badge">Holdings</span>
                    <span class="holdings-title">Current Positions</span>
                </div>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Ticker</th>
                            <th class="number">Quantity</th>
                            <th class="number">Total Cost</th>
                            <th class="number">Avg Cost/Unit</th>
                        </tr>
                    </thead>
                    <tbody>${rows}</tbody>
                </table>
            </div>
        `;
  },
};

function generateReport() {
  if (!state.wasmReady) {
    showToast("WASM module not ready yet", "error");
    return;
  }

  if (state.isProcessing) {
    showToast("Already processing a request", "info");
    return;
  }

  const input = document.getElementById("input").value.trim();
  const taxYear = document.getElementById("taxYear").value;
  const yearCheckbox = document.getElementById("yearCheckbox");
  const output = document.getElementById("output");

  if (!input) {
    showToast("Please enter transaction DSL", "error");
    return;
  }

  try {
    state.isProcessing = true;
    showLoading(true);

    const year = yearCheckbox.checked && taxYear ? parseInt(taxYear, 10) : null;
    const result = window.wasmCalculateTax(input, year);
    const data = JSON.parse(result);

    const parts = [
      ReportBuilder.header(),
      data.tax_years?.length > 1 ? ReportBuilder.summaryTable(data.tax_years) : "",
      ...(data.tax_years?.map((ty) => ReportBuilder.taxYear(ty, data.tax_years.length === 1)) ||
        []),
      ReportBuilder.holdings(data.holdings),
    ];

    output.innerHTML = parts.filter(Boolean).join("");
    output.classList.add("html-mode");
    showToast("Report generated successfully!", "success");
  } catch (error) {
    output.classList.remove("html-mode");
    output.textContent = `Error: ${error.message || error}`;
    showToast(`Failed to generate report: ${error.message || error}`, "error");
    console.error("Report generation error:", error);
  } finally {
    state.isProcessing = false;
    showLoading(false);
  }
}

function calculateTax() {
  if (!state.wasmReady) {
    showToast("WASM module not ready yet", "error");
    return;
  }

  if (state.isProcessing) {
    showToast("Already processing a request", "info");
    return;
  }

  const input = document.getElementById("input").value.trim();
  const taxYear = document.getElementById("taxYear").value;
  const yearCheckbox = document.getElementById("yearCheckbox");
  const output = document.getElementById("output");

  if (!input) {
    showToast("Please enter transaction DSL", "error");
    return;
  }

  try {
    state.isProcessing = true;
    showLoading(true);

    const year = yearCheckbox.checked && taxYear ? parseInt(taxYear, 10) : null;
    const result = window.wasmCalculateTax(input, year);
    const parsed = JSON.parse(result);
    output.classList.remove("html-mode");
    output.textContent = JSON.stringify(parsed, null, 2);
    showToast("Tax calculated successfully!", "success");
  } catch (error) {
    output.classList.remove("html-mode");
    output.textContent = `Error: ${error.message || error}`;
    showToast(`Failed to calculate tax: ${error.message || error}`, "error");
    console.error("Tax calculation error:", error);
  } finally {
    state.isProcessing = false;
    showLoading(false);
  }
}

function parseTransactions() {
  if (!state.wasmReady) {
    showToast("WASM module not ready yet", "error");
    return;
  }

  if (state.isProcessing) {
    showToast("Already processing a request", "info");
    return;
  }

  const input = document.getElementById("input").value.trim();
  const output = document.getElementById("output");

  if (!input) {
    showToast("Please enter transaction DSL", "error");
    return;
  }

  try {
    state.isProcessing = true;
    showLoading(true);

    const result = window.wasmParseTransactions(input);
    const parsed = JSON.parse(result);
    output.classList.remove("html-mode");
    output.textContent = JSON.stringify(parsed, null, 2);
    showToast("Transactions parsed successfully!", "success");
  } catch (error) {
    output.classList.remove("html-mode");
    output.textContent = `Error: ${error.message || error}`;
    showToast(`Failed to parse: ${error.message || error}`, "error");
    console.error("Parse error:", error);
  } finally {
    state.isProcessing = false;
    showLoading(false);
  }
}

function validateDSL() {
  if (!state.wasmReady) {
    showToast("WASM module not ready yet", "error");
    return;
  }

  if (state.isProcessing) {
    showToast("Already processing a request", "info");
    return;
  }

  const input = document.getElementById("input").value.trim();
  const output = document.getElementById("output");

  if (!input) {
    showToast("Please enter transaction DSL", "error");
    return;
  }

  try {
    state.isProcessing = true;
    showLoading(true);

    const result = window.wasmValidateDSL(input);
    const parsed = JSON.parse(result);
    output.classList.remove("html-mode");
    output.textContent = JSON.stringify(parsed, null, 2);

    if (parsed.is_valid) {
      if (parsed.warnings && parsed.warnings.length > 0) {
        showToast(`Valid with ${parsed.warnings.length} warning(s)`, "info");
      } else {
        showToast("Validation passed!", "success");
      }
    } else {
      showToast(`Validation failed with ${parsed.errors.length} error(s)`, "error");
    }
  } catch (error) {
    output.classList.remove("html-mode");
    output.textContent = `Error: ${error.message || error}`;
    showToast(`Failed to validate: ${error.message || error}`, "error");
    console.error("Validation error:", error);
  } finally {
    state.isProcessing = false;
    showLoading(false);
  }
}

function loadExample(event) {
  event.preventDefault();
  const example = `# Comprehensive CGT Example - Significant Gains Leading to Tax Liability
# Demonstrates: Same Day, Bed & Breakfast, Section 104, FX conversion, TAX LIABILITY!

# === 2023 Tax Year (6 Apr 2023 - 5 Apr 2024) ===

# Initial US tech stock purchases (buying low)
2023-04-15 BUY AAPL 100 @ 120.00 USD FEES 10.00 USD
2023-05-20 BUY MSFT 50 @ 280.00 USD FEES 8.00 USD
2023-06-10 BUY GOOGL 30 @ 105.00 USD FEES 7.50 USD

# UK FTSE stock (using ISIN)
2023-07-01 BUY GB0007980591 500 @ 4.20 FEES 12.50

# Dividend income (no CGT impact)
2023-08-15 DIVIDEND AAPL 100 TOTAL 45.00 USD TAX 6.75 USD
2023-09-20 DIVIDEND MSFT 50 TOTAL 32.50 USD TAX 4.88 USD

# Highly profitable sales - buying at 120, selling at 195! (big gain)
2023-11-10 SELL AAPL 80 @ 195.00 USD FEES 10.00 USD

# Same-day rule (no gain since bought same day at higher price)
2023-12-15 SELL MSFT 20 @ 355.00 USD FEES 5.00 USD
2023-12-15 BUY MSFT 20 @ 360.00 USD FEES 5.00 USD

# === 2024 Tax Year (6 Apr 2024 - 5 Apr 2025) ===

# Bed & Breakfast rule (bought at 105, selling at 155)
2024-04-10 SELL GOOGL 15 @ 155.00 USD FEES 4.00 USD
2024-04-25 BUY GOOGL 15 @ 158.00 USD FEES 4.00 USD

# Stock split (4:1 split = multiply shares by 4, divide price by 4)
2024-05-01 SPLIT AAPL RATIO 4

# More purchases (post-split prices)
2024-06-15 BUY AAPL 200 @ 42.00 USD FEES 12.00 USD
2024-07-20 BUY GB0007980591 300 @ 4.80 FEES 10.00

# Capital return (reduces cost base, good for future gains!)
2024-08-10 CAPRETURN GB0007980591 800 TOTAL 160.00

# Major profitable sales (prices increased significantly - BIG GAINS!)
2024-09-05 SELL AAPL 200 @ 58.00 USD FEES 12.00 USD
2024-10-12 SELL GB0007980591 400 @ 6.20 FEES 15.00
2024-11-08 SELL GOOGL 25 @ 175.00 USD FEES 6.00 USD

# More dividends
2024-11-15 DIVIDEND AAPL 90 TOTAL 65.00 USD TAX 9.75 USD
2024-12-01 DIVIDEND GB0007980591 500 TOTAL 85.00 TAX 0

# Year-end position: Remaining holdings showing healthy unrealized gains!`;

  document.getElementById("input").value = example;
  document.getElementById("taxYear").value = "2024";
  showToast("Example loaded!", "info");
}

document.getElementById("helpToggle").addEventListener("click", toggleHelp);
document.getElementById("yearCheckbox").addEventListener("change", toggleYearInput);
document.getElementById("btnReport").addEventListener("click", generateReport);
document.getElementById("btnCalculate").addEventListener("click", calculateTax);
document.getElementById("btnParse").addEventListener("click", parseTransactions);
document.getElementById("btnValidate").addEventListener("click", validateDSL);
document.getElementById("exampleLink").addEventListener("click", loadExample);

initWasm();
