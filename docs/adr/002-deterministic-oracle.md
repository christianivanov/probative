# ADR-002: A deterministic oracle based on sanitizers

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

The value of a verdict depends on how it is decided. A crash alone proves little: programs exit with errors on bad input all the time, and many crashes are in dependencies or are not reproducible. Language models can describe a crash but cannot be relied on to classify it or to tell a bug from a vulnerability. For memory-safety bugs in C and C++, sanitizers give an objective signal: an instrumented build either reports an invalid memory access at a specific location or it does not.

## Decision

- The verdict is decided only by deterministic code that interprets AddressSanitizer or UndefinedBehaviorSanitizer output (one sanitizer per build), plus libFuzzer events when there is no sanitizer report.
- Every case runs **three times**. The outcome is `reproduced` only if at least two runs produce a finding with the same signature and at least one frame in the project's own code.
- Every other situation maps to `inconclusive` with an explicit reason (`build_failed`, `crash_in_dependency`, `unsymbolized`, `flaky`, `timeout`, `resource_limit`, `no_sanitizer_report`), or to `not_reproduced` when all three runs are clean. A non-zero exit code alone is never a finding.
- Frames are classified as project, harness, dependency, runtime or unknown. The deduplication signature is a hash of the bug kind, the access type and the first three normalized project frames, so it does not depend on addresses, line numbers, template arguments or process IDs.
- The severity hint is a fixed table based on the bug kind and access type, never a model's opinion.
- The result is a `Decision` value that only the oracle can construct. See [ADR-004](004-llm-outside-verify.md).

## Alternatives considered

- **A single run.** Cheaper, but cannot distinguish flaky crashes from reliable ones.
- **Crash or exit code only, without a sanitizer.** Many memory errors do not crash without instrumentation, and many crashes are not memory errors.
- **LLM judgement of the crash.** Not reproducible, open to prompt injection, and known to misjudge severity.
- **MemorySanitizer.** Requires every dependency to be instrumented, which most builds cannot provide. It may be added later.
- **Several sanitizers in one build.** Some combinations are incompatible and the reports become harder to attribute. One sanitizer per build keeps results clear.

## Consequences

- Scope is limited to bugs that sanitizers detect. Logic bugs need other oracles in the future.
- The decision table is the core guarantee of the project. Changing it, or the signature rules, requires a new ADR.
- When in doubt the outcome is `inconclusive`, which favours zero false `reproduced` verdicts over a higher reproduction rate.
- The oracle trusts that sanitizer output comes from the sanitizer runtime. The threat model records how a PoC could fake it ([T12](../THREAT_MODEL.md#threats)) and the candidate mitigations ([Q1](../THREAT_MODEL.md#q1-fake-sanitizer-reports)), which must be settled before the oracle is implemented.
