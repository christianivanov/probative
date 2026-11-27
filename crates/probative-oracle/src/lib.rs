//! The deterministic oracle of Probative.
//!
//! This crate will parse AddressSanitizer, UndefinedBehaviorSanitizer and
//! libFuzzer output, classify stack frames, compute deduplication signatures and
//! turn the results of three runs into a decision. It is the only code that can
//! create a decision.
//!
//! The `Decision` type itself, with private fields and no `Deserialize`, is
//! added together with the decision table.

use probative_core::{Access, BugKind, Sanitizer, Sha256};
use serde::{Deserialize, Serialize};

/// Where a stack frame's code comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameClass {
    /// The project's own code, inside its source directory.
    Project,
    /// The fuzzing harness entry points (`LLVMFuzzerTestOneInput`, `LLVMFuzzerInitialize`).
    Harness,
    /// Code outside the project, or in a vendored dependency directory.
    Dependency,
    /// Sanitizer, fuzzer and C/C++ runtime code.
    Runtime,
    /// A frame without symbols.
    Unknown,
}

/// One stack frame from a sanitizer report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Position in the stack, `#0` being the innermost.
    pub index: u32,
    /// The function name, demangled and normalized.
    pub function: String,
    /// The source file, if symbolized.
    pub file: Option<String>,
    /// The source line, if symbolized.
    pub line: Option<u32>,
    /// Where the frame's code comes from.
    pub class: FrameClass,
}

/// A memory or undefined-behavior error reported by a sanitizer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizerFinding {
    /// The sanitizer that reported it.
    pub sanitizer: Sanitizer,
    /// The kind of bug.
    pub kind: BugKind,
    /// Whether the invalid access read or wrote, when the sanitizer says.
    pub access: Option<Access>,
    /// The frames of the first stack after the error line.
    pub frames: Vec<Frame>,
    /// The sanitizer's `SUMMARY:` line.
    pub summary_line: String,
}

/// A libFuzzer event, used only when there is no sanitizer report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibFuzzerEvent {
    /// `ERROR: libFuzzer: timeout`
    Timeout,
    /// `ERROR: libFuzzer: out-of-memory`
    OutOfMemory,
    /// `ERROR: libFuzzer: deadly signal`
    DeadlySignal,
}

/// What the oracle found in one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunAnalysis {
    /// The sanitizer finding, if any.
    pub finding: Option<SanitizerFinding>,
    /// The dedup signature, if the finding has project frames.
    pub signature: Option<Sha256>,
    /// A libFuzzer event, if there was no sanitizer report.
    pub event: Option<LibFuzzerEvent>,
}

/// The outcome of a verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The sanitizer reported the same bug in project code in at least two of three runs.
    Reproduced,
    /// Three clean runs.
    NotReproduced,
    /// Anything in between; see [`InconclusiveReason`].
    Inconclusive,
}

/// Why a verification is inconclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InconclusiveReason {
    /// The target could not be built.
    BuildFailed,
    /// Findings differed between runs, or appeared in only one.
    Flaky,
    /// The crash has no frames in project code.
    CrashInDependency,
    /// The process died without a sanitizer report.
    NoSanitizerReport,
    /// The stack has no symbols.
    Unsymbolized,
    /// A run timed out without a finding.
    Timeout,
    /// A run hit a memory or process limit without a finding.
    ResourceLimit,
}

/// How the finding compares to what the report claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimMatch {
    /// Every claimed field matches.
    Matches,
    /// Reproduced, but some claimed field differs.
    Different,
    /// Nothing claimed, or not reproduced.
    Unknown,
}

/// A severity hint from a fixed heuristic, never from a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Null dereference, stack overflow, arithmetic undefined behavior and others.
    Low,
    /// Out-of-bounds reads and stack-use-after-return.
    Medium,
    /// Out-of-bounds writes, use-after-free, double-free and invalid-free.
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string as json;

    #[test]
    fn enums_serialize_in_snake_case_as_in_the_verdict_schema() {
        assert_eq!(json(&Outcome::NotReproduced).unwrap(), "\"not_reproduced\"");
        assert_eq!(
            json(&InconclusiveReason::CrashInDependency).unwrap(),
            "\"crash_in_dependency\""
        );
        assert_eq!(
            json(&InconclusiveReason::BuildFailed).unwrap(),
            "\"build_failed\""
        );
        assert_eq!(
            json(&LibFuzzerEvent::OutOfMemory).unwrap(),
            "\"out_of_memory\""
        );
        assert_eq!(json(&ClaimMatch::Different).unwrap(), "\"different\"");
        assert_eq!(json(&Severity::High).unwrap(), "\"high\"");
        assert_eq!(json(&FrameClass::Dependency).unwrap(), "\"dependency\"");
    }

    #[test]
    fn findings_use_the_kebab_case_bug_kind() {
        let finding = SanitizerFinding {
            sanitizer: Sanitizer::Address,
            kind: BugKind::HeapBufferOverflow,
            access: Some(Access::Write),
            frames: vec![],
            summary_line: String::new(),
        };
        let value = serde_json::to_value(&finding).unwrap();
        assert_eq!(value["kind"], "heap-buffer-overflow");
        assert_eq!(value["sanitizer"], "address");
        assert_eq!(value["access"], "write");
    }
}
