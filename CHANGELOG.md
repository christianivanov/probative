# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project will adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it is released.

## [Unreleased]

### Added

- Manifest parser support for `network` PoCs (`role`, `lang`, `binary`, `argv` with `{port}`), with tests for every rule.
- Domain types and manifest parsing in `probative-core`: validated values (`GitSha`, `Sha256`, `RepoUrl`, `RelPath`), the typed manifest, one test per validation rule, and `load_manifest`, which checks the allowlist, resolves symlinks, enforces the PoC size limit and hash, and reads the PoC exactly once. Oracle data types and `RunRecord` with the schema's serialization names. Every example manifest is cross-checked against the parser.
- `network` PoC type in the manifest and verdict schemas (`role`, `lang`, `binary` and an `argv` with `{port}`), with examples; network PoCs are compiled C or C++ for now. `docs/BENCHMARK.md` describes how the public benchmark is measured and run. Decisions recorded: Sigstore-signed releases with provenance attestations, the GitHub Action only publishes the verdict and summary by default, and Firecracker is required for any use beyond the operator's own machine.
- Spec v0.3, second part: network PoCs that talk to the project over loopback only (proposed design, threat model T16 and C19); verdicts as a public contract with a third-party verification guide planned; single-binary distribution with signed releases (T17 and C20); a backend-neutral sandbox trait with Firecracker required before running untrusted PoCs at scale (ADR-001); a public benchmark on hundreds of ARVO cases; and ADR-008 on extending Probative to other languages and oracles.
- Operator-local manual recipes: same isolated build container, only the origin changes; never from a manifest or a report. Verdicts record `recipe_source` and `recipe_sha256` (schema, examples, ADR-006 and threat model C8 updated).
- Roadmap: first end-to-end reproduction with ARVO images; Markdown summaries and `probative init` in the Verdict phase.
- JSON Schemas for the manifest (`docs/manifest.schema.json`) and the verdict (`docs/verdict.schema.json`), with 10 valid and 49 invalid examples in `docs/examples/`. A test checks each invalid example is rejected at the expected location.
- Architecture decision records ADR-001 to ADR-007 in `docs/adr/`: sandbox backend, deterministic oracle, verdict format, LLM outside the verdict path, license, recipes as trusted configuration and PoC injection through a derived image.
- Threat model (`docs/THREAT_MODEL.md`): trust zones, assets, adversaries, 15 threats each linked to its controls, 18 controls with where they are enforced and how they are tested, and three open questions.
- Cargo workspace (edition 2024, resolver 3) with the crates `probative-cli`, `probative-core`, `probative-build`, `probative-sandbox`, `probative-oracle`, `probative-verdict`, `probative-extract` and `probative-mcp`, all empty for now.
- Workspace lints: `unsafe` code forbidden; `unwrap()`, `expect()`, `dbg!` and printing denied outside tests; public items must be documented.
- `deny.toml` for cargo-deny: license allow list, security advisories, banned sources and duplicate versions.
- GitHub Actions CI (format, clippy, tests, docs, release build and cargo-deny) with actions pinned by commit SHA, plus Dependabot for those actions.
- Project documents: README, contributing guide, Contributor License Agreement, security policy, pull request template and `CLAUDE.md` for AI coding agents. Code contributions are not accepted yet.

### Changed

- README redesigned as the project's front page: an illustrative `verify` terminal capture, a trust-zone diagram for light and dark themes (both SVG in `docs/assets/`), a by-hand versus Probative comparison, the three outcomes with their exit codes, the four PoC types, a sample issue summary, a roadmap table and an FAQ.
