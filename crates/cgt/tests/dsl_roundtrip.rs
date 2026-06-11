//! DSL roundtrip over every golden fixture: parse -> serialize -> reparse
//! must equal the original parse. Fixtures are discovered, never listed.

use std::fs;
use std::path::PathBuf;

fn fixture_paths() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/inputs");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "cgt"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

#[test]
fn fixtures_exist() {
    assert!(
        fixture_paths().len() >= 40,
        "fixture auto-discovery found too few .cgt files"
    );
}

#[test]
fn parse_serialize_reparse_roundtrip() {
    let mut failures: Vec<String> = Vec::new();
    for path in fixture_paths() {
        let Ok(content) = fs::read_to_string(&path) else {
            failures.push(format!("cannot read {}", path.display()));
            continue;
        };
        let Ok(parsed) = cgt::dsl::parse(&content) else {
            // Fixtures that intentionally fail to parse are out of scope here.
            continue;
        };
        let serialized = cgt::dsl::serialize(&parsed);
        match cgt::dsl::parse(&serialized) {
            Ok(reparsed) if reparsed == parsed => {}
            Ok(_) => failures.push(format!("roundtrip mismatch: {}", path.display())),
            Err(e) => failures.push(format!("reparse failed for {}: {e}", path.display())),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
