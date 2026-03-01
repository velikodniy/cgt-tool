// Capital Gains Tax Report Template
#let data = (
  generation_date: (year: 0, month: 0, day: 0),
  summary_rows: (),
  tax_years: (),
  has_holdings: false,
  holdings_rows: (),
  has_transactions: false,
  transaction_rows: (),
  has_asset_events: false,
  asset_event_rows: (),
) + sys.inputs

// Color palette - Minimal, professional grayscale
#let text-dark = rgb("#1F2328")
#let text-muted = rgb("#57606A")
#let border-color = rgb("#D0D7DE")
#let header-bg = rgb("#F6F8FA")
#let alt-row = rgb("#FAFBFC")
#let accent-blue = text-dark
#let gain-green = rgb("#2F6F3E")
#let loss-red = rgb("#8A2D2D")

// Page setup with Roboto font
// "tnum": 1 enables Tabular Numbers (essential for financial data alignment)
#set page(
  paper: "a4",
  margin: (x: 1.4cm, y: 1.0cm),
  footer: align(right)[
    #context [
      #text(size: 7pt, fill: text-muted)[Page #counter(page).display("1")]
    ]
  ],
)
#set text(font: "Roboto", size: 9pt, fill: text-dark, features: ("tnum": 1))
#set par(leading: 0.35em)

// Headings
#show heading.where(level: 1): it => {
  v(0.4em)
  block(below: 0.3em, stroke: (bottom: 0.5pt + border-color), width: 100%, inset: (bottom: 0.2em))[
    #text(size: 11.5pt, weight: "bold", fill: text-dark)[#it.body]
  ]
}
#show heading.where(level: 2): it => {
  v(0.4em)
  block(stroke: (bottom: 0.5pt + border-color), width: 100%, inset: (bottom: 0.2em))[
    #text(size: 10pt, weight: "bold", fill: text-dark)[#it.body]
  ]
  v(0.2em)
}

// Formatting helpers
#let pad2(n) = if n < 10 { "0" + str(n) } else { str(n) }

#let fmt-date(d) = {
  pad2(d.day) + "/" + pad2(d.month) + "/" + str(d.year)
}

#let fmt-tax-year(start-year) = {
  let end-year = start-year + 1
  let end-short = calc.rem(end-year, 100)
  str(start-year) + "/" + pad2(end-short)
}

#let fmt-fixed(value, digits: 2) = {
  let rounded = calc.round(value, digits: digits)
  if digits == 0 {
    str(rounded)
  } else {
    let text = str(rounded)
    if text.contains(".") {
      let parts = text.split(".")
      let int = parts.at(0)
      let frac = parts.at(1, default: "")
      let frac-fixed = if frac.len() < digits {
        frac + ("0" * (digits - frac.len()))
      } else if frac.len() > digits {
        frac.slice(0, digits)
      } else {
        frac
      }
      int + "." + frac-fixed
    } else {
      text + "." + ("0" * digits)
    }
  }
}

#let trim-zeros(text) = {
  if text.contains(".") {
    let parts = text.split(".")
    let frac = parts.at(1, default: "")
    let trimmed = frac.trim("0", at: end)
    if trimmed.len() == 0 {
      parts.at(0)
    } else {
      parts.at(0) + "." + trimmed
    }
  } else {
    text
  }
}

#let fmt-qty(value) = trim-zeros(fmt-fixed(value, digits: 6))

#let group-int(text) = {
  let len = text.len()
  if len <= 3 {
    text
  } else {
    let head = calc.rem(len, 3)
    let start = if head == 0 { 3 } else { head }
    let parts = (text.slice(0, start),)
    for i in range(start, len, step: 3) {
      parts.push(text.slice(i, i + 3))
    }
    parts.join(",")
  }
}

#let fmt-money(value) = {
  let sign = if value < 0 { sym.minus } else { "" }
  let abs = calc.abs(value)
  let fixed = fmt-fixed(abs, digits: 2)
  let parts = fixed.split(".")
  let int = group-int(parts.at(0))
  let frac = parts.at(1, default: "00")
  sign + "£" + int + "." + frac
}

