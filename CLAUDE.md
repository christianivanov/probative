# CLAUDE.md

Guidance for Claude Code and other AI coding agents working in this repository. Human contributors should read [CONTRIBUTING.md](CONTRIBUTING.md); the rules are the same.

## Project

Probative verifies memory-safety vulnerability reports for C and C++ projects. It builds the affected commit with a sanitizer (ASan or UBSan), runs the reporter's proof of concept (PoC) three times in a locked-down container, lets a deterministic oracle decide the outcome, and emits a signed, canonical JSON verdict that anyone can re-run.

Current phase: **design (F0)**. There is no execution, build or Docker logic yet.

## Layout

| Path | Contents |
| --- | --- |
| `crates/probative-cli` | `probative` binary |
| `crates/probative-core` | domain types, manifest, configuration, shared errors |
| `crates/probative-build` | `oss_fuzz`, `arvo` and `manual` recipes; build-image cache |
| `crates/probative-sandbox` | `SandboxBackend` trait and `DockerBackend` |
| `crates/probative-oracle` | sanitizer parser, frame classification, signatures, decision table, severity |
| `crates/probative-verdict` | verdicts, JCS, Ed25519 signing, store, dedup |
| `crates/probative-extract` | optional LLM-assisted manifest drafting (never used by `verify`) |
| `crates/probative-mcp` | MCP server |

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace -- --ignored   # tests that need Docker
cargo deny check
```

## Conventions

- Stable Rust, edition 2024. `cargo fmt` and `clippy -D warnings` must pass.
- No `unwrap()` or `expect()` outside tests (enforced by workspace lints). `thiserror` in libraries, `anyhow` in the CLI.
- Log with `tracing`. No `println!` outside the CLI output module (enforced by workspace lints).
- Before using a crate's API, check docs.rs **for the version pinned in `Cargo.toml`**. Several dependencies have recent major versions; do not write APIs from memory.
- Do not add dependencies without justifying them in the PR. Licenses must be in the `deny.toml` allow list.
- Code, comments, README, ADRs and commit messages in English.
- While the maintainer is the only developer, work directly on `main`: one Conventional Commit per task, with the decisions taken (anything the spec left open) in the commit body and reported to the maintainer. Run all CI checks locally before every push.
- Every task updates `CHANGELOG.md`; no `TODO` without an issue.

## Security rules

- Never run PoCs or build scripts on the host. Never mount the Docker socket or use `--privileged`.
- Any change to sandbox settings, a recipe, the oracle or the verdict format updates [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) in the same commit, keeping every threat linked to its controls.
- Never log report text or PoC bytes above `debug`.
- Defensive use only: never generate or extend exploits or PoCs. Fixtures use minimal synthetic bugs.
- Only the oracle can create a `Decision`; nothing else, and certainly no LLM, may construct or deserialize one.

## Ask before

Never decide these on your own; ask the maintainer:

- Changing the license, or accepting a dependency whose license is not on the allow list.
- Relaxing any sandbox setting or limit, or giving the run container network access.
- Accepting recipes that come from a report.
- Enabling the LLM by default or sending reports to an LLM provider.
- Changing the oracle decision table or the signature rules.
- Publishing to crates.io, creating a GitHub organization, or declaring version 1.0 of the manifest format.
