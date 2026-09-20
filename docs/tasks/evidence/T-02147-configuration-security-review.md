# Task Evidence: T-02147 (PEP Decision Engine Configuration: Security Review)

## Overview
- **Task ID**: `T-02147`
- **Task Name**: configuration: Security Review
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-21T01:04:10+05:00
- **Status**: COMPLETED

## Threat Modeling & Security Controls Analysis (`THREAT-PEPCONF-01..06`)

### `THREAT-PEPCONF-01`: Path Traversal via `store_path`
- **Vector**: Malicious configuration points `store_path` to `/etc/shadow.json` or `../../root/.ssh/id_rsa.json`.
- **Analysis**: `PepConfig::validate()` enforces strict path hygiene: rejection of any `..` component, control characters, lengths $> 1024$, and non-`.json` extensions.
- **Verdict**: MITIGATED. Tested in `test_pep_config.rs` and `test_pep_config_smoke.py`.

### `THREAT-PEPCONF-02`: Symlink Hijacking of Config File
- **Vector**: Attacker places a symlink targeting sensitive files or locations prior to loading configuration.
- **Analysis**: `PepConfig::from_path()` inspects `symlink_metadata(path)` and immediately errors if the file type is a symlink.
- **Verdict**: MITIGATED.

### `THREAT-PEPCONF-03`: Memory Exhaustion via Oversized Config File
- **Vector**: Attacker creates a multi-gigabyte `pep_config.json` causing OOM during read.
- **Analysis**: `PepConfig::from_path()` uses `file.by_ref().take(MAX_CONFIG_BYTES).read_to_string()`, capping reads strictly at 64 KiB (`MAX_CONFIG_BYTES`).
- **Verdict**: MITIGATED.

### `THREAT-PEPCONF-04`: Numerical Boundary Evasion
- **Vector**: Configuration specifies `max_rules: 0` (causing division-by-zero or panic) or `max_rules: 100_000_000` (causing RAM exhaustion).
- **Analysis**: `PepConfig::validate()` enforces `max_rules` $\in [1, 50\,000]$ and `max_store_bytes` $\in [1\,024, 104\,857\,600]$. Any out-of-bounds value returns `PEPCONF_ERR_BOUNDS`.
- **Verdict**: MITIGATED. Tested in `test_pep_config.rs`.

### `THREAT-PEPCONF-05`: Malicious Environment Variable Injection
- **Vector**: Attacker injects malformed or unwhitelisted strings into `AIOSH_PEP_DEFAULT_ALGORITHM` or `AIOSH_PEP_MAX_RULES`.
- **Analysis**: `PepConfig::from_env()` strictly matches algorithm strings against `"deny_overrides"`, `"permit_overrides"`, and `"first_applicable"`, returning an explicit error on any unrecognized string. Integer parsing uses strict `.parse::<usize>()` with error wrapping.
- **Verdict**: MITIGATED.

### `THREAT-PEPCONF-06`: Incomplete State Persistence
- **Vector**: Interrupted configuration write corrupts active configuration on disk.
- **Analysis**: `PepConfig::save_to_path()` writes to a unique `.pep_config.tmp.<pid>` file before executing an atomic `fs::rename()`, ensuring all-or-nothing disk writes.
- **Verdict**: MITIGATED.

## Conclusion
Zero open vulnerabilities found. All configuration attack surfaces are robustly mitigated.
