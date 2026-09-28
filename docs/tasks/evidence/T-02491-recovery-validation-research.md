# Task T-02491 Evidence: Sandbox Recovery & Validation Research

## Goal
Establish facts, constraints, and prior art for the recovery & validation subsystem of Sandbox Enforcement.

## Facts vs. Assumptions

| Domain | Facts (Authoritative Sources) | Assumptions |
|---|---|---|
| Factory Baseline Integrity | AIOS Sandbox Enforcement requires the three factory profiles (`standard`, `strict`, `permissive`) to be present and structurally valid for predictable containment. | Corrupted custom profile manifests or disk configurations should not prevent fallback to factory defaults. |
| Diagnostic Reporting | System administrators and orchestration agents require structured diagnostic reports identifying corrupted files, missing profiles, invalid resource bounds, and traversal attempts. | Diagnostic reports can use an `issues` vector with structured severity codes (`Error`, `Warning`). |
| Recovery Strategies | Automatic overwriting of corrupt files without preserving evidence violates audit and forensics standards. | A quarantine pattern (moving corrupted files into `.quarantine_<timestamp>` before resetting) preserves evidence while restoring service health. |

## Authoritative Prior Art in AIOS
- `code/aiosh-rust/aiosh-core/src/pep_recovery.rs`: Multi-level validation, diagnostic reporting with severity, quarantine backups, salvage strategies.
- `code/aiosh-rust/aiosh-core/src/service_recovery.rs`: State file validation, backup creation, and atomic reset.
- `code/aiosh-rust/aiosh-core/src/audit_chain_recovery.rs`: Pre-flight snapshotting, structural cycle detection, forward repair anchoring.

## Subsystem Architecture & Decisions
1. **Module Creation**: Implement `code/aiosh-rust/aiosh-core/src/sandbox_recovery.rs`.
2. **Key Structures**:
   - `SandboxValidationIssue`: Carries `profile_name`, `code`, `message`, `severity` (`Error`, `Warning`).
   - `SandboxValidationReport`: Carries `is_healthy`, `factory_profiles_intact`, `total_profiles_checked`, `valid_count`, `corrupt_count`, `issues`, `timestamp`.
   - `SandboxRecoveryStrategy`: `DryRun`, `RestoreFactoryDefaults`, `QuarantineAndReset`.
   - `SandboxRecoveryResult`: Reports `success`, `strategy`, `quarantine_path`, `profiles_restored`, `issues_resolved`.
3. **Recovery Manager**: `SandboxRecoveryManager` orchestrating validation and repair across `SandboxService` and profile storage.
