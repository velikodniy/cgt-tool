// Capital Gains Tax Report Template
#let data = sys.inputs

// Color palette - soft, professional tones
#let header-bg = rgb("#F8F9FA")
#let alt-row = rgb("#FAFAFA")
#let gain-green = rgb("#2E7D32")
#let loss-red = rgb("#C62828")
#let accent-blue = rgb("#1565C0")
#let border-color = rgb("#DEE2E6")

// Page setup with Roboto font (Apache 2.0 license)
#set page(paper: "a4", margin: (x: 2cm, y: 1.5cm))
#set text(font: "Roboto", size: 9pt)
#set par(leading: 0.5em)

#show heading.where(level: 1): it => {
  v(0.6em)
  text(size: 11pt, weight: "bold", fill: accent-blue)[#it.body]
  v(0.2em)
}
#show heading.where(level: 2): it => {
  v(0.4em)
  text(size: 10pt, weight: "bold")[#it.body]
  v(0.1em)
}

// Header
#align(center)[
  #text(size: 16pt, weight: "bold", fill: accent-blue)[Capital Gains Tax Report]
  #v(0.15em)
  #text(size: 8pt, fill: luma(100))[Generated: #data.generation_date]
]
#v(0.6em)

// Summary Section
= Summary

#table(
  columns: (1.2fr, 1fr, 1fr, 1fr, 1fr),
  align: (left, right, right, right, right),
  stroke: (x: none, y: 0.5pt + border-color),
  inset: 6pt,
  fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row },
  table.header(
    [*Tax Year*], [*Gain/Loss*], [*Proceeds*], [*Exemption*], [*Taxable*],
  ),
  ..data.summary_rows.flatten()
)
#text(size: 7pt, fill: luma(100))[Note: Proceeds = SA108 Box 21 (gross, before sale fees)]

// Disposal Details Section
= Disposal Details

#for year in data.tax_years [
  == Tax Year #year.period

  #if year.disposals.len() == 0 [
    _No disposals._
  ] else [
    #for (i, disposal) in year.disposals.enumerate() [
      #let is-gain = disposal.gain_type == "GAIN"
      #let result-color = if is-gain { gain-green } else { loss-red }

      #box(
        fill: if calc.odd(i) { alt-row } else { white },
        inset: (x: 8pt, y: 6pt),
        radius: 2pt,
        width: 100%,
      )[
        *#(i + 1). SELL #disposal.quantity #disposal.ticker on #disposal.date* — #text(fill: result-color, weight: "bold")[#disposal.gain_type #disposal.gain_amount]
        #v(0.2em)
        #pad(left: 1em)[
          #for match in disposal.matches [
            #text(fill: luma(80))[•] #match.description #linebreak()
          ]
          #v(0.1em)
          Gross Proceeds: #disposal.gross_proceeds_calc #linebreak()
          #if disposal.has_fees [
            Net Proceeds: #disposal.net_proceeds_calc #linebreak()
          ]
          Cost: #disposal.total_cost #linebreak()
          Result: #text(fill: result-color, weight: "bold")[#disposal.result]
        ]
      ]
      #v(0.2em)
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
    inset: 6pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row },
    table.header(
      [*Ticker*], [*Quantity*], [*Avg Cost*],
    ),
    ..data.holdings_rows.flatten()
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
    inset: 6pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row },
    table.header(
      [*Date*], [*Type*], [*Ticker*], [*Qty*], [*Price*], [*Fees*],
    ),
    ..data.transaction_rows.flatten()
  )
]

// Asset Events Section (if any)
#if data.has_asset_events [
  == Asset Events

  #table(
    columns: (auto, auto, 1fr, auto, auto),
    align: (left, left, left, right, right),
    stroke: (x: none, y: 0.5pt + border-color),
    inset: 6pt,
    fill: (_, row) => if row == 0 { header-bg } else if calc.odd(row) { alt-row },
    table.header(
      [*Date*], [*Type*], [*Ticker*], [*Amount*], [*Value*],
    ),
    ..data.asset_event_rows.flatten()
  )
]
