//! Errors for value validation and manifest loading.

use std::path::PathBuf;

use crate::values::{RepoUrl, Sha256};

/// A value that does not have the required format.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValueError {
    /// Not a full lowercase 40-character commit hash.
    #[error("{0:?} is not a full commit hash (40 lowercase hex characters)")]
    GitSha(String),
    /// Not a 64-character lowercase SHA-256 digest.
    #[error("{0:?} is not a SHA-256 digest (64 lowercase hex characters)")]
    Sha256(String),
    /// Not a normalized https repository URL.
    #[error("repository URL {value:?} {reason}")]
    RepoUrl {
        /// The offending value, shortened.
        value: String,
        /// Why it was rejected.
        reason: &'static str,
    },
    /// Not a safe relative path.
    #[error("path {value:?} {reason}")]
    RelPath {
        /// The offending value, shortened.
        value: String,
        /// Why it was rejected.
        reason: &'static str,
    },
    /// Not a known bug kind.
    #[error("{0:?} is not a known bug kind")]
    BugKind(String),
}

/// Why a manifest could not be loaded.
///
/// Errors never include PoC bytes, and long values are shortened.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// Reading a file failed.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The manifest file is larger than allowed.
    #[error("the manifest is larger than {max} bytes")]
    ManifestTooLarge {
        /// The limit, in bytes.
        max: u64,
    },
    /// The manifest is not UTF-8.
    #[error("the manifest is not valid UTF-8")]
    NotUtf8,
    /// The manifest is not valid TOML or has an unknown key or a wrong type.
    #[error("invalid manifest: {0}")]
    Toml(#[from] toml::de::Error),
    /// `schema_version` is not supported.
    #[error("unsupported schema_version {found:?}; this version of Probative reads \"0.1\"")]
    SchemaVersion {
        /// The version found in the manifest, shortened.
        found: String,
    },
    /// A field has an invalid value.
    #[error("{field}: {reason}")]
    Invalid {
        /// The field, for example `target.commit`.
        field: &'static str,
        /// Why the value was rejected.
        reason: String,
    },
    /// A field required by the recipe or PoC type is missing.
    #[error("{field} is required {because}")]
    Missing {
        /// The missing field.
        field: &'static str,
        /// The condition that requires it, for example `with recipe = "arvo"`.
        because: &'static str,
    },
    /// A field that does not apply to the recipe or PoC type is present.
    #[error("{field} is not allowed {because}")]
    Forbidden {
        /// The forbidden field.
        field: &'static str,
        /// The condition that forbids it.
        because: &'static str,
    },
    /// The repository is not in the allowlist. A policy error.
    #[error("repository {0} is not in the allowlist")]
    RepoNotAllowed(RepoUrl),
    /// The PoC resolves outside the manifest directory, for example through a symlink.
    #[error("the PoC file {path} resolves outside the manifest directory")]
    PocOutsideManifestDir {
        /// The PoC path as written in the manifest.
        path: PathBuf,
    },
    /// The PoC is not a regular file.
    #[error("the PoC {path} is not a regular file")]
    PocNotAFile {
        /// The PoC path as written in the manifest.
        path: PathBuf,
    },
    /// The PoC is larger than allowed. A policy error.
    #[error("the PoC is larger than the limit of {max} bytes")]
    PocTooLarge {
        /// The limit, in bytes.
        max: u64,
    },
    /// The PoC does not match the SHA-256 declared in the manifest.
    #[error("the PoC's SHA-256 is {actual}, but the manifest declares {expected}")]
    PocHashMismatch {
        /// The hash declared in the manifest.
        expected: Sha256,
        /// The hash of the file.
        actual: Sha256,
    },
}

impl ManifestError {
    /// Whether this error is a policy refusal (CLI exit code 3) rather than an
    /// invalid manifest or an I/O failure.
    pub fn is_policy_violation(&self) -> bool {
        matches!(
            self,
            Self::RepoNotAllowed(_) | Self::PocTooLarge { .. } | Self::ManifestTooLarge { .. }
        )
    }
}
