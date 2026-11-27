//! Validates the published JSON Schemas in `docs/` and every example in
//! `docs/examples/`.
//!
//! Valid examples must produce no errors. Each invalid example must be rejected
//! with an error at the exact location listed in `docs/examples/expected-errors.toml`,
//! so an example cannot pass by failing for the wrong reason.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::Validator;
use serde_json::Value;

fn docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Loads a schema, checks it against the JSON Schema 2020-12 meta-schema and
/// compiles it with format validation enabled.
fn load_schema(name: &str) -> Validator {
    let schema: Value = serde_json::from_str(&read(&docs_dir().join(name)))
        .unwrap_or_else(|e| panic!("{name} is not valid JSON: {e}"));
    if let Err(e) = jsonschema::meta::validate(&schema) {
        panic!("{name} is not a valid JSON Schema: {e}");
    }
    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap_or_else(|e| panic!("{name} does not compile: {e}"))
}

/// Parses a TOML or JSON example into the JSON data model.
fn load_instance(path: &Path) -> Value {
    let text = read(path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => toml::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is not valid TOML: {e}", path.display())),
        Some("json") => serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display())),
        _ => panic!("unexpected example file {}", path.display()),
    }
}

/// Lists the example files in a directory, sorted, and fails if there are none.
fn examples(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("cannot read an entry of {}: {e}", dir.display()))
                .path()
        })
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no examples in {}", dir.display());
    files
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_else(|| panic!("{} has no file name", path.display()))
        .to_string_lossy()
        .into_owned()
}

/// Returns `(location, message)` for every validation error.
fn errors(validator: &Validator, instance: &Value) -> Vec<(String, String)> {
    validator
        .iter_errors(instance)
        .map(|e| (e.instance_path().to_string(), e.to_string()))
        .collect()
}

fn check_examples(schema: &str, kind: &str) {
    let validator = load_schema(schema);
    let base = docs_dir().join("examples");

    for path in examples(&base.join(kind).join("valid")) {
        let found = errors(&validator, &load_instance(&path));
        assert!(
            found.is_empty(),
            "valid example {} was rejected: {found:#?}",
            path.display()
        );
    }

    let all_expected: BTreeMap<String, BTreeMap<String, String>> =
        toml::from_str(&read(&base.join("expected-errors.toml")))
            .unwrap_or_else(|e| panic!("expected-errors.toml is invalid: {e}"));
    let expected = &all_expected[kind];
    let invalid = examples(&base.join(kind).join("invalid"));

    let names: Vec<String> = invalid.iter().map(|p| file_name(p)).collect();
    let listed: Vec<String> = expected.keys().cloned().collect();
    assert_eq!(
        names, listed,
        "invalid {kind} examples and expected-errors.toml must list the same files"
    );

    for path in invalid {
        let location = &expected[&file_name(&path)];
        let found = errors(&validator, &load_instance(&path));
        assert!(
            found.iter().any(|(at, _)| at == location),
            "invalid example {} must fail at {location:?}, got: {found:#?}",
            path.display()
        );
    }
}

#[test]
fn manifest_schema_accepts_valid_and_rejects_invalid_examples() {
    check_examples("manifest.schema.json", "manifests");
}

#[test]
fn verdict_schema_accepts_valid_and_rejects_invalid_examples() {
    check_examples("verdict.schema.json", "verdicts");
}
