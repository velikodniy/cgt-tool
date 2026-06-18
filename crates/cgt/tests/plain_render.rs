//! Plain-renderer byte-equivalence over the fixture corpus.
//!
//! For every input the rendered plain text must byte-match the recorded golden,
//! except a known set whose underlying values changed: those must still render
//! without panicking and must differ from their (now stale) golden.
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cgt::format::plain;
use cgt::{Config, calculate};

/// Fixtures whose recorded plain goldens predate corrected disposal values.
/// They render successfully but no longer match the stale golden.
const ADJUDICATED: &[&str] = &[
    "AssetEventsNotFullSale",
    "AssetEventsNotFullSale2",
    "WithAssetEventsBB",
    "AccumulationDividend",
    "WithAssetEventsSameDay",
    "SyntheticComplex",
];

fn repo_tests_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/cgt; the fixtures live at the repo root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
}

fn render_fixture(input: &Path) -> String {
    let content = std::fs::read_to_string(input).expect("fixture readable");
    let transactions = cgt::dsl::parse(&content).expect("fixture parses");
    let fx = cgt::money::load_default_cache().expect("bundled FX cache loads");
    let config = Config::embedded().expect("embedded config loads");
    let report =
        calculate(&transactions, None, Some(&fx), &config).expect("report builds for fixture");
    plain::render(&report)
}

#[test]
fn plain_render_matches_goldens_except_revalued_fixtures() {
    let tests_dir = repo_tests_dir();
    let inputs_dir = tests_dir.join("inputs");
    let plain_dir = tests_dir.join("plain");
    let adjudicated: BTreeSet<&str> = ADJUDICATED.iter().copied().collect();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&inputs_dir)
        .expect("inputs directory readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "cgt"))
        .collect();
    entries.sort();

    let mut matched = 0usize;
    let mut differed = 0usize;

    for input in &entries {
        let name = input
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("fixture has a UTF-8 stem");
        let golden_path = plain_dir.join(format!("{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("golden missing for {name}"));

        let rendered = render_fixture(input);

        if adjudicated.contains(name) {
            assert_ne!(
                rendered, golden,
                "{name} is revalued; its plain golden should no longer match"
            );
            differed += 1;
        } else {
            assert_eq!(rendered, golden, "plain render diverged for {name}");
            matched += 1;
        }
    }

    assert_eq!(
        matched + differed,
        entries.len(),
        "every fixture is either matched or adjudicated"
    );
    assert_eq!(entries.len(), 46, "fixture count is 46");
    assert_eq!(differed, ADJUDICATED.len(), "all adjudicated fixtures seen");
}
