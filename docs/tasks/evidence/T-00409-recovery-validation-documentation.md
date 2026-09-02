# T-00409 — Dependency & Toolchain Pinning / recovery & validation: Documentation

## 1. Documentation Scope
This task documents the Recovery & Validation capabilities for Dependency & Toolchain Pinning in `docs/README.md`.

## 2. Documentation Updates
- **Document**: `docs/README.md`
- **Section Header**: Updated to `## Dependency & Toolchain Pinning (T-00311..T-00410)`.
- **New Subsection (`**Recovery & Validation:**`)**:
  - **Structural Validation (`validate_toolchain_manifest`)**: Offline JSON syntax and schema validation.
  - **Default Recovery (`recover_default_toolchain`)**: In-memory canonical compile-time fallback defaults.
  - **Drift Reconciliation (`reconcile_toolchain`)**: Runtime compiler probing and structured remediation guidance.
- **Evidence Line**: Updated to link tasks up to `T-00408-recovery-validation-hardening.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
