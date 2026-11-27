# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project will adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it is released.

## [Unreleased]

### Added

- JSON Schemas for the manifest (`docs/manifest.schema.json`) and the verdict (`docs/verdict.schema.json`), with 10 valid and 49 invalid examples in `docs/examples/`. A test checks each invalid example is rejected at the expected location.
- Architecture decision records ADR-001 to ADR-007 in `docs/adr/`: sandbox backend, deterministic oracle, verdict format, LLM outside the verdict path, license, recipes as trusted configuration and PoC injection through a derived image.
- Threat model (`docs/THREAT_MODEL.md`): trust zones, assets, adversaries, 15 threats each linked to its controls, 18 controls with where they are enforced and how they are tested, and three open questions.
- Cargo workspace (edition 2024, resolver 3) with the crates `probative-cli`, `probative-core`, `probative-build`, `probative-sandbox`, `probative-oracle`, `probative-verdict`, `probative-extract` and `probative-mcp`, all empty for now.
- Workspace lints: `unsafe` code forbidden; `unwrap()`, `expect()`, `dbg!` and printing denied outside tests; public items must be documented.
- `deny.toml` for cargo-deny: license allow list, security advisories, banned sources and duplicate versions.
- GitHub Actions CI (format, clippy, tests, docs, release build and cargo-deny) with actions pinned by commit SHA, plus Dependabot for those actions.
- Project documents: README, contributing guide, Contributor License Agreement, security policy, pull request template and `CLAUDE.md` for AI coding agents. Code contributions are not accepted yet.
