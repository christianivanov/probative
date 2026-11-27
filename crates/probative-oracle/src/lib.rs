//! The deterministic oracle of Probative.
//!
//! This crate will parse AddressSanitizer, UndefinedBehaviorSanitizer and
//! libFuzzer output, classify stack frames, compute deduplication signatures and
//! turn the results of three runs into a decision. It is the only code that can
//! create a decision.
