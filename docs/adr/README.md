# Architecture decision records

Each record captures one significant decision: its context, the decision, the alternatives considered and the consequences. Records are numbered and never deleted. A decision that changes is superseded by a new record that links back to it.

| ADR | Title | Status |
| --- | --- | --- |
| [001](001-sandbox-backend.md) | Sandbox backend: Docker first, behind a trait | Proposed |
| [002](002-deterministic-oracle.md) | A deterministic oracle based on sanitizers | Proposed |
| [003](003-verdict-format.md) | Verdict format: canonical JSON, Ed25519 and UUID v7 | Proposed |
| [004](004-llm-outside-verify.md) | Keep the LLM outside the verdict path | Proposed |
| [005](005-license.md) | License: AGPL-3.0-only with a CLA | Proposed |
| [006](006-recipes-as-trusted-configuration.md) | Build recipes are trusted configuration | Proposed |
| [007](007-poc-injection-derived-image.md) | Inject the PoC through a derived image | Proposed |
| [008](008-extensibility.md) | Extensibility to other languages and oracles | Proposed |

**Statuses:** Proposed (under review), Accepted, Superseded (replaced by a later record).

**Template:** title, status and date, then the sections Context, Decision, Alternatives considered and Consequences. Security-relevant records link to the [threat model](../THREAT_MODEL.md).