#let fmt-currency(amount, code) = if code == "GBP" {
  fmt-money(amount)
} else {
  let sign = if amount < 0 { sym.minus } else { "" }
  let abs = calc.abs(amount)
  let fixed = fmt-fixed(abs, digits: 2)
  let parts = fixed.split(".")
  let int = group-int(parts.at(0))
  let frac = parts.at(1, default: "00")
  sign + code + " " + int + "." + frac
}

#let gain-label(value) = if value >= 0 { "GAIN" } else { "LOSS" }
#let gain-color(value) = if value >= 0 { gain-green } else { loss-red }

#let match-text(item) = {
  let qty = fmt-qty(item.quantity)
  if item.rule == "SAME_DAY" {
    "Same Day: " + qty + " shares"
  } else if item.rule == "BED_AND_BREAKFAST" {
    if item.acquisition_date != none {
      "B&B: " + qty + " shares from " + fmt-date(item.acquisition_date)
    } else {
      "B&B: " + qty + " shares"
    }
  } else {
    let unit = if item.quantity != 0 { item.allowable_cost / item.quantity } else { 0 }
    "Section 104: " + qty + " shares @ " + fmt-money(unit)
  }
}

#let tax-year-header(label) = {
  block(stroke: (bottom: 0.5pt + border-color), width: 100%, inset: (bottom: 0.2em))[
    #text(size: 9.5pt, weight: "bold", fill: text-dark)[#label]
  ]
  v(0.15em)
}

// Header
#grid(
  columns: (1fr, auto),
  align: (left, right),
  [
    #text(size: 15pt, weight: "bold", fill: text-dark)[Capital Gains Tax Report]
  ],
  [
    #text(size: 8pt, fill: text-muted)[Generated: #fmt-date(data.generation_date)]
  ]
)
#v(0.4em)

// Summary Section
= Summary

#table(
  columns: (1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
  align: (left, right, right, right, right, right, right, right),
  stroke: (x: none, y: 0.5pt + border-color),
  inset: 3pt,
  fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row } else { none },
  table.header(
    text(size: 8pt)[*Tax Year*],
    text(size: 8pt)[*Disposals*#super("1")],
    stack(
      spacing: 1pt,
      text(size: 8pt)[*Gains*#super("2")],
      text(size: 7pt)[*(after losses)*],
    ),
    stack(
      spacing: 1pt,
      text(size: 8pt)[*Gains*#super("2")],
      text(size: 7pt)[*(before losses)*],
    ),
    text(size: 8pt)[*Losses*#super("2")],
    text(size: 8pt)[*Proceeds*#super("3")],
    text(size: 8pt)[*Exemption*],
    text(size: 8pt)[*Taxable gain*],
  ),
  ..data.summary_rows
    .map(row => (
      fmt-tax-year(row.start_year),
      str(row.disposal_count),
      fmt-money(row.net_gain),
      fmt-money(row.total_gain),
      fmt-money(row.total_loss),
      fmt-money(row.gross_proceeds),
      fmt-money(row.exemption),
      fmt-money(row.taxable),
    ))
    .flatten()
)
#v(0.2em)
#block[
  #set text(size: 6pt, fill: text-muted)
  *Notes:*
  #enum(numbering: "1.",
    [Disposals groups same-day disposals into a single transaction (CG51560) and may differ from raw SELL transactions],
    [Gains and losses are net per disposal after matching rules (CG51560) and align with SA108 Boxes 23 and 24],
    [Proceeds = SA108 Box 21 (gross, before sale fees)],
  )
]

// Disposal Details Section
= Disposal Details

#for year in data.tax_years [
  #tax-year-header("Tax Year " + fmt-tax-year(year.start_year))

  #if year.disposals.len() == 0 [
    #pad(left: 0.5em)[_No disposals recorded for this period._]
  ] else [
    #for (i, disposal) in year.disposals.enumerate() [
      #let is-last = i == year.disposals.len() - 1
      #let result-color = gain-color(disposal.total_gain)
      #let gain-text = gain-label(disposal.total_gain)

      #box(
        stroke: none,
        width: 100%,
        inset: (x: 4pt, y: 3pt)
      )[
        #grid(
          columns: (1fr, auto),
          align: (left, right),
          [
            #text(weight: "bold")[#(i + 1). #disposal.ticker]
            #h(0.6em)
            #text(fill: text-muted)[#fmt-qty(disposal.quantity) shares]
            #h(1em)
            #text(fill: text-muted)[Sold #fmt-date(disposal.date)]
          ],
          [
            #text(weight: "bold", fill: result-color)[#gain-text #fmt-money(calc.abs(disposal.total_gain))]
          ]
        )
        #v(0.25em)

        #grid(
          columns: (1fr, 1.35fr),
          gutter: 6pt,
          [
            #text(size: 7pt, weight: "bold", fill: text-muted)[COST BASIS]
            #v(0.05em)
            #block[
              #set text(size: 8pt)
              #for match in disposal.matches [
                #grid(
                  columns: (6pt, 1fr),
                  gutter: 0.4em,
                  align: (left, top),
                  [#text(fill: text-muted)[-]],
                  [#match-text(match)]
                )
              ]
            ]
          ],
          [
            #set align(right)
            #set text(size: 7.5pt)
            #text(size: 7pt, weight: "bold", fill: text-muted)[CALCULATION]
            #v(0.15em)

            #grid(
              columns: (auto, auto),
              gutter: 0.3em,
              align: (right, right),
              [Gross Proceeds:], [#fmt-qty(disposal.quantity) #sym.times #fmt-money(disposal.gross_proceeds / disposal.quantity) = #fmt-money(disposal.gross_proceeds)],
              ..if (disposal.gross_proceeds - disposal.proceeds) > 0 {
                ([Net Proceeds:], [#fmt-money(disposal.gross_proceeds) #sym.minus #fmt-money(disposal.gross_proceeds - disposal.proceeds) = #fmt-money(disposal.proceeds)])
              } else { () },
              [Cost:], [#fmt-money(disposal.total_cost)],
              line(length: 100%, stroke: 0.35pt + border-color), line(length: 100%, stroke: 0.35pt + border-color),
              [*Result:*], [#text(weight: "bold", fill: result-color)[#fmt-money(disposal.total_gain)]]
            )
          ]
        )
      ]
      #if not is-last [
        #v(0.1em)
        #line(length: 100%, stroke: 0.35pt + border-color)
        #v(0.1em)
      ]
    ]
  ]
]

