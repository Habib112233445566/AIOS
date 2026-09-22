# Task Evidence: T-02247 (Grant Lifecycle Configuration: Security Review)

## Overview
- **Task ID**: `T-02247`
- **Task Name**: Grant Lifecycle Configuration: Security Review
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-23T01:28:00+05:00
- **Status**: COMPLETED

## Threat Modeling & Security Controls Analysis (`THREAT-GRANTCONF-01..06`)

### `THREAT-GRANTCONF-01`: Path Traversal via `store_path`
- **Vector**: Malicious configuration points `store_path` to `/etc/shadow.json` or `../../root/.ssh/id_rsa.json`.
- **Analysis**: `PepGrantConfig::validate()` strictly enforces path hygiene: rejects any `..` traversal component, ASCII/Unicode control characters, non-`.json` extensions, and lengths $> 1024$.
- **Verdict**: MITIGATED. Tested in `test_pep_grant_config.rs`.

### `THREAT-GRANTCONF-02`: Symlink Hijacking of Configuration File
- **Vector**: Attacker places a symlink pointing to sensitive system files prior to loading `pep_grant_config.json`.
- **Analysis**: `PepGrantConfig::from_path()` inspects `symlink_metadata(path)` and immediately fails closed if the file is a symlink (`GRANTCONF_ERR_VALIDATION`).
- **Verdict**: MITIGATED.

### `THREAT-GRANTCONF-03`: Memory Exhaustion via Oversized Configuration or Store
- **Vector**: Attacker supplies multi-gigabyte files to cause Out-Of-Memory (OOM) denial-of-service.
- **Analysis**: Configuration file reading is bounded by `MAX_CONFIG_BYTES` ($64\text{ KiB}$) using `file.by_ref().take(MAX_CONFIG_BYTES)`. Grant store sizing is clamped by `max_store_bytes` $\in [1\,024, 104\,857\,600]$ ($1\text{ KiB} \dots 100\text{ MiB}$), with 16 MiB default guard.
- **Verdict**: MITIGATED.

### `THREAT-GRANTCONF-04`: Numerical Boundary Evasion
- **Vector**: Configuration specifies `max_grants: 0` or `default_max_delegation_depth: 0` / `999` to bypass authorization or exhaust memory.
- **Analysis**: `PepGrantConfig::validate()` strictly bounds:
  - `max_grants` $\in [1, 50\,000]$
  - `default_max_delegation_depth` $\in [1, 10]$
  - `max_store_bytes` $\in [1\,024, 104\,857\,600]$
- **Verdict**: MITIGATED. Tested in `test_pep_grant_config.rs`.

### `THREAT-GRANTCONF-05`: Malicious Environment Variable Injection
- **Vector**: Malformed strings injected into `AIOSH_PEP_GRANT_AUTO_SWEEP`, `AIOSH_PEP_GRANT_CASCADE_REVOCATION`, or numeric variables.
- **Analysis**: Boolean parser strictly recognizes only `{"true", "1", "yes"}` and `{"false", "0", "no"}`, failing closed on unrecognized tokens. Numeric parsers return explicit `GRANTCONF_ERR_BOUNDS` upon syntax failure.
- **Verdict**: MITIGATED.

### `THREAT-GRANTCONF-06`: Incomplete State Persistence / Atomic Replacement
- **Vector**: Interrupted configuration saves leave corrupted partial JSON on disk.
- **Analysis**: `PepGrantConfig::save_to_path()` stages data into `.pep_grant_config.tmp.<pid>`, flushes, and executes an atomic `std::fs::rename()`. Temporary files are removed if any step fails.
- **Verdict**: MITIGATED.

## Conclusion
Zero high or medium severity risks identified. All configuration threat vectors are guarded by defense-in-depth controls.
