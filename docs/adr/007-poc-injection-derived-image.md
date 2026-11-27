# ADR-007: Inject the PoC through a derived image

- **Status:** Proposed
- **Date:** 2026-11-27

## Context

The PoC has to be inside the run container, which has a read-only root filesystem, no bind mounts and no network ([ADR-001](001-sandbox-backend.md)). The obvious tool, `docker cp`, does not work there: the Docker daemon refuses to copy into a container whose `ReadonlyRootfs` is set, whether or not it is running ("container rootfs is marked read-only", in `checkWritablePath`, `daemon/archive_unix.go` in [moby/moby](https://github.com/moby/moby/blob/a63d92884480c7e6e5985264a0d2c8f6ec262957/daemon/archive_unix.go#L183)). The PoC should also be part of what a verdict pins, so that a re-run uses exactly the same bytes.

## Decision

- For each case, Probative builds a small **derived image**: `FROM <build image digest>`, then `COPY` the PoC to `/poc/` with mode `0444`. The build context contains only the PoC bytes, read once and hashed (see [C10](../THREAT_MODEL.md#c10-strict-manifest-and-poc-validation)).
- That image build runs with networking disabled.
- The image is tagged `probative-run:<hash of the build image digest and the PoC SHA-256>`, and its digest is recorded in the verdict as `run_image_digest`.

## Alternatives considered

- **`docker cp`.** Refused by the daemon with a read-only root filesystem.
- **Bind-mounting the PoC.** Bind mounts are forbidden in run containers, because one mistake in the mounted path exposes the host.
- **A named volume filled by a helper container.** Needs an extra writable container, leaves state outside images and gives no digest to pin.
- **Streaming the PoC through stdin.** Only works for programs that read their input from stdin, and not for file arguments.

## Consequences

- The run image digest identifies the build and the PoC together, which makes verdicts easier to reproduce and to audit.
- Each case adds one thin image layer, which `probative cache prune` cleans up.
- The PoC is stored in the local image store. Run images are never pushed to a registry, because PoCs are usually under embargo.
