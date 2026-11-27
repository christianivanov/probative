//! Loading a manifest from disk: the rules that need the file system or the
//! configuration.
//!
//! The PoC is read exactly once. The same bytes are hashed, checked and later
//! used to build the run image, so replacing the file after validation has no
//! effect.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::ManifestError;
use crate::manifest::Manifest;
use crate::values::{RepoUrl, Sha256};

/// Default maximum PoC size: 10 MiB.
pub const DEFAULT_MAX_POC_BYTES: u64 = 10 * 1024 * 1024;
/// Maximum manifest size: 1 MiB. Real manifests are a few hundred bytes.
pub const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

/// The configuration a manifest is checked against.
#[derive(Debug, Clone, Copy)]
pub struct LoadPolicy<'a> {
    /// Repositories Probative may build.
    pub allowlist: &'a [RepoUrl],
    /// Maximum PoC size, in bytes.
    pub max_poc_bytes: u64,
}

/// The PoC, read once and hashed.
#[derive(Clone, PartialEq, Eq)]
pub struct PocBundle {
    /// File name used inside the sandbox, under `/poc/`.
    pub file_name: String,
    /// The PoC bytes.
    pub bytes: Vec<u8>,
    /// SHA-256 of `bytes`.
    pub sha256: Sha256,
}

impl std::fmt::Debug for PocBundle {
    /// Never prints the PoC bytes, which may be under embargo.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PocBundle")
            .field("file_name", &self.file_name)
            .field("len", &self.bytes.len())
            .field("sha256", &self.sha256)
            .finish()
    }
}

/// A validated manifest together with its PoC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedManifest {
    /// The validated manifest.
    pub manifest: Manifest,
    /// SHA-256 of the manifest file's bytes.
    pub manifest_sha256: Sha256,
    /// The PoC.
    pub poc: PocBundle,
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> ManifestError + '_ {
    move |source| ManifestError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Reads at most `max` bytes; returns `None` if the file is larger.
fn read_limited(file: File, max: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    file.take(max.saturating_add(1)).read_to_end(&mut bytes)?;
    Ok((bytes.len() as u64 <= max).then_some(bytes))
}

/// Loads a manifest, checks it against `policy` and reads its PoC.
///
/// On top of [`Manifest::from_toml_str`], this checks that the repository is
/// on the allowlist, that the PoC stays inside the manifest's directory after
/// resolving symlinks, that it is a regular file within the size limit, and
/// that it matches the declared SHA-256.
pub fn load_manifest(
    path: &Path,
    policy: &LoadPolicy<'_>,
) -> Result<LoadedManifest, ManifestError> {
    let file = File::open(path).map_err(io_error(path))?;
    let bytes = read_limited(file, MAX_MANIFEST_BYTES)
        .map_err(io_error(path))?
        .ok_or(ManifestError::ManifestTooLarge {
            max: MAX_MANIFEST_BYTES,
        })?;
    let text = std::str::from_utf8(&bytes).map_err(|_| ManifestError::NotUtf8)?;
    let manifest = Manifest::from_toml_str(text)?;

    if !policy.allowlist.contains(&manifest.target.repo_url) {
        return Err(ManifestError::RepoNotAllowed(manifest.target.repo_url));
    }

    let poc = read_poc(path, &manifest, policy.max_poc_bytes)?;

    Ok(LoadedManifest {
        manifest,
        manifest_sha256: Sha256::of(&bytes),
        poc,
    })
}

