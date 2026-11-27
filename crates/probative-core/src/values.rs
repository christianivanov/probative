//! Validated value types: commit hashes, SHA-256 digests, repository URLs and
//! relative paths.
//!
//! Each type can only be built through a constructor that checks its format,
//! so holding one means the value is valid.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256 as Sha256Hasher};

use crate::error::ValueError;

fn is_lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Shortens a value for an error message, so errors never echo huge inputs.
fn excerpt(value: &str) -> String {
    const MAX: usize = 80;
    if value.chars().count() <= MAX {
        value.to_owned()
    } else {
        let short: String = value.chars().take(MAX).collect();
        format!("{short}…")
    }
}

/// A full Git commit hash: 40 lowercase hexadecimal characters.
///
/// Short hashes, branches and tags are rejected so that the code a manifest
/// refers to cannot change.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GitSha(String);

impl GitSha {
    /// Validates a full, lowercase SHA-1 commit hash.
    pub fn new(value: impl Into<String>) -> Result<Self, ValueError> {
        let value = value.into();
        if is_lower_hex(&value, 40) {
            Ok(Self(value))
        } else {
            Err(ValueError::GitSha(excerpt(&value)))
        }
    }

    /// The hash as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A SHA-256 digest as 64 lowercase hexadecimal characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256(String);

impl Sha256 {
    /// Validates a SHA-256 digest written as 64 lowercase hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ValueError> {
        let value = value.into();
        if is_lower_hex(&value, 64) {
            Ok(Self(value))
        } else {
            Err(ValueError::Sha256(excerpt(&value)))
        }
    }

    /// Computes the SHA-256 digest of `bytes`.
    pub fn of(bytes: &[u8]) -> Self {
        let digest = Sha256Hasher::digest(bytes);
        let mut hex = String::with_capacity(64);
        for byte in digest.iter() {
            hex.push(char::from(HEX[usize::from(byte >> 4)]));
            hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self(hex)
    }

    /// The digest as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

/// A normalized `https` repository URL.
///
/// Normalized means: lowercase host, no credentials, query, fragment, trailing
/// slash or `.git` suffix, and no `.` or `..` path segments. Two URLs for the
/// same repository therefore compare equal, which the allowlist relies on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoUrl(String);

impl RepoUrl {
    /// Maximum length of a repository URL, in bytes.
    pub const MAX_LEN: usize = 2048;

    /// Validates a normalized `https` repository URL.
    pub fn new(value: impl Into<String>) -> Result<Self, ValueError> {
        let value = value.into();
        match Self::check(&value) {
            Ok(()) => Ok(Self(value)),
            Err(reason) => Err(ValueError::RepoUrl {
                value: excerpt(&value),
                reason,
            }),
        }
    }

    fn check(value: &str) -> Result<(), &'static str> {
        if value.len() > Self::MAX_LEN {
            return Err("is too long");
        }
        let rest = value
            .strip_prefix("https://")
            .ok_or("must start with https://")?;
        if rest.contains(['@', '?', '#']) || rest.chars().any(char::is_whitespace) {
            return Err("must not contain credentials, a query, a fragment or whitespace");
        }
        let (authority, path) = rest
            .split_once('/')
            .ok_or("must include a repository path")?;
        let (host, port) = match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        };
        if let Some(port) = port
            && (port.is_empty() || port.len() > 5 || !port.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err("has an invalid port");
        }
        let labels: Vec<&str> = host.split('.').collect();
        let label_ok = |label: &&str| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if labels.len() < 2 || !labels.iter().all(label_ok) {
            return Err("must have a lowercase host name such as github.com");
        }
        if path.ends_with('/') {
            return Err("must not end with a slash");
        }
        if path.ends_with(".git") {
            return Err("must not end with .git");
        }
        for segment in path.split('/') {
            let allowed =
                |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'~' | b'-');
            if segment.is_empty() || !segment.bytes().all(allowed) {
                return Err("has an empty or invalid path segment");
            }
            if segment == "." || segment == ".." {
                return Err("must not contain . or .. path segments");
            }
        }
        Ok(())
    }

    /// The URL as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A relative path that cannot leave its base directory lexically: no absolute
