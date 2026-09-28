# Task T-02503 Evidence: Privilege Escalation Prevention Data Model Scaffold

## Goal
Scaffold the types, interfaces, error codes, and library exports for the Privilege Escalation Prevention data model.

## Delivered Artifacts
1. **Module Implementation (`code/aiosh-rust/aiosh-core/src/privilege_data_model.rs`)**:
   - `PrivilegeLevel`: `Guest` (0), `User` (10), `Operator` (20), `Admin` (30), `SystemKernel` (40).
   - `PrivilegeCapability`: Granular typed capabilities (`ProcessSpawn`, `NetworkConnect`, `NetworkListen`, `FilesystemWrite`, `MemoryInspect`, `AuditLogAdmin`, `SystemReboot`, `KernelModuleLoad`) with `minimum_level()`.
   - `PrivilegeContext`: Tracks actor ID, active tier, granted capabilities set, elevation grant ID, and session ID with validation methods (`new`, `validate`, `add_capability`, `has_capability`).
   - `PrivilegeTransitionRequest`: Carries `actor_id`, `from_level`, `target_level`, `requested_capabilities`, `grant_id`, and transition evaluation logic.
   - `PrivilegeEscalationVerdict`: `Allowed`, `GrantRequired`, `Denied`.
   - Standard constants: `MAX_ACTOR_ID_LEN` (128), `MAX_CAPABILITIES_COUNT` (32), and error codes (`PRIVESC_ERR_*`).
2. **Library Exports (`code/aiosh-rust/aiosh-core/src/lib.rs`)**:
   - Declared `pub mod privilege_data_model;`.
   - Re-exported core symbols and error codes.
3. **Compilation**:
   - `cargo check --workspace`: Clean build with 0 warnings and 0 errors across all workspace crates.