fn read_poc(
    manifest_path: &Path,
    manifest: &Manifest,
    max: u64,
) -> Result<PocBundle, ManifestError> {
    let relative = manifest.poc.file().as_path();
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    let base = if base.as_os_str().is_empty() {
        Path::new(".")
    } else {
        base
    };
    let base = base.canonicalize().map_err(io_error(base))?;

    let joined: PathBuf = base.join(relative);
    let resolved = joined.canonicalize().map_err(io_error(&joined))?;
    if !resolved.starts_with(&base) {
        return Err(ManifestError::PocOutsideManifestDir {
            path: relative.to_path_buf(),
        });
    }
    let metadata = resolved.metadata().map_err(io_error(&resolved))?;
    if !metadata.is_file() {
        return Err(ManifestError::PocNotAFile {
            path: relative.to_path_buf(),
        });
    }

    let file = File::open(&resolved).map_err(io_error(&resolved))?;
    let bytes = read_limited(file, max)
        .map_err(io_error(&resolved))?
        .ok_or(ManifestError::PocTooLarge { max })?;
    let sha256 = Sha256::of(&bytes);
    if let Some(expected) = manifest.poc.sha256()
        && *expected != sha256
    {
        return Err(ManifestError::PocHashMismatch {
            expected: expected.clone(),
            actual: sha256,
        });
    }

    let file_name = relative
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| ManifestError::Invalid {
            field: "poc.file",
            reason: "must name a file".into(),
        })?;

    Ok(PocBundle {
        file_name,
        bytes,
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const REPO: &str = "https://github.com/example/libexample";

    fn manifest_text(poc_file: &str, sha256: Option<&str>) -> String {
        let sha = sha256
            .map(|s| format!("sha256 = \"{s}\"\n"))
            .unwrap_or_default();
        format!(
            "schema_version = \"0.1\"\n\n[target]\nrepo_url = \"{REPO}\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n\n\
             [build]\nrecipe = \"oss_fuzz\"\noss_fuzz_project = \"libexample\"\n\n\
             [poc]\ntype = \"fuzz_input\"\nfile = \"{poc_file}\"\nfuzz_target = \"parse_fuzzer\"\n{sha}"
        )
    }

    /// A case directory with a manifest at `case/probative.toml` and a PoC at `case/poc/crash`.
    fn case(poc: &[u8], sha256: Option<&str>) -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let case = dir.path().join("case");
        fs::create_dir_all(case.join("poc")).unwrap();
        fs::write(case.join("poc/crash"), poc).unwrap();
        let manifest = case.join("probative.toml");
        fs::write(&manifest, manifest_text("poc/crash", sha256)).unwrap();
        (dir, manifest)
    }

    fn allow() -> Vec<RepoUrl> {
        vec![RepoUrl::new(REPO).unwrap()]
    }

    fn policy(allowlist: &[RepoUrl]) -> LoadPolicy<'_> {
        LoadPolicy {
            allowlist,
            max_poc_bytes: DEFAULT_MAX_POC_BYTES,
        }
    }

    #[test]
    fn loads_the_manifest_and_reads_the_poc_once() {
        let (_dir, path) = case(b"crash bytes", None);
        let loaded = load_manifest(&path, &policy(&allow())).unwrap();
        assert_eq!(loaded.poc.bytes, b"crash bytes");
        assert_eq!(loaded.poc.sha256, Sha256::of(b"crash bytes"));
        assert_eq!(loaded.poc.file_name, "crash");
        assert_eq!(
            loaded.manifest_sha256,
            Sha256::of(&fs::read(&path).unwrap())
        );
    }

    #[test]
    fn debug_output_never_contains_poc_bytes() {
        let (_dir, path) = case(b"SECRET-EXPLOIT-BYTES", None);
        let loaded = load_manifest(&path, &policy(&allow())).unwrap();
        assert!(!format!("{loaded:?}").contains("SECRET-EXPLOIT-BYTES"));
    }

    #[test]
    fn repository_must_be_on_the_allowlist() {
        let (_dir, path) = case(b"x", None);
        let other = vec![RepoUrl::new("https://github.com/example/other").unwrap()];
        let err = load_manifest(&path, &policy(&other)).unwrap_err();
        assert!(matches!(err, ManifestError::RepoNotAllowed(_)));
        assert!(err.is_policy_violation());
    }

    #[test]
    fn poc_over_the_size_limit_is_a_policy_error() {
        let (_dir, path) = case(&[0u8; 101], None);
        let small = LoadPolicy {
            allowlist: &allow(),
            max_poc_bytes: 100,
        };
        let err = load_manifest(&path, &small).unwrap_err();
        assert!(matches!(err, ManifestError::PocTooLarge { max: 100 }));
        assert!(err.is_policy_violation());

        let (_dir, path) = case(&[0u8; 100], None);
        assert!(load_manifest(&path, &small).is_ok());
    }

    #[test]
    fn declared_sha256_must_match() {
        let good = Sha256::of(b"x");
        let (_dir, path) = case(b"x", Some(good.as_str()));
        assert!(load_manifest(&path, &policy(&allow())).is_ok());

        let wrong = Sha256::of(b"y");
        let (_dir, path) = case(b"x", Some(wrong.as_str()));
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::PocHashMismatch { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_must_not_escape_the_manifest_directory() {
        let (dir, path) = case(b"x", None);
        let outside = dir.path().join("secret");
        fs::write(&outside, b"host secret").unwrap();
        let case_dir = path.parent().unwrap();
        fs::remove_file(case_dir.join("poc/crash")).unwrap();
        std::os::unix::fs::symlink(&outside, case_dir.join("poc/crash")).unwrap();
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::PocOutsideManifestDir { .. })
        ));

        // A directory symlink that leaves the case is rejected too.
        fs::remove_file(case_dir.join("poc/crash")).unwrap();
        fs::remove_dir(case_dir.join("poc")).unwrap();
        std::os::unix::fs::symlink(dir.path(), case_dir.join("poc")).unwrap();
        fs::write(dir.path().join("crash"), b"x").unwrap();
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::PocOutsideManifestDir { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_inside_the_manifest_directory_are_allowed() {
        let (_dir, path) = case(b"x", None);
        let case_dir = path.parent().unwrap();
        fs::write(case_dir.join("real"), b"inside").unwrap();
        fs::remove_file(case_dir.join("poc/crash")).unwrap();
        std::os::unix::fs::symlink(case_dir.join("real"), case_dir.join("poc/crash")).unwrap();
        let loaded = load_manifest(&path, &policy(&allow())).unwrap();
        assert_eq!(loaded.poc.bytes, b"inside");
    }

    #[test]
    fn poc_must_be_a_regular_file() {
        let (_dir, path) = case(b"x", None);
        let case_dir = path.parent().unwrap();
        fs::remove_file(case_dir.join("poc/crash")).unwrap();
        fs::create_dir(case_dir.join("poc/crash")).unwrap();
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::PocNotAFile { .. })
        ));
    }

    #[test]
    fn missing_poc_is_an_io_error() {
        let (_dir, path) = case(b"x", None);
        fs::remove_file(path.parent().unwrap().join("poc/crash")).unwrap();
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::Io { .. })
        ));
    }

    #[test]
    fn oversized_manifest_is_rejected_before_parsing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("probative.toml");
        let mut text = manifest_text("poc/crash", None);
        text.push_str(&format!(
            "\n[extensions]\npad = \"{}\"\n",
            "a".repeat(MAX_MANIFEST_BYTES as usize)
        ));
        fs::write(&path, text).unwrap();
        let err = load_manifest(&path, &policy(&allow())).unwrap_err();
        assert!(matches!(err, ManifestError::ManifestTooLarge { .. }));
    }

    #[test]
    fn manifest_must_be_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("probative.toml");
        fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        assert!(matches!(
            load_manifest(&path, &policy(&allow())),
            Err(ManifestError::NotUtf8)
        ));
    }
}
