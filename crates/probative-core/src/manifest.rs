//! The `probative.toml` manifest: its typed model and the rules that turn TOML
//! text into it.
//!
//! [`Manifest::from_toml_str`] applies every rule that does not need the file
//! system or the configuration. [`crate::load_manifest`] adds the rest: the
//! allowlist, symlink resolution, the PoC size limit and its hash.

use serde::Deserialize;

use crate::domain::{Access, BugKind, FoundWith, Lang, Sanitizer};
use crate::error::ManifestError;
use crate::values::{GitSha, RelPath, RepoUrl, Sha256};

/// The only manifest format version this release reads.
pub const SCHEMA_VERSION: &str = "0.1";

/// Maximum number of `argv` elements.
pub const MAX_ARGV: usize = 64;
/// Maximum length of one `argv` element, in bytes.
pub const MAX_ARG_BYTES: usize = 4096;
/// Maximum length of the free-text fields in `[claim]` and `[provenance]`, in characters.
pub const MAX_TEXT_CHARS: usize = 256;
/// Placeholder in `argv` that is replaced by the PoC path inside the sandbox.
pub const POC_PLACEHOLDER: &str = "{poc}";

/// A validated manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Always [`SCHEMA_VERSION`].
    pub schema_version: String,
    /// What to build.
    pub target: Target,
    /// How to build it.
    pub build: BuildSpec,
    /// The proof of concept.
    pub poc: PocSpec,
    /// What the report claims. Empty when the manifest has no `[claim]`.
    pub claim: ClaimedBug,
    /// Where the report came from. Never influences the verdict.
    pub provenance: Option<Provenance>,
}

/// The repository and commits to verify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// Repository, normalized; must be on the allowlist.
    pub repo_url: RepoUrl,
    /// The affected commit.
    pub commit: GitSha,
    /// The commit that fixes the bug, if known.
    pub fix_commit: Option<GitSha>,
}

/// How to build the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSpec {
    /// The recipe.
    pub recipe: Recipe,
    /// The sanitizer; address by default.
    pub sanitizer: Sanitizer,
}

/// A build recipe. Recipes are trusted configuration: the manifest only names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipe {
    /// A project integrated in OSS-Fuzz.
    OssFuzz {
        /// The OSS-Fuzz project name.
        project: String,
        /// The OSS-Fuzz commit; the configured one if absent.
        oss_fuzz_commit: Option<GitSha>,
    },
    /// A prebuilt ARVO case.
    Arvo {
        /// The ARVO case identifier.
        id: String,
    },
    /// A reviewed manual recipe, from Probative's `recipes/` or the operator's local directory.
    Manual {
        /// The recipe name.
        name: String,
    },
}

/// The proof of concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PocSpec {
    /// An input file for a fuzz target.
    FuzzInput {
        /// The PoC file, relative to the manifest.
        file: RelPath,
        /// Expected SHA-256 of the file.
        sha256: Option<Sha256>,
        /// The fuzz target binary name.
        fuzz_target: String,
    },
    /// An invocation of a binary declared by the recipe.
    Cli {
        /// The PoC file, relative to the manifest.
        file: RelPath,
        /// Expected SHA-256 of the file.
        sha256: Option<Sha256>,
        /// The binary name, as declared by the recipe.
        binary: String,
        /// Arguments; [`POC_PLACEHOLDER`] is replaced by the PoC path.
        argv: Vec<String>,
        /// Whether the PoC is passed on standard input.
        stdin: bool,
    },
    /// A C or C++ program linked against the library.
    Harness {
        /// The PoC file, relative to the manifest.
        file: RelPath,
        /// Expected SHA-256 of the file.
        sha256: Option<Sha256>,
        /// The harness language.
        lang: Lang,
    },
}

impl PocSpec {
    /// The PoC file, relative to the manifest.
    pub fn file(&self) -> &RelPath {
        match self {
            Self::FuzzInput { file, .. } | Self::Cli { file, .. } | Self::Harness { file, .. } => {
                file
            }
        }
    }

    /// The SHA-256 declared in the manifest, if any.
    pub fn sha256(&self) -> Option<&Sha256> {
        match self {
            Self::FuzzInput { sha256, .. }
            | Self::Cli { sha256, .. }
            | Self::Harness { sha256, .. } => sha256.as_ref(),
        }
    }
}

/// What the report claims. Every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimedBug {
    /// The claimed bug kind.
    pub kind: Option<BugKind>,
    /// The claimed access type.
    pub access: Option<Access>,
    /// The claimed function.
    pub function: Option<String>,
    /// The claimed source file.
    pub file: Option<String>,
}

