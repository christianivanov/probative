# Examples

Example manifests and verdicts, checked in CI against the published schemas:

- [`manifest.schema.json`](../manifest.schema.json) for `probative.toml` manifests
- [`verdict.schema.json`](../verdict.schema.json) for verdicts

| Directory | Contents |
| --- | --- |
| `manifests/valid/` | Manifests for each recipe and PoC type |
| `manifests/invalid/` | Manifests that each break exactly one rule, explained in their first line |
| `verdicts/valid/` | Verdicts for each kind of outcome |
| `verdicts/invalid/` | Verdicts that each break exactly one rule, named after it |
| [`expected-errors.toml`](expected-errors.toml) | Where each invalid example must be rejected, as a JSON pointer |

The test in `crates/probative-core/tests/doc_schemas.rs` checks that every valid example is accepted and that every invalid example is rejected **at the expected location**, so an example cannot pass by failing for an unrelated reason.

Repositories, commits, hashes and digests are illustrative. Verdict signatures have the right format but are placeholders, not real Ed25519 signatures.

Some rules cannot be expressed in JSON Schema and are enforced by Probative itself: symlink resolution of `poc.file`, the PoC size limit and hash, argv lengths in bytes, and the repository allowlist.
