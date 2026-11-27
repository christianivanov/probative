# Security policy

Probative executes untrusted proofs of concept, so we take its own security seriously.

## Reporting a vulnerability in Probative

Please report vulnerabilities **privately** through GitHub: open the repository's **Security** tab and choose **Report a vulnerability**. Do not open a public issue or pull request.

Useful details include the affected version or commit, your host setup (OS, container runtime), steps to reproduce, and the impact you observed. We will acknowledge your report, keep you updated while we work on a fix, and credit you in the advisory unless you prefer otherwise.

Examples of what we consider in scope:

- Escaping the sandbox, or a PoC affecting the host, the network or other cases.
- Forging, altering or replaying a verdict so that it still verifies as genuine.
- Leaking report contents, PoCs or signing keys through logs, files or network traffic.
- Path traversal or resource exhaustion through a manifest, PoC or recipe.
- Text in a report changing a verdict or a reviewed manifest.

## Not a reporting channel for other projects

Probative is a tool for verifying vulnerability reports. **It is not a place to report vulnerabilities in other software.** Please send those to the affected project through its own security policy.

## Supported versions

Probative has not been released yet. Until the first release, fixes land on the `main` branch only.
