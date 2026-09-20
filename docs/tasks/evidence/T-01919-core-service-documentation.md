# Task Evidence: T-01919 - System Update Mechanism / core service: Documentation

## 1. Overview
- **Task ID**: `T-01919`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Author comprehensive technical documentation for the System Update Core Service in `docs/system_update.md`.

---

## 2. Documentation Authored
File: `docs/system_update.md`

Authored **Section 5: Core Service Subsystem**:
- **5.1 Service Configuration**: Documented `SystemUpdateServiceConfig` fields and defaults.
- **5.2 Service Lifecycle Flow**: Step-by-step description of intake (`check_manifest`), staging (`stage_artifact`), verification (`verify_staged`), slot switching (`apply_update`), boot confirmation (`confirm_boot`), and rollback (`rollback`).
- **5.3 Core Service Invariants (`USVC1..USVC6`)**: Staging isolation, cryptographic digest gate, running slot non-interference, atomic state persistence, rollback safeguard, and deterministic error reporting.
- **5.4 Core Service Threat Mitigations**: Table summarizing `THREAT-USVC-01..04` mitigations (symlink defense, disk quota DoS defense, temporary file hygiene, state deserialization validation).

---

## 3. Verification
- `docs/system_update.md` Section 5 verified complete and cross-referenced with `system_update_service.rs`.