// Holdings Section
= Holdings

#if not data.has_holdings [
  _No remaining holdings._
] else [
  #table(
    columns: (2fr, 1fr, 1fr),
    align: (left, right, right),
    stroke: (x: none, y: 0.5pt + border-color),
    inset: 4pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row } else { none },
    table.header(
      [*Ticker*], [*Quantity*], [*Avg Cost*],
    ),
    ..data.holdings_rows
      .map(row => (
        row.ticker,
        fmt-qty(row.quantity),
        fmt-money(row.total_cost / row.quantity),
      ))
      .flatten()
  )
]

// Transactions Section
= Transactions

#if not data.has_transactions [
  _No transactions._
] else [
  #table(
    columns: (auto, auto, 1fr, auto, auto, auto),
    align: (left, left, left, right, right, right),
    stroke: (x: none, y: 0.5pt + border-color),
    inset: 3pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row } else { none },
    table.header(
      [*Date*], [*Type*], [*Ticker*], [*Qty*], [*Price*], [*Fees*],
    ),
    ..data.transaction_rows
      .map(row => (
        fmt-date(row.date),
        row.type,
        row.ticker,
        fmt-qty(row.quantity),
        fmt-currency(row.price.amount, row.price.currency),
        fmt-currency(row.fees.amount, row.fees.currency),
      ))
      .flatten()
  )
]

// Asset Events Section (if any)
#if data.has_asset_events [
  == Asset Events

  #table(
    columns: (auto, auto, 1fr, auto, auto),
    align: (left, left, left, right, right),
    stroke: (x: none, y: 0.5pt + border-color),
    inset: 3pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row } else { none },
    table.header(
      [*Date*], [*Type*], [*Ticker*], [*Amount*], [*Value*],
    ),
    ..data.asset_event_rows
      .map(row => (
        fmt-date(row.date),
        row.type,
        row.ticker,
        if row.amount == none { "-" } else { fmt-qty(row.amount) },
        if row.value == none {
          "-"
        } else {
          fmt-currency(row.value.amount, row.value.currency)
        },
      ))
      .flatten()
  )
]
