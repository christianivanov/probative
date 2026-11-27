# Benchmark

Probative's central promise is that a `reproduced` verdict is never wrong. This document describes how that promise is measured in public. **There are no results yet**: the benchmark runs once the Verdict phase is complete, and its results will be published here.

## What is measured

| Metric | How it is reported |
| --- | --- |
| False `reproduced` verdicts | Count, with an upper bound on the true rate. With 0 false verdicts in *n* cases, the 95 % upper bound is about 3/*n* (the "rule of three"): about 1 % for 300 cases. |
| Reproduction rate | Percentage of positive cases reproduced, with a 95 % Wilson confidence interval |
| Correct `not_reproduced` on fixed versions | Percentage, with a 95 % Wilson confidence interval |
| Coverage | Results by PoC type (fuzzer input, CLI, harness, network) and by bug kind |
| `inconclusive` results | Count by reason, so the limits of the tool are visible |
| Cost | Time per case and cache hit rate |

Every report records the Probative version and the digests of every image used.

## Cases

- **At least 300 historical cases** from [ARVO](https://github.com/n132/arvo), stratified by project, bug kind and language (C and C++), each with its vulnerable and fixed versions.
- **Real curl vulnerabilities** with public PoCs, from the scope study in the evaluation plan.
- **Adversarial cases** that test the sandbox: fork bombs, endless output, memory bombs, attempts to reach the network, and network PoCs that try to escape or fake a sanitizer report.

Where licenses do not allow redistributing PoCs, `eval/fetch.sh` downloads them from their source.

## Reproducing it

```sh
eval/fetch.sh
probative eval eval/ --report docs/BENCHMARK.md --prune-images
```

`--prune-images` deletes each case's images once the case is finished, so the disk only has to hold the largest case at a time.

## Where it runs

The benchmark is meant to run on hardware anyone can get:

- **Locally**, on a Linux x86_64 machine, or an Intel Mac with Docker Desktop or Colima. Cases run one at a time with `--prune-images`. With Colima, make sure the virtual machine's disk can hold the largest case image (`colima start --disk <GiB>`). Expect a long run: hundreds of image downloads of several gigabytes each.
- **On a cloud Linux x86_64 virtual machine** rented by the hour, only while the benchmark runs, when a local run is not practical.

Both produce the same report for the same Probative version and image digests.
