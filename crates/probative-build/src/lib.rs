//! Build recipes for Probative.
//!
//! This crate will resolve a recipe (`oss_fuzz`, `arvo` or `manual`) into a
//! build plan that produces a sanitizer-instrumented build image, and manage
//! the cache of those images.
