# Task Evidence: T-02091 (recovery & validation: Research)

## Sub-Epic 10 Launch: Capability Model Recovery & Validation Subsystem
- **Task ID**: T-02091
- **Component**: `aiosh-core::capability_recovery`
- **Objective**: Research requirements, failure modes, data integrity invariants, and quarantine recovery patterns for the Capability Model.

## Research Findings & Architecture Design

### 1. Existing Recovery Patterns in AIOS
Analysis of `session_recovery.rs`, `kernel_module_recovery.rs`, and `distro_recovery.rs` establishes key requirements:
- **Non-Destructive Quarantine**: When a store file contains invalid JSON, corrupt records, or filesystem errors, it must never be simply deleted or overwritten (mitigating N-8 and N-20 class arbitrary overwrite vulnerabilities). The damaged file must be copied to a collision-resistant timestamped backup (`<file>.bak.<timestamp>`) with restricted permissions (`0600` on Unix).
- **Comprehensive Deep Validation**: Validation must not merely verify JSON deserialization. It must recursively verify structural invariants:
  - Valid and invalid counts partition total capabilities (`CAPREC1`).
  - Health flag accurately reflects zero errors and zero invalid capabilities (`CAPREC2`).
  - Capability lineage integrity: every `parent_id` must resolve to an existing parent node; delegation trees must be acyclic (`CAPREC3`).
  - Monotonic attenuation: child rights must be a subset of parent rights, and child scope must be within parent scope (`CAPREC4`).
  - Saturated quota sanity: consumed invocations cannot exceed max invocations, and consumed bytes cannot exceed quota bytes (`CAP4`).
- **Safe Recovery Actions**:
  - `LoadedExisting`: Clean store loaded successfully without modifications.
  - `CreatedDefaultFresh`: No store existed; a clean empty capability store was initialized.
  - `RecoveredFromBackup`: Damaged store was quarantined and a healthy store was reconstructed or re-initialized.

### 2. Proposed Invariants (`CAPREC1..CAPREC6`)
- `CAPREC1`: `valid_capabilities + invalid_capabilities == total_capabilities`.
- `CAPREC2`: `healthy == (errors.is_empty() && invalid_capabilities == 0)`.
- `CAPREC3`: Lineage graph completeness and acyclicity (`parent_id` resolution).
- `CAPREC4`: Monotonic attenuation verification across all child capabilities.
- `CAPREC5`: Non-destructive quarantine backup creation before store recovery.
- `CAPREC6`: Deterministic JSON serialization and atomic writes.
