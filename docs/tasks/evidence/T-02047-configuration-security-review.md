# Task Evidence: T-02047 (Capability Model / configuration: Security Review)

## Task Information
- **Task ID**: T-02047
- **Title**: Capability Model / configuration: Security Review
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Threat Model: Configuration Subsystem (`THREAT-CAPCFG-01..06`)

| Threat ID | Threat Vector | CWE | Impact | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-CAPCFG-01` | Path Traversal in `store_path` or `AIOS_CAPABILITY_STORE_PATH` | CWE-22 | Arbitrary file write / persistence hijacking | `CapabilityConfig::validate` inspects all path components and rejects `ParentDir` (`..`). |
| `THREAT-CAPCFG-02` | Resource Exhaustion (DoS) via Unbounded Configuration Fields | CWE-400 / CWE-770 | OOM or disk fill by configuring millions of capabilities | Strict range checks: `max_capabilities` $\in [1, 1\,000\,000]$, `max_store_bytes` $\in [1024, 104\,857\,600]$. |
| `THREAT-CAPCFG-03` | Unbounded Configuration File Ingestion | CWE-776 / CWE-400 | Exhausting memory parsing huge JSON file | `from_path` reads at most `MAX_CONFIG_BYTES` (64 KiB) using `take()`. |
| `THREAT-CAPCFG-04` | Null-Byte & Control Character Injection | CWE-626 / CWE-150 | Null-byte path truncation or terminal log corruption | `store_path` and `version` reject any ASCII control characters (`< 32`). |
| `THREAT-CAPCFG-05` | Inconsistent Registry State on Abrupt Termination | CWE-372 | Corrupted capability registry on crash | Atomic temp file write (`.tmp.<pid>`) + atomic filesystem rename. |
| `THREAT-CAPCFG-06` | Environment Variable Injection & Malformed Overrides | CWE-20 | Process crash or invalid configuration state | `from_env` strictly parses numeric types with safe fallback and re-validates the entire configuration. |

## Review Conclusion
The configuration subsystem architecture has sound foundations. Hardening improvements to be implemented in `T-02048` include:
1. Ensuring `from_env()` rejects malformed numeric environment variables instead of silently ignoring them when set.
2. Enforcing extension validation (`.json`) on `store_path` within `CapabilityConfig::validate`.
3. Ensuring file permissions and symlink metadata checks are consistent between `CapabilityConfig` and `CapabilityService`.
