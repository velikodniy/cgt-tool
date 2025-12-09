# Quickstart: PDF Typst Formatter

## Overview

This feature adds PDF output support to the CGT CLI using embedded Typst. Users can generate professional tax reports in PDF format without installing any external tools.

## Usage

### Basic PDF Generation

```bash
cgt-cli report transactions.cgt --year 2023 --format pdf
```

This generates `transactions.pdf` in the current directory.

### Specify Output Path

```bash
cgt-cli report transactions.cgt --year 2023 --format pdf --output report.pdf
```

### Example Output

The generated PDF contains:

1. **Header** - Report title and generation date
2. **Summary Table** - Tax year, gains, proceeds, exemption, taxable amount
3. **Disposal Details** - Each sale with matching rule breakdown
4. **Holdings** - Remaining positions after the tax year
5. **Transactions** - Chronological list of all buys/sells

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
typst-as-lib = { version = "0.15", features = ["typst-kit-embed-fonts"] }
typst-pdf = "0.14"
```

## Implementation Pattern

### Crate Structure

```text
crates/cgt-formatter-pdf/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Public API

```rust
use cgt_core::{CgtError, TaxReport, Transaction};

/// Generate a PDF report from tax data.
///
/// # Arguments
/// * `report` - The calculated tax report
/// * `transactions` - Original transactions for display
///
/// # Returns
/// PDF file contents as bytes, or error if generation fails.
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<Vec<u8>, CgtError>;
```

### Basic Implementation Skeleton

```rust
use typst_as_lib::TypstEngine;

pub fn format(report: &TaxReport, transactions: &[Transaction]) -> Result<Vec<u8>, CgtError> {
    // 1. Generate Typst markup from data
    let typst_content = generate_typst_markup(report, transactions)?;

    // 2. Compile with embedded fonts
    let engine = TypstEngine::builder().main_file(typst_content).build();

    let doc = engine
        .compile()
        .output
        .map_err(|e| CgtError::PdfGeneration(e.to_string()))?;

    // 3. Export to PDF
    let pdf = typst_pdf::pdf(&doc, &Default::default())
        .map_err(|e| CgtError::PdfGeneration(e.to_string()))?;

    Ok(pdf)
}
```

### Typst Template Example

```typst
#set page(paper: "a4", margin: 2cm)
#set text(font: "Linux Libertine", size: 11pt)

#align(center)[
  #text(size: 18pt, weight: "bold")[Capital Gains Tax Report]
  #v(0.5em)
  Tax Year: 2023/2024
  #v(0.3em)
  #text(size: 9pt, fill: gray)[Generated: 09/12/2024]
]

#v(1em)

= Summary

#table(
  columns: (1fr, 1fr),
  stroke: 0.5pt,
  [*Metric*], [*Value*],
  [Total Gain/Loss], [£1,234],
  [Total Proceeds], [£10,000],
  [Annual Exemption], [£6,000],
  [Taxable Gain], [£0],
)

= Disposals

// ... disposal details ...

= Holdings

// ... holdings table ...

= Transactions

// ... transactions table ...
```

## CLI Integration

In `crates/cgt-cli/src/main.rs`:

```rust
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Plain,
    Json,
    Pdf,  // NEW
}

// In report command handler:
match format {
    OutputFormat::Plain => {
        print!("{}", cgt_formatter_plain::format(&report, &transactions)?);
    }
    OutputFormat::Json => {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    OutputFormat::Pdf => {
        let pdf = cgt_formatter_pdf::format(&report, &transactions)?;
        let output_path = output.unwrap_or_else(|| {
            input.with_extension("pdf")
        });
        std::fs::write(&output_path, pdf)?;
        println!("PDF written to {}", output_path.display());
    }
}
```

## Testing

### Unit Tests

```rust
#[test]
fn test_pdf_generation_simple() {
    let report = create_test_report();
    let transactions = create_test_transactions();

    let pdf = format(&report, &transactions).expect("PDF generation failed");

    // Verify PDF header
    assert!(pdf.starts_with(b"%PDF"));
}
```

### Integration Tests

```rust
#[test]
fn test_all_inputs_generate_valid_pdfs() {
    for (name, year) in TEST_CASES {
        let input = format!("../../tests/inputs/{}.cgt", name);
        let cmd = Command::cargo_bin("cgt-cli").unwrap();
        let output = cmd
            .arg("report")
            .arg("--year")
            .arg(year.to_string())
            .arg("--format")
            .arg("pdf")
            .arg(&input)
            .output()
            .expect("CLI failed");

        assert!(output.status.success(), "Failed for {}", name);
        // Verify PDF file was created
    }
}
```

## Error Handling

New error variant in `cgt-core`:

```rust
#[derive(Error, Debug)]
pub enum CgtError {
    // ... existing variants ...
    #[error("PDF generation failed: {0}")]
    PdfGeneration(String),
}
```
