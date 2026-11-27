# ADR-005: License: AGPL-3.0-only with a CLA

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

Probative is a self-hosted tool that is also meant to be run as a network service, for example inside a security team's infrastructure. Its license must keep the project free software, ensure that improvements to deployed versions come back to the community, and be compatible with the dependency ecosystem it builds on.

## Decision

- Probative is licensed under the **GNU Affero General Public License v3.0 only** (`AGPL-3.0-only`).
- Contributions require agreeing to a **Contributor License Agreement** ([CLA.md](../../CLA.md)), adapted from the Apache Individual CLA. Contributors keep the copyright to their work.
- The CLA allows the maintainer to relicense contributions, including under commercial terms, while the project remains available under AGPL-3.0-only.
- Third-party dependencies must use licenses compatible with AGPL-3.0-only. The allow list is enforced by `cargo-deny` in CI ([`deny.toml`](../../deny.toml)): MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0 and MPL-2.0.

## Alternatives considered

- **MIT or Apache-2.0.** Maximize adoption, but allow modified versions to be offered as a network service without sharing the changes.
- **GPL-3.0.** Copyleft, but does not cover use over a network.
- **AGPL-3.0-or-later.** Lets the Free Software Foundation's future versions apply automatically. `-only` keeps the licensing terms under the project's control.
- **Developer Certificate of Origin (DCO) instead of a CLA.** Lighter for contributors, but it does not grant the rights needed to relicense contributions.

## Consequences

- Anyone may use, modify and self-host Probative under the AGPL. Anyone who offers a modified version over a network must provide its source.
- Some contributors prefer not to sign CLAs. This is accepted as a trade-off.
- Code contributions are not accepted until the CLA process is in place. Issues and design feedback are welcome now.
- Adding a license to the allow list, or changing the project license, requires the maintainer's approval and a new ADR.
