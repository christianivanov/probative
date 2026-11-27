# ADR-008: Extensibility to other languages and oracles

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

Probative starts with memory-safety bugs in C and C++, because sanitizers give an objective oracle there ([ADR-002](002-deterministic-oracle.md)). Vulnerability reports also arrive for other ecosystems, and the manifest and verdict formats are meant to outlive the first release. Adding an ecosystem later must not require redesigning the sandbox or breaking manifests and verdicts that already exist. At the same time, widening the scope before the first oracle is proven would dilute the one guarantee that matters: zero false `reproduced` verdicts.

## Decision

- **Focus first.** Until the Verdict phase closes with a published benchmark, Probative only covers memory-safety bugs in C and C++. No language or oracle is added before that.
- **Oracles behind a trait.** Interpreting a run goes through an `Oracle` trait that turns a run record and build information into an analysis. The sanitizer oracle for C and C++ is its first implementation. The decision table stays a single function in the oracle crate, which is still the only code that can construct a `Decision`.
- **Recipes and PoC types are open enums.** A new ecosystem adds recipe kinds (how to build with its instrumentation) and, if needed, PoC types. Each addition ships with its own threat-model update.
- **Bug kinds grow additively.** New ecosystems add new `BugKind` values; existing values never change meaning.
- **Format versioning.** While the formats are `0.x`, an additive change (a new recipe, PoC type or bug kind) bumps the minor `schema_version`. Consumers reject versions they do not know rather than guessing. Changing or removing the meaning of an existing field requires a new major version and a new ADR.
- **How an ecosystem is added**, in order: an ADR describing its objective oracle; build recipes with that ecosystem's instrumentation; an `Oracle` implementation and its decision-table tests; real reproducible cases in the evaluation set; and benchmark results published before it is announced as supported.

## First candidate: Rust

Rust is the first candidate after the Verdict phase. Memory-safety bugs in `unsafe` Rust can be detected with the same sanitizers (AddressSanitizer through `-Zsanitizer=address` on nightly) and with Miri, and many Rust projects already fuzz with `cargo-fuzz`, which uses libFuzzer. The open questions for its ADR are which of those signals count as objective evidence, and how panics, which are often not vulnerabilities, are reported.

## Alternatives considered

- **Support several ecosystems from the start.** Broader appeal, but each oracle needs its own evaluation, and none would be proven well.
- **Hard-code C and C++ everywhere.** Simplest today, but every later ecosystem would mean breaking formats and rewriting the sandbox integration.
- **Free-form strings for bug kinds.** Easy to extend, but loses validation and makes deduplication and severity hints ambiguous.

## Consequences

- The first release stays narrow, and its scope limits are stated openly.
- The extension points cost little now: one trait and versioning rules.
- Manifests and verdicts written today remain valid when new ecosystems arrive.
