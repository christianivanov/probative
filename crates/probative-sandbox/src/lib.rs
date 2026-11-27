//! Sandbox backends for Probative.
//!
//! This crate will define the `SandboxBackend` trait and its Docker
//! implementation. Proofs of concept only ever run inside a container with no
//! network, no secrets, a read-only root filesystem and strict resource limits.
