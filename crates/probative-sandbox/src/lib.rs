//! Sandbox backends for Probative.
//!
//! This crate will define the `SandboxBackend` trait and its Docker
//! implementation. Proofs of concept only ever run inside a container with no
//! network, no secrets, a read-only root filesystem and strict resource limits.

use serde::{Deserialize, Serialize};

/// What happened in one run of the PoC.
///
/// Only the tail of stderr is kept, because the sanitizer report is at the end
/// of the output; stdout is only counted.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRecord {
    /// The process exit code, if the process exited normally.
    pub exit_code: Option<i64>,
    /// Wall-clock duration of the run, in milliseconds.
    pub wall_ms: u64,
    /// Whether the container was killed for exceeding its memory limit.
    pub oom_killed: bool,
    /// Whether the run was killed for exceeding its time limit.
    pub timed_out: bool,
    /// The last bytes of stderr, up to the configured output limit.
    pub stderr_tail: Vec<u8>,
    /// Whether stderr was longer than the kept tail.
    pub stderr_truncated: bool,
    /// Total bytes written to stdout.
    pub stdout_bytes: u64,
}

impl std::fmt::Debug for RunRecord {
    /// Shows the size of stderr, not its contents, which may echo the PoC.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunRecord")
            .field("exit_code", &self.exit_code)
            .field("wall_ms", &self.wall_ms)
            .field("oom_killed", &self.oom_killed)
            .field("timed_out", &self.timed_out)
            .field("stderr_tail_len", &self.stderr_tail.len())
            .field("stderr_truncated", &self.stderr_truncated)
            .field("stdout_bytes", &self.stdout_bytes)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_does_not_include_stderr_contents() {
        let record = RunRecord {
            exit_code: Some(1),
            wall_ms: 10,
            oom_killed: false,
            timed_out: false,
            stderr_tail: b"echoed PoC contents".to_vec(),
            stderr_truncated: false,
            stdout_bytes: 0,
        };
        assert!(!format!("{record:?}").contains("echoed PoC contents"));
    }
}
