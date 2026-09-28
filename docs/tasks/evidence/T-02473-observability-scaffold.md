# T-02473: Sandbox Enforcement Observability Scaffold

## 1. Scaffold Overview
This task establishes the module skeleton and interface contracts for the Sandbox Enforcement Observability subsystem in `code/aiosh-rust/aiosh-core/src/sandbox_observability.rs` and re-exports it in `aiosh-core/src/lib.rs`.

---

## 2. Types & Interface Definitions
- `SandboxObservabilityReport`: Complete telemetry struct containing timestamp, profile counts, execution tallies, outcome distributions, host capabilities, and health status.
- `sanitize_telemetry_text`: Text sanitization function stripping control characters and enforcing length bounds (`MAX_TELEMETRY_TEXT_LEN = 256`).
- `SandboxObservabilityReport::generate`: Gathers telemetry from `SandboxService` and `AuditRing`.
- `SandboxObservabilityReport::validate`: Asserts bounded maps and timestamp format.

---

## 3. Build Status
- `cargo check --workspace` builds cleanly with 0 warnings, 0 errors.
