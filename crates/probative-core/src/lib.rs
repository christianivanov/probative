//! Core domain types shared by every Probative crate.
//!
//! - [`values`]: validated value types (commit hashes, SHA-256 digests,
//!   repository URLs, relative paths).
//! - [`domain`]: sanitizers, bug kinds, access types and harness languages.
//! - [`manifest`]: the `probative.toml` model and its validation rules.
//! - [`load_manifest`]: loading a manifest from disk against the allowlist and
//!   size limits, reading the PoC exactly once.

pub mod domain;
pub mod error;
mod load;
pub mod manifest;
pub mod values;

pub use domain::{Access, BugKind, FoundWith, Lang, NetRole, Sanitizer};
pub use error::{ManifestError, ValueError};
pub use load::{
    DEFAULT_MAX_POC_BYTES, LoadPolicy, LoadedManifest, MAX_MANIFEST_BYTES, PocBundle, load_manifest,
};
pub use manifest::{
    BuildSpec, ClaimedBug, Manifest, PocSpec, Provenance, Recipe, SCHEMA_VERSION, Target,
};
pub use values::{GitSha, RelPath, RepoUrl, Sha256};
