# T-00371 — Dependency & Toolchain Pinning / security policy: Research

## 1. Goal
Establish facts, constraints, security threat models, and prior art for the security policy governing Dependency & Toolchain Pinning within AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Code & Specifications):
1. **Toolchain Specification**: `config/toolchain.json`, `rust-toolchain.toml`, and `.python-version` declare pinned versions (`rustc 1.99.0`, `python 3.14`, `node v24.18`).
2. **Read-Only Verification**: The MCP tool `aios.toolchain.check` and CLI command `aiosh toolchain check` execute read-only version inquiries (`-V` / `-v`) against host binaries with a 15-second timeout and process reaping.
3. **Audit Ring Invariant**: Both MCP tool invocations and CLI commands write structured audit entries (`aios.toolchain.check`, `aios.toolchain.config.get`, `toolchain.check`, `toolchain.show`) to the immutable audit log.
4. **Current Security Policy**: `SECURITY.md` governs reporting protocols, vulnerability definitions, and supported surfaces, checked in CI by `tools/check_security_policy.py`.

### Assumptions (To Be Verified / Formatted in Specification):
1. Toolchain configuration tampering or silent downgrade to vulnerable compiler toolchains constitutes a security-critical event in an ethical hacking platform.
2. Mutation of toolchain configuration in future releases must be strictly PEP-gated.

## 3. Prior Art & Authoritative Citations
- **SLSA v1.0 (Supply-chain Levels for Software Artifacts)**: Mandates hermetic and reproducible build toolchains to guarantee tamper-proof build pipelines.
- **NIST SP 800-218 (Secure Software Development Framework — SSDF §PO.3.1)**: Requires explicit pinning and integrity validation of development and compilation toolchains.
- **OpenSSF Scorecard (Dependency-Pinning & Maintained Requirements)**: Requires pinned compilers and dependencies to mitigate supply chain contamination.
- **AIOS Constitution (ADR-0035 §D-4 / §F-2)**: Classifier → PEP → Audit gate ordering and honest audit emission for all system surfaces.

## 4. Decisions Needed
1. **Vulnerability Scope Inclusion**: Should `SECURITY.md` explicitly list toolchain pinning bypasses and unauthorized toolchain tampering under "What Counts as a Vulnerability"?
   - *Decision*: Yes, add explicit wording to `SECURITY.md`.
2. **Automated Security Policy Check**: Should `tools/check_security_policy.py` assert the presence of toolchain security references?
   - *Decision*: Yes, verify links to toolchain security evidence in the security knowledge index.
3. **Hash Enforcement Phase-In**: When `enforce_hashes` is enabled, how should hash failures be handled?
   - *Decision*: Fail closed with structured error JSON and audit row recording.

## 5. Next Steps
Advance to Specification (T-00372) to formalize the security policy requirements and verification assertions.
