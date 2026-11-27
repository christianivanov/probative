# ADR-001: Sandbox backend: Docker first, behind a trait

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

Probative runs untrusted proofs of concept against untrusted builds. It needs strong isolation, but also has to run where maintainers already work: Linux servers and workstations, and Intel Macs during development. The isolation technology will change over time. Containers are available everywhere today, and microVMs give each run its own kernel but need KVM.

## Decision

- All execution goes through a `SandboxBackend` trait with three operations: obtain a build image, derive a run image that contains the PoC, and run it with limits. The rest of Probative never talks to a container runtime directly.
- The first implementation is `DockerBackend`, using the Docker Engine API (through `bollard`). It works with Docker, rootless Docker and Colima, and connects through `DOCKER_HOST` or the platform default.
- Run containers use a fixed hardened configuration: no network, all capabilities dropped, `no-new-privileges`, default seccomp, unprivileged user, read-only root filesystem with a small `tmpfs`, and memory, CPU, process, file and time limits. These settings are constants, not user configuration. See controls [C1 to C5](../THREAT_MODEL.md#controls).
- A Firecracker microVM backend is planned once the oracle is proven, so that each run gets its own kernel.

## Alternatives considered

- **Firecracker from the start.** Strongest isolation, but needs KVM. It does not run on macOS, is awkward in nested virtualization and would delay the first useful version.
- **gVisor (`runsc`) as the default runtime.** A good hardening layer on Linux, but not available on macOS, and it can be adopted later as a Docker runtime without changing Probative.
- **Host sandboxes (nsjail, bubblewrap).** Linux only, and they would make Probative responsible for building root filesystems that Docker already provides.
- **WebAssembly.** Cannot run native C and C++ binaries instrumented with AddressSanitizer.

## Consequences

- Containers share the host kernel, so a kernel exploit in a PoC could escape ([T1](../THREAT_MODEL.md#threats)). Rootless Docker on Linux and a VM on macOS reduce the impact; the microVM backend removes the shared kernel.
- The trait keeps backends swappable and lets tests use a fake backend without Docker.
- Tests that need Docker are marked `#[ignore]` and run separately.
- Only x86_64 hosts are supported for now. arm64 hosts are refused with a clear error rather than emulated.
