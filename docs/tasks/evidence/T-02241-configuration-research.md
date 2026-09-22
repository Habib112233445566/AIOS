# Task Evidence: T-02241 (Grant Lifecycle Configuration: Research)

## Overview
- **Task ID**: `T-02241`
- **Task Name**: Grant Lifecycle Configuration: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem (LAUNCH)
- **Timestamp**: 2026-09-23T01:08:00+05:00
- **Status**: COMPLETED

## Objective
Establish architectural facts, operational constraints, and prior art for the configuration subsystem of the Grant Lifecycle Engine, following patterns established in `pep_config.rs`, `capability_config.rs`, and `system_update_config.rs`.

## Research Findings & Architectural Facts

### 1. Precedence Order
AIOS subsystems enforce strict 4-tier configuration precedence:
1. **CLI Flag Overrides / MCP Tool Arguments** (e.g., `--store`, `store_path`): Highest precedence.
2. **Environment Variables** (e.g., `AIOSH_PEP_GRANT_CONFIG`, `AIOSH_PEP_GRANT_STORE_PATH`, `AIOSH_PEP_GRANT_MAX_GRANTS`): Operator overrides.
3. **Configuration File** (`pep_grant_config.json`): Persistent environment configuration profile.
4. **Compiled Defaults**: Fallback when no configuration source specifies a value.

### 2. Operational Invariants (`GRANTCONF1..GRANTCONF6`)
- `GRANTCONF1`: **Path Hygiene**: `store_path` must be $\le 1024$ chars, end in `.json`, contain zero control characters, and contain no parent directory (`..`) traversal components.
- `GRANTCONF2`: **Store Size Bounds**: `max_store_bytes` bounded within $[1\,024, 104\,857\,600]$ (1 KiB to 100 MiB) with a default of 10 MiB ($10,485,760$ bytes).
- `GRANTCONF3`: **Grant Registry Bounds**: `max_grants` bounded within $[1, 50\,000]$ with default of 5,000.
- `GRANTCONF4`: **Delegation Depth Bounds**: `default_max_delegation_depth` bounded within $[1, 10]$ with default of 3.
- `GRANTCONF5`: **Lifecycle Automations**:
  - `auto_sweep_on_load: bool` (default: true) controls whether expired grants are automatically transitioned to Expired upon service initialization.
  - `cascade_revocation_by_default: bool` (default: false) controls whether revocation commands cascade to child grants when unspecified.
- `GRANTCONF6`: **Atomic Persistence & File Size Cap**: Configuration files must not exceed 64 KiB (`MAX_CONFIG_BYTES`) to prevent memory exhaustion, and disk writes must execute atomic two-phase write-and-rename mechanics (`.tmp.<pid>`).

### 3. Decisions & Prior Art
- **Module Location**: `code/aiosh-rust/aiosh-core/src/pep_grant_config.rs` exported via `code/aiosh-rust/aiosh-core/src/lib.rs`.
- **Struct Name**: `PepGrantConfig`.
- **Environment Variables**:
  - `AIOSH_PEP_GRANT_CONFIG`: Path to configuration file.
  - `AIOSH_PEP_GRANT_STORE_PATH` / `AIOSH_PEP_GRANT_STORE`: Path to grant store JSON.
  - `AIOSH_PEP_GRANT_MAX_GRANTS`: Capacity ceiling.
  - `AIOSH_PEP_GRANT_MAX_STORE_BYTES`: File size ceiling.
  - `AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH`: Default delegation depth.
  - `AIOSH_PEP_GRANT_AUTO_SWEEP`: Auto-sweep toggle ("true"/"false", "1"/"0").
  - `AIOSH_PEP_GRANT_CASCADE_REVOCATION`: Cascade revocation toggle ("true"/"false", "1"/"0").

## Acceptance
- Facts separated from assumptions.
- No code modified in research phase.
- Ready for formal specification in `T-02242`.
