//! Cross-checks the Rust manifest parser against the published examples: every
//! valid example must parse, and every invalid one must be rejected, so the
//! schema and the parser cannot drift apart.

use std::fs;
use std::path::{Path, PathBuf};

use probative_core::Manifest;

fn examples(kind: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/examples/manifests")
        .join(kind);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("cannot read an entry of {}: {e}", dir.display()))
                .path()
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no examples in {}", dir.display());
    files
}

#[test]
fn every_valid_example_manifest_parses() {
    for path in examples("valid") {
        let text = fs::read_to_string(&path).unwrap();
        if let Err(e) = Manifest::from_toml_str(&text) {
            panic!("{} was rejected: {e}", path.display());
        }
    }
}

#[test]
fn every_invalid_example_manifest_is_rejected() {
    for path in examples("invalid") {
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            Manifest::from_toml_str(&text).is_err(),
            "{} was accepted",
            path.display()
        );
    }
}
