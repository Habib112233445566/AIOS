# T-01187: Base Image Build Documentation Security Review

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Documentation  
**Task ID:** T-01187  

## 1. Security Review & Threat Modeling

### A. Secret & Credential Leakage Audit
- Scanned `docs/base_image_build.md` for leaked credentials, passwords, cryptographic keys, and internal infrastructure URLs.
- **Finding**: Zero sensitive tokens or private keys present. All paths, identifiers, and configuration examples use sanitized, generic placeholders.

### B. Command & Argument Injection Surface
- Reviewed all documented CLI commands (`aiosh image *`) and MCP tools (`aios.image.*`).
- Verified that examples adhere strictly to safe syntax:
  - Positional arguments (e.g. `id`) and option flags (`--format`, `--distro`, `--store`, `--config`) are shown without shell metacharacters (`|`, `;`, `&`, `$()`, backticks).
  - Codebases (`aiosh-cli` and `aiosh-mcp`) enforce ASCII graphic character validation and reject control characters with code `INVALID_ARGUMENT` (exit code 2).

### C. Abuse Scenarios & Mitigations

#### Scenario 1: Malicious Manifest Injection via Kernel Parameters
- **Threat**: An adversary or compromised agent injects `nokaslr` or `init=/bin/sh` into a manifest to compromise the target rootfs during Stage 2 (Customize).
- **Mitigation**: Base Image Security Policy invariant `P1` rejects blacklisted kernel parameters in `Enforcing` mode, preventing build plan synthesis and writing an audit violation to SQLite WAL.

#### Scenario 2: Legacy Insecure Package Backdoor
- **Threat**: An attacker specifies `telnetd` or `rsh-client` in `packages` to establish unencrypted remote root access.
- **Mitigation**: Base Image Security Policy invariant `P2` explicitly blacklists unencrypted legacy packages. Attempted inclusion results in immediate validation failure.

#### Scenario 3: Store Path Traversal in MCP Tool Calls
- **Threat**: An agent calls `aios.image.report` with `store_path: "../../../etc/shadow"` to probe external files.
- **Mitigation**: Store paths are checked against character whitelists, length bounds (4096 chars), and directory existence; file reads are constrained to valid JSON manifest schemas.

#### Scenario 4: Documentation Drift / Rot Exploitation
- **Threat**: Documentation omits critical security invariants, leading operators or automated pipelines to construct unsafe configurations.
- **Mitigation**: Automated unit suite `tools/test_base_image_doc.py` asserts explicit documentation of kernel parameter blacklists, package blacklists, and invariants. `tools/check_task_docs.py` enforces rot-proof link resolution.

## 2. Review Conclusion
Zero policy bypasses remain open. The documentation accurately reflects implemented security controls and audit guarantees.
