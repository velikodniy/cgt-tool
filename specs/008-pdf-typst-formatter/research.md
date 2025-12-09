# Research: PDF Typst Formatter

## Decision: Typst Library Choice

**Decision**: Use `typst-as-lib` crate (v0.15+) with `typst-pdf` for PDF export

**Rationale**:

- `typst-as-lib` provides a simple builder API for compiling Typst documents
- Actively maintained, 92k+ downloads, MIT licensed
- Works with embedded fonts via `typst-kit-embed-fonts` feature
- Pure Rust implementation, no external dependencies
- `typst-pdf` is the official PDF export crate from the Typst team

**Alternatives considered**:

- `typst` crate directly: Requires implementing the `World` trait manually - more complex
- `papermake`: Early stage, API subject to change
- External Typst CLI: Rejected per user requirement (no external tools)

## Decision: Font Embedding Strategy

**Decision**: Use `typst-kit-embed-fonts` feature to embed fonts from `typst-assets` at compile time

**Rationale**:

- Ensures reproducible PDF output across all platforms
- No runtime font discovery needed
- Includes professional fonts suitable for tax documents
- Increases binary size slightly but eliminates external dependencies

**Alternatives considered**:

- System fonts via `typst-kit-fonts`: Would require fonts installed on user's system
- Custom embedded fonts: Unnecessary complexity, typst-assets fonts are sufficient

## Decision: Template Approach

**Decision**: Embed Typst template as a string constant using `include_str!()` macro

**Rationale**:

- Single binary distribution, no external template files
- Template can be version-controlled with the code
- Compile-time inclusion ensures template is always available

**Alternatives considered**:

- External template file: Would require file resolution, distribution complexity
- Runtime template loading: Unnecessary for fixed report format

## Decision: Data Passing to Template

**Decision**: Generate Typst markup programmatically in Rust, then compile

**Rationale**:

- Full control over data formatting (currency, dates)
- Can reuse existing formatting functions from `cgt-formatter-plain`
- Avoids complex data serialization to Typst's input format
- Template defines structure, Rust code fills in values

**Alternatives considered**:

- JSON input to template: Typst's JSON handling is limited
- Direct variable binding: `typst-as-lib` supports this but string interpolation is simpler

## Decision: Output Handling

**Decision**: Return `Vec<u8>` from formatter, CLI handles file writing

**Rationale**:

- Consistent with formatter responsibility (format data, not I/O)
- Allows CLI to determine output path based on `--output` flag
- Enables testing without filesystem side effects

**Alternatives considered**:

- Formatter writes file directly: Violates separation of concerns

## Technical Notes: typst-as-lib API

```rust
// Basic usage pattern
let engine = TypstEngine::builder()
    .main_file(template_content)
    .fonts(embedded_fonts)
    .build();

let doc = engine
    .compile()  // or compile_with_input() for variables
    .output?;

let pdf_bytes = typst_pdf::pdf(&doc, &PdfOptions::default())?;
```

## Technical Notes: Typst Template Structure

Typst uses a simple, readable markup:

```typst
#set page(paper: "a4", margin: 2cm)
#set text(font: "Linux Libertine", size: 11pt)

= Capital Gains Tax Report
== Tax Year 2023/2024

#table(
  columns: (auto, auto, auto),
  [Item], [Value], [Notes],
  [Total Gain], [£1,234], [],
)
```

Key features for our use:

- `#table()` for summary and transaction tables
- `#heading()` / `=` syntax for sections
- `#text()` for formatting control
- `#page()` for A4 sizing and margins

## Sources

- [typst-as-lib on crates.io](https://crates.io/crates/typst-as-lib)
- [typst-as-lib on lib.rs](https://lib.rs/crates/typst-as-lib)
- [typst-as-lib GitHub](https://github.com/Relacibo/typst-as-lib)
- [typst-pdf on crates.io](https://crates.io/crates/typst-pdf)
- [Typst Open Source](https://typst.app/open-source/)
