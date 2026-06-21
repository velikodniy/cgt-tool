//! Plain-renderer byte-equivalence over the fixture corpus.
//!
//! For every input the rendered plain text must byte-match the recorded golden.
#![allow(clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use cgt::format::plain;
use cgt::{Config, calculate};

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
fn plain_render_matches_goldens() {
    let tests_dir = repo_tests_dir();
    let inputs_dir = tests_dir.join("inputs");
    let plain_dir = tests_dir.join("plain");

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&inputs_dir)
        .expect("inputs directory readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "cgt"))
        .collect();
    entries.sort();

    for input in &entries {
        let name = input
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("fixture has a UTF-8 stem");
        let golden_path = plain_dir.join(format!("{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("golden missing for {name}"));

        let rendered = render_fixture(input);
        assert_eq!(rendered, golden, "plain render diverged for {name}");
    }

    assert_eq!(entries.len(), 48, "fixture count is 48");
}
