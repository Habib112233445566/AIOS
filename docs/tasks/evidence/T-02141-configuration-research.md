# Task Evidence: T-02141 (PEP Decision Engine Configuration: Research)

## Overview
- **Task ID**: `T-02141`
- **Task Name**: configuration: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem (LAUNCH)
- **Timestamp**: 2026-09-21T00:55:20+05:00
- **Status**: COMPLETED

## Objective
Establish architectural facts, operational constraints, and prior art for the configuration subsystem of the PEP Decision Engine, following patterns established in `capability_config.rs` and `system_update_config.rs`.

## Research Findings & Architectural Facts

### 1. Precedence Order
AIOS subsystems enforce strict 4-tier configuration precedence:
1. **CLI Flag Overrides** (e.g., `--store`, `--algorithm`): Highest precedence.
2. **Environment Variables** (e.g., `AIOSH_PEP_CONFIG`, `AIOSH_PEP_STORE_PATH`, `AIOSH_PEP_DEFAULT_ALGORITHM`): Intermediate operator overrides.
3. **Configuration File** (`pep_config.json`): Persistent environment profile.
4. **Compiled Defaults**: Fallback when no configuration source specifies a value.

### 2. Safety Bounds & Invariants (`PEPCONF1..PEPCONF6`)
- `PEPCONF1`: **Path Hygiene**: `store_path` must be $\le 1024$ chars, end in `.json`, contain zero control characters, and contain no parent directory (`..`) traversal components.
- `PEPCONF2`: **Rule Registry Bounds**: `max_rules` bounded within $[1, 50\,000]$ with default of 5,000. `max_store_bytes` bounded within $[1\,024, 104\,857\,600]$ (1 KiB to 100 MiB) with default of 10 MiB.
- `PEPCONF3`: **Algorithm Governance**: `default_algorithm` restricted strictly to known combining algorithms: `deny_overrides`, `permit_overrides`, `first_applicable`.
- `PEPCONF4`: **Audit & Observability Settings**: `audit_all_evaluations: bool` (default: true) controls whether every evaluation emits a row or only state mutations.
- `PEPCONF5`: **Non-Destructive Resilience**: `auto_quarantine_corrupt: bool` (default: true) governs automatic quarantine of unparseable store files to `.bak.<timestamp>`.
- `PEPCONF6`: **Atomic Persistence & File Size Cap**: Configuration files must not exceed 64 KiB (`MAX_CONFIG_BYTES`) to prevent memory exhaustion, and writes must use `.tmp.<pid>` atomic replacement.

## Decisions & Open Questions
- **Decision 1**: Follow the exact struct shape of `CapabilityConfig` and `SystemUpdateConfig`, placing `PepConfig` in `code/aiosh-rust/aiosh-core/src/pep_config.rs` and re-exporting in `lib.rs`.
- **Decision 2**: Integrate `PepCombiningAlgorithm` serialization directly so configuration JSON can specify `"default_algorithm": "deny_overrides"`.

## Acceptance
- Facts separated from assumptions.
- No code modified in research phase.
- Ready for formal specification in `T-02142`.
