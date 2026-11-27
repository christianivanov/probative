# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project will adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it is released.

## [Unreleased]

### Added

- Cargo workspace (edition 2024, resolver 3) with the crates `probative-cli`, `probative-core`, `probative-build`, `probative-sandbox`, `probative-oracle`, `probative-verdict`, `probative-extract` and `probative-mcp`, all empty for now.
- Workspace lints: `unsafe` code forbidden; `unwrap()`, `expect()`, `dbg!` and printing denied outside tests; public items must be documented.
- `deny.toml` for cargo-deny: license allow list, security advisories, banned sources and duplicate versions.
- GitHub Actions CI (format, clippy, tests, docs, release build and cargo-deny) with actions pinned by commit SHA, plus Dependabot for those actions.
- Project documents: README, contributing guide, Contributor License Agreement, security policy, pull request template and `CLAUDE.md` for AI coding agents. Code contributions are not accepted yet.
