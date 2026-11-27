# ADR-004: Keep the LLM outside the verdict path

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

Writing a manifest by hand from a free-text report takes time, and language models are good at extracting structured fields from prose. But report text is untrusted and may contain instructions aimed at a model (prompt injection), reports are often under embargo, and a verdict must never depend on a model's judgement.

## Decision

- `probative verify` **never calls a language model**. Verdicts come only from the deterministic oracle ([ADR-002](002-deterministic-oracle.md)).
- An optional command, `probative extract`, drafts a `probative.toml` from a report. A human reviews the draft before running `verify`.
- The LLM integration is **disabled by default**. Enabling it is an explicit configuration choice, and the CLI warns that the report leaves the machine unless the endpoint is local.
- Requests go through a configurable gateway (`[llm] endpoint`) that holds the provider credentials, so Probative never stores them.
- The prompt is fixed in code, with the report passed as delimited data and temperature 0. The answer must be JSON that matches a schema, with length limits. Invalid fields, and repositories outside the allowlist, are left empty in the draft for a human to fill in. The model never supplies PoC bytes.
- `Decision` has private fields, does not implement `Deserialize`, and can only be created by the oracle. The verdict builder only accepts a `Decision`. A `compile_fail` doctest proves that no other crate can build one.

## Alternatives considered

- **LLM triage inside `verify`.** Makes verdicts depend on a non-deterministic, injectable component. Rejected.
- **No LLM support at all.** Safe, but leaves manifest writing as manual toil. Keeping it optional and outside the verdict path gives the benefit without the risk.
- **Enabled by default.** Would send embargoed reports to a third party without an explicit decision by the user. Rejected.

## Consequences

- Prompt injection can at worst produce a wrong draft, which a human reviews and which `verify` validates like any other manifest ([T4](../THREAT_MODEL.md#threats), [C15](../THREAT_MODEL.md#c15-llm-outside-the-verdict-path)).
- The type system, not only code review, enforces that no model output becomes a verdict.
- An evaluation group of reports with injected instructions checks that drafts never gain fields or change outcomes.
