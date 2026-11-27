# Contributing to Probative

Thanks for your interest in Probative. The project is in its early design phase, so the most useful contributions right now are **feedback on the design**, **real-world sanitizer output** we can use as test fixtures, and **issues** describing how you triage vulnerability reports today.

> [!NOTE]
> **We are not accepting code contributions yet.** Issues, questions and design feedback are very welcome. This guide describes how pull requests will work once we open them.

## Ground rules

Probative runs untrusted code by design, so a few rules are not negotiable:

- **Defensive use only.** Probative runs the proof of concept that comes with a report. It never generates, extends or weaponizes exploits, and contributions that do will be declined. Test fixtures use minimal, synthetic bugs.
- **Never run a PoC or a build script on the host.** All untrusted code runs inside the sandbox. Never mount the Docker socket into a container and never use `--privileged`.
- **No LLM in the verdict path.** Only the deterministic oracle decides a verdict.
- **Never log report text or PoC bytes** above the `debug` level. Reports are often under embargo.
- **Sandbox and recipe changes update the [threat model](docs/THREAT_MODEL.md)** in the same pull request.

## Development setup

You need a stable Rust toolchain (edition 2024) and [cargo-deny](https://github.com/EmbarkStudios/cargo-deny):

```sh
rustup toolchain install stable --component rustfmt clippy
cargo install --locked cargo-deny
```

Run the same checks as CI before opening a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check
```

Tests that need Docker are marked `#[ignore]` and run with `cargo test -- --ignored`.

## Code conventions

- `cargo fmt` and `clippy -D warnings` are enforced. The workspace lints forbid `unsafe` code and deny `unwrap()`, `expect()` and printing outside tests.
- Libraries use `thiserror` for errors; the CLI uses `anyhow`. A tool error (for example, Docker being unavailable) is never turned into a verdict.
- Log with `tracing`. Only the CLI output module writes to stdout.
- New dependencies must be justified in the pull request and use a license allowed by [`deny.toml`](deny.toml).
- Code, comments, documentation and commit messages are in English.

## Pull requests

- One logical change per pull request.
- Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages (`feat:`, `fix:`, `docs:`, `chore:`…).
- Fill in the pull request template, including a **Decisions taken** section for any choice the issue or design docs did not cover.
- Add an entry to [`CHANGELOG.md`](CHANGELOG.md) under "Unreleased".
- Leave no `TODO` without a linked issue.

## Contributor License Agreement

All contributions require agreeing to the [Contributor License Agreement](CLA.md). You keep the copyright to your work; the CLA grants the project the rights it needs to distribute it. Tick the CLA box in the pull request template to agree.

## Reporting security issues

Please do not open public issues for vulnerabilities in Probative itself. See [SECURITY.md](SECURITY.md).