/// Where the report came from. Informative only.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    /// Who reported it.
    pub reporter: Option<String>,
    /// How it was found.
    pub found_with: Option<FoundWith>,
    /// The tool used.
    pub tool: Option<String>,
    /// The model used, if any.
    pub model: Option<String>,
}

// Raw TOML structure. Every value is read as a string first, so each rule can
// report the exact field and reason instead of a generic deserialization error.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    schema_version: String,
    target: RawTarget,
    build: RawBuild,
    poc: RawPoc,
    #[serde(default)]
    claim: Option<RawClaim>,
    #[serde(default)]
    provenance: Option<RawProvenance>,
    /// Free-form; ignored by `verify`.
    #[serde(default, rename = "extensions")]
    _extensions: Option<toml::Table>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTarget {
    repo_url: String,
    commit: String,
    fix_commit: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuild {
    recipe: String,
    sanitizer: Option<String>,
    oss_fuzz_project: Option<String>,
    oss_fuzz_commit: Option<String>,
    arvo_id: Option<String>,
    manual: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPoc {
    #[serde(rename = "type")]
    kind: String,
    file: String,
    sha256: Option<String>,
    fuzz_target: Option<String>,
    binary: Option<String>,
    argv: Option<Vec<String>>,
    stdin: Option<bool>,
    lang: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClaim {
    kind: Option<String>,
    access: Option<String>,
    function: Option<String>,
    file: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvenance {
    reporter: Option<String>,
    found_with: Option<String>,
    tool: Option<String>,
    model: Option<String>,
}

fn invalid(field: &'static str, reason: impl Into<String>) -> ManifestError {
    ManifestError::Invalid {
        field,
        reason: reason.into(),
    }
}

fn required<T>(
    field: &'static str,
    value: Option<T>,
    because: &'static str,
) -> Result<T, ManifestError> {
    value.ok_or(ManifestError::Missing { field, because })
}

fn forbid<T>(
    field: &'static str,
    value: &Option<T>,
    because: &'static str,
) -> Result<(), ManifestError> {
    match value {
        Some(_) => Err(ManifestError::Forbidden { field, because }),
        None => Ok(()),
    }
}

/// Recipe and OSS-Fuzz project names: lowercase file names, never paths.
fn project_name(field: &'static str, value: String) -> Result<String, ManifestError> {
    let ok = (1..=64).contains(&value.len())
        && value.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && value.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        });
    if ok {
        Ok(value)
    } else {
        Err(invalid(
            field,
            "must be a lowercase name (a-z, 0-9, '.', '_', '-'), never a path",
        ))
    }
}

/// Binary and fuzz target names: file names, never paths.
fn binary_name(field: &'static str, value: String) -> Result<String, ManifestError> {
    let ok = (1..=128).contains(&value.len())
        && value.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'+' | b'-'));
    if ok {
        Ok(value)
    } else {
        Err(invalid(
            field,
            "must be a binary name (A-Z, a-z, 0-9, '_', '.', '+', '-'), never a path",
        ))
    }
}

fn text(
    field: &'static str,
    value: Option<String>,
    allow_empty: bool,
) -> Result<Option<String>, ManifestError> {
    match value {
        Some(v) if v.chars().count() > MAX_TEXT_CHARS => Err(invalid(
            field,
            format!("must be at most {MAX_TEXT_CHARS} characters"),
        )),
        Some(v) if v.is_empty() && !allow_empty => Err(invalid(field, "must not be empty")),
        other => Ok(other),
    }
}

fn git_sha(field: &'static str, value: String) -> Result<GitSha, ManifestError> {
    GitSha::new(value).map_err(|e| invalid(field, e.to_string()))
}

fn sha256(field: &'static str, value: String) -> Result<Sha256, ManifestError> {
    Sha256::new(value).map_err(|e| invalid(field, e.to_string()))
}

fn enum_value<T: Copy>(
    field: &'static str,
    value: &str,
    options: &[(&str, T)],
) -> Result<T, ManifestError> {
    options
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, v)| *v)
        .ok_or_else(|| {
            let names: Vec<&str> = options.iter().map(|(name, _)| *name).collect();
            invalid(field, format!("must be one of: {}", names.join(", ")))
        })
}

