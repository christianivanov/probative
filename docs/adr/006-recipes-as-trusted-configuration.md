# ADR-006: Build recipes are trusted configuration

- **Status:** Proposed
- **Date:** 2026-11-27 (operator-local recipes added the same day)

## Context

To reproduce a report, Probative must build the affected commit with a sanitizer. How to build a project is code: it runs commands with network access. If a report could describe its own build, a reporter could run anything in the build zone. Builds of old commits also tend to fail because dependencies drift over time, so historical cases need prebuilt images.

## Decision

A **recipe** describes how to produce a build image with instrumented binaries in `/out`, `llvm-symbolizer` on the `PATH` and a `/out/probative-build.json` file recording the recipe, commit, sanitizer, flags, source directory and digests. Recipes are trusted configuration. They come from Probative's own `recipes/` directory or from the operator's local recipe directory, and **never from a manifest or a report**. There are three kinds:

- **`oss_fuzz`** for projects integrated in [OSS-Fuzz](https://github.com/google/oss-fuzz). Probative checks out a pinned OSS-Fuzz commit, builds the project image from `projects/<name>/Dockerfile`, checks out the target commit inside the build container and runs `compile` with `SANITIZER`, `FUZZING_ENGINE=libfuzzer` and `ARCHITECTURE=x86_64`. It does **not** use `infra/helper.py`, because its `docker_run` function starts every container with `--privileged` (`infra/helper.py`, line 662 at OSS-Fuzz commit [`13df93cc`](https://github.com/google/oss-fuzz/blob/13df93cc67be7b3e0208a65cd82e1d9a875aee44/infra/helper.py#L662)). Base images are published under `gcr.io/oss-fuzz-base` (for example `base-builder`), and Probative records their digest in every verdict.
- **`arvo`** for historical cases and evaluation. [ARVO](https://github.com/n132/arvo) publishes images with each vulnerability already built, in a vulnerable (`<id>-vul`) and a fixed (`<id>-fix`) version. Probative uses them as build images, pinned by digest, but always runs the PoC in its own sandbox rather than with ARVO's `arvo` command. Where the binary and the PoC live inside these images will be documented here when the recipe is implemented.
- **`manual`** for CLI PoCs and projects outside OSS-Fuzz. A reviewed TOML file in Probative's `recipes/` directory declares a base image by digest, packages, environment, build steps and the binaries a PoC may invoke. Probative adds the sanitizer flags itself; recipes do not define them.
- **Operator-local recipes.** A manual recipe can also live in the operator's local recipe directory (`[recipes] local_dir`), so maintainers can describe their own project without waiting for a review here. Only the origin changes: local recipes run in exactly the same isolated build container, with the same limits. The manifest can only name a recipe, never give a path or its contents. A name that exists both here and locally is a policy error, the directory must not be writable by other users, and the verdict records `recipe_source` and the recipe's SHA-256 so that a re-run can detect a missing or different recipe.

In every case, the build container has network access but no secrets, no Docker socket and no host mounts, and never runs the PoC. Builds are cached under a hash of all their inputs.

## Alternatives considered

- **Use `infra/helper.py`.** Simplest, but runs privileged containers. Rejected.
- **Let the manifest describe the build.** Flexible, but lets a reporter run arbitrary build commands. Rejected.
- **Only recipes from this repository.** Easiest to review, but manual recipes would only scale with our review capacity. Operator-local recipes keep the same isolation while letting each maintainer write the recipe for their own project once.
- **Only OSS-Fuzz.** Covers many projects, but not historical commits whose dependencies no longer build, nor CLI tools outside OSS-Fuzz.
- **Build old commits from source with pinned snapshots.** Possible in theory, but ARVO already solved it for thousands of cases.

## Consequences

- Adding a project outside OSS-Fuzz requires a recipe: a reviewed one in this repository, or a local one written by the operator. A useful local recipe can be proposed here.
- Every change to a recipe or to the build container updates the [threat model](../THREAT_MODEL.md) ([T2](../THREAT_MODEL.md#threats), [C7](../THREAT_MODEL.md#c7-isolated-build-zone), [C8](../THREAT_MODEL.md#c8-recipes-are-trusted-configuration)).
- Build scripts still run with network access, and the allowlist alone does not guarantee that a commit was reviewed upstream. Both gaps are recorded as [Q2](../THREAT_MODEL.md#q2-commits-that-only-exist-in-a-fork) and [Q3](../THREAT_MODEL.md#q3-network-access-during-builds).
- ARVO images are third-party content. They are pinned by digest, and the dataset's license is checked before any PoC from it is redistributed.
