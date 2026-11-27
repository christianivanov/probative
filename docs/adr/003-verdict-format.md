# ADR-003: Verdict format: canonical JSON, Ed25519 and UUID v7

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

A verdict is evidence that other people rely on: a maintainer, the reporter, or a coordinator who never ran Probative. It must be readable by people and tools, impossible to edit without detection, and checkable offline. Reports are often under embargo, so verifying a verdict must not require publishing anything.

## Decision

- A verdict is a JSON document with a published schema. It records the outcome and its reason, the finding, the commit, the build and run image digests, the PoC hash, each run, the fix check, the dedup signature and the command to reproduce it.
- The signed bytes are the **JSON Canonicalization Scheme** ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)) serialization of the verdict without its `signature` field. Any tool that implements JCS can recompute them.
- Verdicts are signed with **Ed25519**. The signature block holds the algorithm, the public key, a key ID (the first 16 hex characters of the SHA-256 of the public key) and the signature, in base64.
- Verdict IDs are **UUID v7**, which sort by creation time, and timestamps are RFC 3339 in UTC.
- `probative check` verifies the signature and says whether the key is trusted locally. `check --rerun` reproduces the case and compares the outcome and dedup signature, not timestamps.

## Alternatives considered

- **JWS or JWT.** The payload becomes an opaque base64 string, which hurts readability, and the many algorithm options have a history of verification mistakes.
- **Sigstore keyless signing with a transparency log.** Attractive for public artifacts, but recording an entry in a public log for every verdict conflicts with embargoed reports, and it requires an online identity provider.
- **OpenPGP.** Complex key handling and tooling for a single, fixed use case.
- **COSE or CBOR.** Compact, but not human-readable, and harder to inspect in an issue or an email.
- **UUID v4.** Works, but loses time ordering in the verdict store and the index.

## Consequences

- Verdicts are plain JSON that anyone can read and attach anywhere, and they verify offline.
- Users manage their own keys. Trust is explicit: a verdict is trusted only if its key is listed in the local configuration.
- Canonicalization means the JSON can be reformatted freely without breaking the signature, as long as the content is unchanged.
- Wrapping verdicts in an in-toto or DSSE envelope remains possible later for supply-chain tooling.