impl Manifest {
    /// Parses and validates a manifest from TOML text.
    ///
    /// This applies every rule that does not need the file system or the
    /// configuration. Use [`crate::load_manifest`] to also check the allowlist
    /// and load the PoC.
    pub fn from_toml_str(text: &str) -> Result<Self, ManifestError> {
        let raw: RawManifest = toml::from_str(text)?;

        if raw.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::SchemaVersion {
                found: raw.schema_version.chars().take(32).collect(),
            });
        }

        Ok(Self {
            schema_version: raw.schema_version,
            target: Self::target(raw.target)?,
            build: Self::build(raw.build)?,
            poc: Self::poc(raw.poc)?,
            claim: Self::claim(raw.claim)?,
            provenance: raw.provenance.map(Self::provenance).transpose()?,
        })
    }

    fn target(raw: RawTarget) -> Result<Target, ManifestError> {
        Ok(Target {
            repo_url: RepoUrl::new(raw.repo_url)
                .map_err(|e| invalid("target.repo_url", e.to_string()))?,
            commit: git_sha("target.commit", raw.commit)?,
            fix_commit: raw
                .fix_commit
                .map(|c| git_sha("target.fix_commit", c))
                .transpose()?,
        })
    }

    fn build(raw: RawBuild) -> Result<BuildSpec, ManifestError> {
        let sanitizer = match raw.sanitizer.as_deref() {
            None => Sanitizer::default(),
            Some(value) => enum_value(
                "build.sanitizer",
                value,
                &[
                    ("address", Sanitizer::Address),
                    ("undefined", Sanitizer::Undefined),
                ],
            )?,
        };

        let recipe = match raw.recipe.as_str() {
            "oss_fuzz" => {
                const WHY: &str = "with recipe = \"oss_fuzz\"";
                forbid("build.arvo_id", &raw.arvo_id, WHY)?;
                forbid("build.manual", &raw.manual, WHY)?;
                let project = required("build.oss_fuzz_project", raw.oss_fuzz_project, WHY)?;
                Recipe::OssFuzz {
                    project: project_name("build.oss_fuzz_project", project)?,
                    oss_fuzz_commit: raw
                        .oss_fuzz_commit
                        .map(|c| git_sha("build.oss_fuzz_commit", c))
                        .transpose()?,
                }
            }
            "arvo" => {
                const WHY: &str = "with recipe = \"arvo\"";
                forbid("build.oss_fuzz_project", &raw.oss_fuzz_project, WHY)?;
                forbid("build.oss_fuzz_commit", &raw.oss_fuzz_commit, WHY)?;
                forbid("build.manual", &raw.manual, WHY)?;
                let id = required("build.arvo_id", raw.arvo_id, WHY)?;
                if id.is_empty() || id.len() > 12 || !id.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(invalid("build.arvo_id", "must be a numeric ARVO case id"));
                }
                Recipe::Arvo { id }
            }
            "manual" => {
                const WHY: &str = "with recipe = \"manual\"";
                forbid("build.oss_fuzz_project", &raw.oss_fuzz_project, WHY)?;
                forbid("build.oss_fuzz_commit", &raw.oss_fuzz_commit, WHY)?;
                forbid("build.arvo_id", &raw.arvo_id, WHY)?;
                let name = required("build.manual", raw.manual, WHY)?;
                Recipe::Manual {
                    name: project_name("build.manual", name)?,
                }
            }
            _ => {
                return Err(invalid(
                    "build.recipe",
                    "must be one of: oss_fuzz, arvo, manual",
                ));
            }
        };

        Ok(BuildSpec { recipe, sanitizer })
    }

    fn poc(raw: RawPoc) -> Result<PocSpec, ManifestError> {
        let file = RelPath::new(raw.file).map_err(|e| invalid("poc.file", e.to_string()))?;
        let sha = raw.sha256.map(|s| sha256("poc.sha256", s)).transpose()?;

        match raw.kind.as_str() {
            "fuzz_input" => {
                const WHY: &str = "with type = \"fuzz_input\"";
                forbid("poc.binary", &raw.binary, WHY)?;
                forbid("poc.argv", &raw.argv, WHY)?;
                forbid("poc.stdin", &raw.stdin, WHY)?;
                forbid("poc.lang", &raw.lang, WHY)?;
                let target = required("poc.fuzz_target", raw.fuzz_target, WHY)?;
                Ok(PocSpec::FuzzInput {
                    file,
                    sha256: sha,
                    fuzz_target: binary_name("poc.fuzz_target", target)?,
                })
            }
            "cli" => {
                const WHY: &str = "with type = \"cli\"";
                forbid("poc.fuzz_target", &raw.fuzz_target, WHY)?;
                forbid("poc.lang", &raw.lang, WHY)?;
                let binary = required("poc.binary", raw.binary, WHY)?;
                let stdin = raw.stdin.unwrap_or(false);
                let argv = raw.argv.unwrap_or_default();
                if argv.len() > MAX_ARGV {
                    return Err(invalid(
                        "poc.argv",
                        format!("must have at most {MAX_ARGV} elements"),
                    ));
                }
                if argv.iter().any(|arg| arg.len() > MAX_ARG_BYTES) {
                    return Err(invalid(
                        "poc.argv",
                        format!("elements must be at most {MAX_ARG_BYTES} bytes"),
                    ));
                }
                if argv.iter().any(|arg| arg.contains('\0')) {
                    return Err(invalid("poc.argv", "elements must not contain NUL bytes"));
                }
                if !stdin && !argv.iter().any(|arg| arg.contains(POC_PLACEHOLDER)) {
                    return Err(invalid(
                        "poc.argv",
                        "must contain {poc} somewhere, or set stdin = true, so the PoC reaches the binary",
                    ));
                }
                Ok(PocSpec::Cli {
                    file,
                    sha256: sha,
                    binary: binary_name("poc.binary", binary)?,
                    argv,
                    stdin,
                })
            }
            "harness" => {
                const WHY: &str = "with type = \"harness\"";
                forbid("poc.fuzz_target", &raw.fuzz_target, WHY)?;
                forbid("poc.binary", &raw.binary, WHY)?;
                forbid("poc.argv", &raw.argv, WHY)?;
                forbid("poc.stdin", &raw.stdin, WHY)?;
                let lang = required("poc.lang", raw.lang, WHY)?;
                Ok(PocSpec::Harness {
                    file,
                    sha256: sha,
                    lang: enum_value("poc.lang", &lang, &[("c", Lang::C), ("cpp", Lang::Cpp)])?,
                })
            }
            _ => Err(invalid(
                "poc.type",
                "must be one of: fuzz_input, cli, harness",
            )),
        }
    }

    fn claim(raw: Option<RawClaim>) -> Result<ClaimedBug, ManifestError> {
        let Some(raw) = raw else {
            return Ok(ClaimedBug::default());
        };
        Ok(ClaimedBug {
            kind: raw
                .kind
                .map(|k| {
                    k.parse::<BugKind>()
                        .map_err(|e| invalid("claim.kind", e.to_string()))
                })
                .transpose()?,
            access: raw
                .access
                .map(|a| {
                    enum_value(
                        "claim.access",
                        &a,
                        &[("read", Access::Read), ("write", Access::Write)],
                    )
                })
                .transpose()?,
            function: text("claim.function", raw.function, false)?,
            file: text("claim.file", raw.file, false)?,
        })
    }

    fn provenance(raw: RawProvenance) -> Result<Provenance, ManifestError> {
        Ok(Provenance {
            reporter: text("provenance.reporter", raw.reporter, true)?,
            found_with: raw
                .found_with
                .map(|f| {
                    enum_value(
                        "provenance.found_with",
                        &f,
                        &[
                            ("fuzzing", FoundWith::Fuzzing),
                            ("llm_agent", FoundWith::LlmAgent),
                            ("manual", FoundWith::Manual),
                            ("other", FoundWith::Other),
                        ],
                    )
                })
                .transpose()?,
            tool: text("provenance.tool", raw.tool, true)?,
            model: text("provenance.model", raw.model, true)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#"
schema_version = "0.1"

[target]
repo_url = "https://github.com/example/libexample"
commit = "0123456789abcdef0123456789abcdef01234567"
fix_commit = "89abcdef0123456789abcdef0123456789abcdef"

[build]
recipe = "oss_fuzz"
sanitizer = "address"
oss_fuzz_project = "libexample"

[poc]
type = "fuzz_input"
file = "poc/crash-1a2b3c"
fuzz_target = "parse_fuzzer"

[claim]
kind = "heap-buffer-overflow"
access = "write"
function = "parse_chunk"
file = "parser.c"
"#;

    const CLI: &str = r#"
schema_version = "0.1"

[target]
repo_url = "https://github.com/example/tool"
commit = "0123456789abcdef0123456789abcdef01234567"

[build]
recipe = "manual"
manual = "tool-cli"

[poc]
type = "cli"
file = "poc/input"
binary = "tool"
argv = ["--check", "{poc}"]
"#;

    const HARNESS: &str = r#"
schema_version = "0.1"

[target]
repo_url = "https://github.com/example/libexample"
commit = "0123456789abcdef0123456789abcdef01234567"

[build]
recipe = "manual"
manual = "libexample"

[poc]
type = "harness"
file = "poc/harness.c"
lang = "c"
"#;

    const ARVO: &str = r#"
schema_version = "0.1"

[target]
repo_url = "https://github.com/example/libexample"
commit = "0123456789abcdef0123456789abcdef01234567"

[build]
recipe = "arvo"
arvo_id = "42487096"

[poc]
type = "fuzz_input"
file = "testcase"
fuzz_target = "parse_fuzzer"
"#;

    /// Replaces exactly one occurrence of `old` in `base`.
    fn with(base: &str, old: &str, new: &str) -> String {
        assert_eq!(
            base.matches(old).count(),
            1,
            "fixture must contain {old:?} once"
        );
        base.replacen(old, new, 1)
    }

    fn parse(text: &str) -> Result<Manifest, ManifestError> {
        Manifest::from_toml_str(text)
    }

    fn invalid_field(text: &str) -> &'static str {
        match parse(text) {
            Err(ManifestError::Invalid { field, .. }) => field,
            other => panic!("expected an invalid field, got {other:?}"),
        }
    }

    fn missing_field(text: &str) -> &'static str {
        match parse(text) {
            Err(ManifestError::Missing { field, .. }) => field,
            other => panic!("expected a missing field, got {other:?}"),
        }
    }

    fn forbidden_field(text: &str) -> &'static str {
        match parse(text) {
            Err(ManifestError::Forbidden { field, .. }) => field,
            other => panic!("expected a forbidden field, got {other:?}"),
        }
    }

    fn assert_toml_error(text: &str, needle: &str) {
        match parse(text) {
            Err(ManifestError::Toml(e)) => assert!(e.to_string().contains(needle), "{e}"),
            other => panic!("expected a TOML error mentioning {needle:?}, got {other:?}"),
        }
    }

    // Valid manifests and defaults

    #[test]
    fn parses_a_complete_manifest() {
        let m = parse(BASE).unwrap();
        assert_eq!(
            m.target.repo_url.as_str(),
            "https://github.com/example/libexample"
        );
        assert_eq!(
            m.target.fix_commit.unwrap().as_str(),
            "89abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(
            m.build.recipe,
            Recipe::OssFuzz {
                project: "libexample".into(),
                oss_fuzz_commit: None
            }
        );
        assert_eq!(
            m.poc.file().as_path(),
            std::path::Path::new("poc/crash-1a2b3c")
        );
        assert_eq!(m.claim.kind, Some(BugKind::HeapBufferOverflow));
        assert_eq!(m.claim.access, Some(Access::Write));
        assert_eq!(m.provenance, None);
    }

    #[test]
    fn applies_defaults() {
        let text = with(BASE, "sanitizer = \"address\"\n", "");
        let text = with(
            &text,
            "[claim]\nkind = \"heap-buffer-overflow\"\naccess = \"write\"\nfunction = \"parse_chunk\"\nfile = \"parser.c\"\n",
            "",
        );
        let m = parse(&text).unwrap();
        assert_eq!(m.build.sanitizer, Sanitizer::Address);
        assert_eq!(m.claim, ClaimedBug::default());
        let cli = parse(CLI).unwrap();
        assert!(matches!(cli.poc, PocSpec::Cli { stdin: false, .. }));
    }

    #[test]
    fn parses_every_recipe_and_poc_type() {
        assert_eq!(
            parse(ARVO).unwrap().build.recipe,
            Recipe::Arvo {
                id: "42487096".into()
            }
        );
        assert_eq!(
            parse(CLI).unwrap().build.recipe,
            Recipe::Manual {
                name: "tool-cli".into()
            }
        );
        assert!(matches!(
            parse(HARNESS).unwrap().poc,
            PocSpec::Harness { lang: Lang::C, .. }
        ));
        let undefined = with(BASE, "sanitizer = \"address\"", "sanitizer = \"undefined\"");
        assert_eq!(
            parse(&undefined).unwrap().build.sanitizer,
            Sanitizer::Undefined
        );
    }

    // schema_version

    #[test]
    fn schema_version_must_be_0_1() {
        let text = with(BASE, "schema_version = \"0.1\"", "schema_version = \"1.0\"");
        assert!(matches!(
            parse(&text),
            Err(ManifestError::SchemaVersion { .. })
        ));
    }

    // Unknown keys and [extensions]

    #[test]
    fn rejects_unknown_keys_everywhere() {
        assert_toml_error(
            &with(
                BASE,
                "schema_version = \"0.1\"",
                "schema_version = \"0.1\"\nnotes = \"x\"",
            ),
            "notes",
        );
        assert_toml_error(
            &with(BASE, "fix_commit =", "branch = \"main\"\nfix_commit ="),
            "branch",
        );
        assert_toml_error(
            &with(
                BASE,
                "recipe = \"oss_fuzz\"",
                "recipe = \"oss_fuzz\"\nflags = \"-O0\"",
            ),
            "flags",
        );
        assert_toml_error(
            &with(
                BASE,
                "type = \"fuzz_input\"",
                "type = \"fuzz_input\"\nenv = \"X=1\"",
            ),
            "env",
        );
        assert_toml_error(
            &with(
                BASE,
                "access = \"write\"",
                "access = \"write\"\nseverity = \"high\"",
            ),
            "severity",
        );
    }

    #[test]
    fn ignores_anything_under_extensions() {
        let text = format!(
            "{BASE}\n[extensions.tracker]\nticket = \"SEC-1\"\nnested = {{ any = [1, 2] }}\n"
        );
        assert!(parse(&text).is_ok());
    }

    // target

    #[test]
    fn repo_url_must_be_normalized_https() {
        for bad in [
            "http://github.com/example/libexample",
            "https://github.com/example/libexample.git",
            "https://github.com/example/libexample/",
            "https://user:token@github.com/example/libexample",
            "https://GitHub.com/example/libexample",
        ] {
            let text = with(BASE, "https://github.com/example/libexample", bad);
            assert_eq!(invalid_field(&text), "target.repo_url", "{bad}");
        }
    }

    #[test]
    fn commits_must_be_full_lowercase_sha1() {
        let short = with(
            BASE,
            "commit = \"0123456789abcdef0123456789abcdef01234567\"",
            "commit = \"0123456\"",
        );
        assert_eq!(invalid_field(&short), "target.commit");
        let upper = with(
            BASE,
            "commit = \"0123456789abcdef0123456789abcdef01234567\"",
            "commit = \"0123456789ABCDEF0123456789ABCDEF01234567\"",
        );
        assert_eq!(invalid_field(&upper), "target.commit");
        let fix = with(
            BASE,
            "fix_commit = \"89abcdef0123456789abcdef0123456789abcdef\"",
            "fix_commit = \"main\"",
        );
        assert_eq!(invalid_field(&fix), "target.fix_commit");
    }

    // build

    #[test]
    fn recipe_must_be_known() {
        assert_eq!(
            invalid_field(&with(BASE, "recipe = \"oss_fuzz\"", "recipe = \"script\"")),
            "build.recipe"
        );
    }

    #[test]
    fn sanitizer_must_be_known() {
        assert_eq!(
            invalid_field(&with(
                BASE,
                "sanitizer = \"address\"",
                "sanitizer = \"memory\""
            )),
            "build.sanitizer"
        );
    }

    #[test]
    fn oss_fuzz_recipe_requires_and_forbids_fields() {
        assert_eq!(
            missing_field(&with(BASE, "oss_fuzz_project = \"libexample\"\n", "")),
            "build.oss_fuzz_project"
        );
        let arvo = with(
            BASE,
            "oss_fuzz_project = \"libexample\"",
            "oss_fuzz_project = \"libexample\"\narvo_id = \"1\"",
        );
        assert_eq!(forbidden_field(&arvo), "build.arvo_id");
        let manual = with(
            BASE,
            "oss_fuzz_project = \"libexample\"",
            "oss_fuzz_project = \"libexample\"\nmanual = \"x\"",
        );
        assert_eq!(forbidden_field(&manual), "build.manual");
        let commit = with(
            BASE,
            "oss_fuzz_project = \"libexample\"",
            "oss_fuzz_project = \"libexample\"\noss_fuzz_commit = \"abc\"",
        );
        assert_eq!(invalid_field(&commit), "build.oss_fuzz_commit");
    }

    #[test]
    fn arvo_recipe_requires_and_forbids_fields() {
        assert_eq!(
            missing_field(&with(ARVO, "arvo_id = \"42487096\"\n", "")),
            "build.arvo_id"
        );
        let project = with(
            ARVO,
            "arvo_id = \"42487096\"",
            "arvo_id = \"42487096\"\noss_fuzz_project = \"x\"",
        );
        assert_eq!(forbidden_field(&project), "build.oss_fuzz_project");
        let commit = with(
            ARVO,
            "arvo_id = \"42487096\"",
            "arvo_id = \"42487096\"\noss_fuzz_commit = \"0123456789abcdef0123456789abcdef01234567\"",
        );
        assert_eq!(forbidden_field(&commit), "build.oss_fuzz_commit");
        assert_eq!(
            invalid_field(&with(ARVO, "arvo_id = \"42487096\"", "arvo_id = \"abc\"")),
            "build.arvo_id"
        );
    }

    #[test]
    fn manual_recipe_requires_and_forbids_fields() {
        assert_eq!(
            missing_field(&with(CLI, "manual = \"tool-cli\"\n", "")),
            "build.manual"
        );
        let project = with(
            CLI,
            "manual = \"tool-cli\"",
            "manual = \"tool-cli\"\noss_fuzz_project = \"x\"",
        );
        assert_eq!(forbidden_field(&project), "build.oss_fuzz_project");
        let arvo = with(
            CLI,
            "manual = \"tool-cli\"",
            "manual = \"tool-cli\"\narvo_id = \"1\"",
        );
        assert_eq!(forbidden_field(&arvo), "build.arvo_id");
    }

    #[test]
    fn recipe_and_project_names_are_never_paths() {
        assert_eq!(
            invalid_field(&with(
                CLI,
                "manual = \"tool-cli\"",
                "manual = \"../../etc/recipe\""
            )),
            "build.manual"
        );
        assert_eq!(
            invalid_field(&with(
                CLI,
                "manual = \"tool-cli\"",
                "manual = \"/tmp/recipe\""
            )),
            "build.manual"
        );
        assert_eq!(
            invalid_field(&with(
                BASE,
                "oss_fuzz_project = \"libexample\"",
                "oss_fuzz_project = \"../evil\""
            )),
            "build.oss_fuzz_project"
        );
    }

    // poc

    #[test]
    fn poc_type_must_be_known() {
        assert_eq!(
            invalid_field(&with(BASE, "type = \"fuzz_input\"", "type = \"server\"")),
            "poc.type"
        );
    }

    #[test]
    fn poc_file_rejects_path_traversal() {
        for bad in [
            "poc/../../secret",
            "../secret",
            "/etc/passwd",
            "poc\\\\crash",
            "",
        ] {
            let text = with(
                BASE,
                "file = \"poc/crash-1a2b3c\"",
                &format!("file = \"{bad}\""),
            );
            assert_eq!(invalid_field(&text), "poc.file", "{bad:?}");
        }
        let nul = with(
            BASE,
            "file = \"poc/crash-1a2b3c\"",
            "file = \"poc/\\u0000crash\"",
        );
        assert_eq!(invalid_field(&nul), "poc.file");
    }

    #[test]
    fn poc_sha256_must_be_a_digest() {
        let text = with(
            BASE,
            "fuzz_target = \"parse_fuzzer\"",
            "fuzz_target = \"parse_fuzzer\"\nsha256 = \"abc\"",
        );
        assert_eq!(invalid_field(&text), "poc.sha256");
    }

    #[test]
    fn fuzz_input_requires_and_forbids_fields() {
        assert_eq!(
            missing_field(&with(BASE, "fuzz_target = \"parse_fuzzer\"\n", "")),
            "poc.fuzz_target"
        );
        for (extra, field) in [
            ("binary = \"tool\"", "poc.binary"),
            ("argv = [\"{poc}\"]", "poc.argv"),
            ("stdin = true", "poc.stdin"),
            ("lang = \"c\"", "poc.lang"),
        ] {
            let text = with(
                BASE,
                "fuzz_target = \"parse_fuzzer\"",
                &format!("fuzz_target = \"parse_fuzzer\"\n{extra}"),
            );
            assert_eq!(forbidden_field(&text), field);
        }
    }

    #[test]
    fn fuzz_target_and_binary_are_never_paths() {
        let target = with(
            BASE,
            "fuzz_target = \"parse_fuzzer\"",
            "fuzz_target = \"../../bin/sh\"",
        );
        assert_eq!(invalid_field(&target), "poc.fuzz_target");
        let binary = with(CLI, "binary = \"tool\"", "binary = \"/bin/sh\"");
        assert_eq!(invalid_field(&binary), "poc.binary");
    }

    #[test]
    fn cli_requires_and_forbids_fields() {
        assert_eq!(
            missing_field(&with(CLI, "binary = \"tool\"\n", "")),
            "poc.binary"
        );
        let target = with(
            CLI,
            "binary = \"tool\"",
            "binary = \"tool\"\nfuzz_target = \"x\"",
        );
        assert_eq!(forbidden_field(&target), "poc.fuzz_target");
        let lang = with(CLI, "binary = \"tool\"", "binary = \"tool\"\nlang = \"c\"");
        assert_eq!(forbidden_field(&lang), "poc.lang");
    }

    #[test]
    fn cli_poc_must_reach_the_binary() {
        let no_poc = with(
            CLI,
            "argv = [\"--check\", \"{poc}\"]",
            "argv = [\"--check\"]",
        );
        assert_eq!(invalid_field(&no_poc), "poc.argv");
        let no_argv = with(CLI, "argv = [\"--check\", \"{poc}\"]\n", "");
        assert_eq!(invalid_field(&no_argv), "poc.argv");
        let stdin = with(
            CLI,
            "argv = [\"--check\", \"{poc}\"]",
            "argv = [\"--check\"]\nstdin = true",
        );
        assert!(parse(&stdin).is_ok());
        let embedded = with(CLI, "\"{poc}\"", "\"--input={poc}\"");
        assert!(parse(&embedded).is_ok());
    }

    #[test]
    fn argv_has_size_limits_and_no_nul_bytes() {
        let many = vec!["\"{poc}\""; MAX_ARGV + 1].join(", ");
        let text = with(CLI, "[\"--check\", \"{poc}\"]", &format!("[{many}]"));
        assert_eq!(invalid_field(&text), "poc.argv");
        let exactly = vec!["\"{poc}\""; MAX_ARGV].join(", ");
        assert!(
            parse(&with(
                CLI,
                "[\"--check\", \"{poc}\"]",
                &format!("[{exactly}]")
            ))
            .is_ok()
        );

        // 4096 bytes is fine, 4097 is not; multi-byte characters count as bytes.
        let at_limit = format!("{{poc}}{}", "a".repeat(MAX_ARG_BYTES - 5));
        assert!(parse(&with(CLI, "\"--check\"", &format!("\"{at_limit}\""))).is_ok());
        let over = "é".repeat(MAX_ARG_BYTES / 2 + 1);
        assert_eq!(
            invalid_field(&with(CLI, "\"--check\"", &format!("\"{over}\""))),
            "poc.argv"
        );

        let nul = with(CLI, "\"--check\"", "\"--check\\u0000\"");
        assert_eq!(invalid_field(&nul), "poc.argv");
    }

    #[test]
    fn harness_requires_lang_and_forbids_cli_fields() {
        assert_eq!(
            missing_field(&with(HARNESS, "lang = \"c\"\n", "")),
            "poc.lang"
        );
        assert_eq!(
            invalid_field(&with(HARNESS, "lang = \"c\"", "lang = \"rust\"")),
            "poc.lang"
        );
        let binary = with(HARNESS, "lang = \"c\"", "lang = \"c\"\nbinary = \"tool\"");
        assert_eq!(forbidden_field(&binary), "poc.binary");
    }

    // claim and provenance

    #[test]
    fn claim_kind_must_be_a_bug_kind() {
        let text = with(
            BASE,
            "kind = \"heap-buffer-overflow\"",
            "kind = \"remote-code-execution\"",
        );
        assert_eq!(invalid_field(&text), "claim.kind");
        let other = with(BASE, "kind = \"heap-buffer-overflow\"", "kind = \"other\"");
        assert_eq!(
            parse(&other).unwrap().claim.kind,
            Some(BugKind::Other(String::new()))
        );
    }

    #[test]
    fn claim_access_must_be_read_or_write() {
        assert_eq!(
            invalid_field(&with(BASE, "access = \"write\"", "access = \"execute\"")),
            "claim.access"
        );
    }

    #[test]
    fn free_text_is_limited_and_not_empty() {
        let long = "f".repeat(MAX_TEXT_CHARS + 1);
        assert_eq!(
            invalid_field(&with(
                BASE,
                "function = \"parse_chunk\"",
                &format!("function = \"{long}\"")
            )),
            "claim.function"
        );
        assert_eq!(
            invalid_field(&with(BASE, "function = \"parse_chunk\"", "function = \"\"")),
            "claim.function"
        );
        let at_limit = "ñ".repeat(MAX_TEXT_CHARS);
        assert!(
            parse(&with(
                BASE,
                "file = \"parser.c\"",
                &format!("file = \"{at_limit}\"")
            ))
            .is_ok()
        );
        let reporter = format!("{BASE}\n[provenance]\nreporter = \"{long}\"\n");
        assert_eq!(invalid_field(&reporter), "provenance.reporter");
    }

    #[test]
    fn provenance_found_with_must_be_known() {
        let ok = format!("{BASE}\n[provenance]\nfound_with = \"llm_agent\"\nmodel = \"m\"\n");
        assert_eq!(
            parse(&ok).unwrap().provenance.unwrap().found_with,
            Some(FoundWith::LlmAgent)
        );
        let bad = format!("{BASE}\n[provenance]\nfound_with = \"magic\"\n");
        assert_eq!(invalid_field(&bad), "provenance.found_with");
    }
}
