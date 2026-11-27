//! Domain enums shared across Probative: sanitizers, bug kinds, access types
//! and harness languages.
//!
//! Serialization follows the published schemas: `BugKind` uses the kebab-case
//! names AddressSanitizer prints (`heap-buffer-overflow`); every other enum
//! uses snake_case.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ValueError;

/// The sanitizer a build is instrumented with. One per build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sanitizer {
    /// AddressSanitizer (the default).
    #[default]
    Address,
    /// UndefinedBehaviorSanitizer.
    Undefined,
}

/// Whether an invalid memory access read or wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    /// A read.
    Read,
    /// A write.
    Write,
}

/// Language of a harness PoC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    /// C.
    C,
    /// C++.
    Cpp,
}

/// The role a network PoC plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetRole {
    /// The PoC listens and the project's binary connects to it.
    Server,
    /// The project's binary listens and the PoC connects to it.
    Client,
}

/// How the reporter found the bug. Informative only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundWith {
    /// A fuzzer.
    Fuzzing,
    /// An LLM-based agent.
    LlmAgent,
    /// Manual review or testing.
    Manual,
    /// Anything else.
    Other,
}

/// The kind of bug a sanitizer reports, named as AddressSanitizer names it.
///
/// `Other` carries the sanitizer's own description when there is one. It
/// always serializes as `"other"`; the original text stays in the verdict's
/// `summary_line`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BugKind {
    /// `heap-buffer-overflow`
    HeapBufferOverflow,
    /// `stack-buffer-overflow`
    StackBufferOverflow,
    /// `global-buffer-overflow`
    GlobalBufferOverflow,
    /// `heap-use-after-free`
    HeapUseAfterFree,
    /// `stack-use-after-return`
    StackUseAfterReturn,
    /// `double-free`
    DoubleFree,
    /// `invalid-free`
    InvalidFree,
    /// `stack-overflow`
    StackOverflow,
    /// `null-deref`
    NullDeref,
    /// `segv-unknown-address`
    SegvUnknownAddress,
    /// `signed-integer-overflow`
    SignedIntegerOverflow,
    /// `shift`
    Shift,
    /// `division-by-zero`
    DivisionByZero,
    /// `out-of-bounds-index`
    OutOfBoundsIndex,
    /// `misaligned-access`
    MisalignedAccess,
    /// `other`, with the sanitizer's description if known.
    Other(String),
}

const BUG_KINDS: [(&str, BugKind); 15] = [
    ("heap-buffer-overflow", BugKind::HeapBufferOverflow),
    ("stack-buffer-overflow", BugKind::StackBufferOverflow),
    ("global-buffer-overflow", BugKind::GlobalBufferOverflow),
    ("heap-use-after-free", BugKind::HeapUseAfterFree),
    ("stack-use-after-return", BugKind::StackUseAfterReturn),
    ("double-free", BugKind::DoubleFree),
    ("invalid-free", BugKind::InvalidFree),
    ("stack-overflow", BugKind::StackOverflow),
    ("null-deref", BugKind::NullDeref),
    ("segv-unknown-address", BugKind::SegvUnknownAddress),
    ("signed-integer-overflow", BugKind::SignedIntegerOverflow),
    ("shift", BugKind::Shift),
    ("division-by-zero", BugKind::DivisionByZero),
    ("out-of-bounds-index", BugKind::OutOfBoundsIndex),
    ("misaligned-access", BugKind::MisalignedAccess),
];

impl BugKind {
    /// The serialized name of this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HeapBufferOverflow => "heap-buffer-overflow",
            Self::StackBufferOverflow => "stack-buffer-overflow",
            Self::GlobalBufferOverflow => "global-buffer-overflow",
            Self::HeapUseAfterFree => "heap-use-after-free",
            Self::StackUseAfterReturn => "stack-use-after-return",
            Self::DoubleFree => "double-free",
            Self::InvalidFree => "invalid-free",
            Self::StackOverflow => "stack-overflow",
            Self::NullDeref => "null-deref",
            Self::SegvUnknownAddress => "segv-unknown-address",
            Self::SignedIntegerOverflow => "signed-integer-overflow",
            Self::Shift => "shift",
            Self::DivisionByZero => "division-by-zero",
            Self::OutOfBoundsIndex => "out-of-bounds-index",
            Self::MisalignedAccess => "misaligned-access",
            Self::Other(_) => "other",
        }
    }
}

impl FromStr for BugKind {
    type Err = ValueError;

    /// Parses a serialized name. `"other"` becomes `Other` with no description.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "other" {
            return Ok(Self::Other(String::new()));
        }
        BUG_KINDS
            .iter()
            .find(|(name, _)| *name == value)
            .map(|(_, kind)| kind.clone())
            .ok_or_else(|| ValueError::BugKind(value.chars().take(80).collect()))
    }
}

impl fmt::Display for BugKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for BugKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BugKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bug_kind_round_trips_through_its_name() {
        for (name, kind) in BUG_KINDS {
            assert_eq!(kind.as_str(), name);
            assert_eq!(name.parse::<BugKind>().unwrap(), kind);
        }
        assert_eq!(
            "other".parse::<BugKind>().unwrap(),
            BugKind::Other(String::new())
        );
        assert!("remote-code-execution".parse::<BugKind>().is_err());
    }

    #[test]
    fn serialization_names_follow_the_schemas() {
        use serde_json::to_string as json;
        assert_eq!(
            json(&BugKind::HeapUseAfterFree).unwrap(),
            "\"heap-use-after-free\""
        );
        assert_eq!(json(&BugKind::Other("weird".into())).unwrap(), "\"other\"");
        assert_eq!(json(&Sanitizer::Undefined).unwrap(), "\"undefined\"");
        assert_eq!(json(&Access::Write).unwrap(), "\"write\"");
        assert_eq!(json(&Lang::Cpp).unwrap(), "\"cpp\"");
        assert_eq!(json(&FoundWith::LlmAgent).unwrap(), "\"llm_agent\"");
        assert_eq!(json(&NetRole::Client).unwrap(), "\"client\"");
        assert_eq!(Sanitizer::default(), Sanitizer::Address);
    }
}