/// paths, no `..` components, no backslashes and no NUL bytes.
///
/// Symlinks are resolved separately when the file is opened.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelPath(PathBuf);

impl RelPath {
    /// Maximum length of a relative path, in bytes.
    pub const MAX_LEN: usize = 4096;

    /// Validates a relative path.
    pub fn new(value: impl Into<String>) -> Result<Self, ValueError> {
        let value = value.into();
        let reason = if value.is_empty() {
            Some("must not be empty")
        } else if value.len() > Self::MAX_LEN {
            Some("is too long")
        } else if value.contains('\0') {
            Some("must not contain NUL bytes")
        } else if value.contains('\\') {
            Some("must use / as separator, not backslashes")
        } else {
            Path::new(&value)
                .components()
                .find_map(|component| match component {
                    Component::ParentDir => Some("must not contain .. components"),
                    Component::RootDir | Component::Prefix(_) => Some("must be relative"),
                    Component::CurDir | Component::Normal(_) => None,
                })
        };
        match reason {
            None => Ok(Self(PathBuf::from(value))),
            Some(reason) => Err(ValueError::RelPath {
                value: excerpt(&value),
                reason,
            }),
        }
    }

    /// The path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

macro_rules! string_newtype_impls {
    ($($ty:ident),*) => {$(
        impl TryFrom<String> for $ty {
            type Error = ValueError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.0
            }
        }
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    )*};
}
string_newtype_impls!(GitSha, Sha256, RepoUrl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_sha_requires_40_lowercase_hex() {
        assert!(GitSha::new("0123456789abcdef0123456789abcdef01234567").is_ok());
        assert!(GitSha::new("0123456").is_err());
        assert!(GitSha::new("0123456789ABCDEF0123456789ABCDEF01234567").is_err());
        assert!(GitSha::new("0123456789abcdef0123456789abcdef0123456g").is_err());
        assert!(GitSha::new("main").is_err());
    }

    #[test]
    fn sha256_of_known_input() {
        // SHA-256 of "hello world".
        assert_eq!(
            Sha256::of(b"hello world").as_str(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert!(Sha256::new(Sha256::of(b"").as_str()).is_ok());
        assert!(Sha256::new("b94d27b9").is_err());
    }

    #[test]
    fn repo_url_accepts_normalized_https() {
        for ok in [
            "https://github.com/GNOME/libxml2",
            "https://gitlab.example.org/group/sub/project",
            "https://git.example.org:8443/project",
        ] {
            assert!(RepoUrl::new(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn repo_url_rejects_non_normalized_or_unsafe_urls() {
        for bad in [
            "http://github.com/curl/curl",
            "https://github.com/curl/curl.git",
            "https://github.com/curl/curl/",
            "https://user:token@github.com/curl/curl",
            "https://GitHub.com/curl/curl",
            "https://github.com/curl/curl?tab=readme",
            "https://github.com/curl/curl#readme",
            "https://github.com/curl/../evil",
            "https://github.com",
            "https://localhost/curl",
            "git@github.com:curl/curl",
        ] {
            assert!(RepoUrl::new(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn rel_path_rejects_escapes() {
        assert!(RelPath::new("poc/crash").is_ok());
        assert!(RelPath::new("./poc/crash").is_ok());
        for bad in [
            "",
            "/etc/passwd",
            "../secret",
            "poc/../../secret",
            "poc\\crash",
            "poc/\0crash",
        ] {
            assert!(RelPath::new(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn errors_never_echo_huge_values() {
        let huge = "x".repeat(10_000);
        let message = RepoUrl::new(format!("https://{huge}"))
            .map(|_| ())
            .unwrap_err()
            .to_string();
        assert!(message.len() < 300, "{message}");
    }
}
