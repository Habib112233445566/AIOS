# T-01492: User Session Bootstrap Recovery & Validation Specification

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01492  

---

## 1. Scope & Objective

This specification establishes the formal technical contract for the **Recovery & Validation** subsystem of User Session Bootstrap (`code/aiosh-rust/aiosh-core/src/session_recovery.rs`).

It defines the exact types, method signatures, error cases, invariant validations (`SSR1..SSR5`), non-destructive backup procedures, audit implications, and CLI/MCP interfaces.

---

## 2. Interface Categorization

### 2.1 Reused Existing Interfaces
- **`UserSessionSpec`** (`aiosh_core::session`): Specification payload for user/agent sessions.
- **`UserSessionStatus`** (`aiosh_core::session`): Dynamic runtime status and state representation.
- **`UserSessionStore`** (`aiosh_core::session`): Persistence container mapping session IDs to specs and statuses.
- **`UserSessionService`** (`aiosh_core::session_service`): Central runtime coordinator with security policy enforcement.
- **`validate_user_session_spec`** / **`validate_user_session_status`**: Invariant validation functions (`SB1..SB5`).

### 2.2 New Subsystem Interfaces (`aiosh_core::session_recovery`)
- **`SessionRecoveryAction`**: Enum recording the action taken during store loading and recovery.
- **`SessionValidationReport`**: Structured deep-health audit report across all managed sessions.
- **`validate_session_store(store, path)`**: Pure verification function generating `SessionValidationReport`.
- **`recover_session_store_with_backup(path)`**: Non-destructive quarantine and fresh store initialization.
- **`load_or_recover(path)`**: Unified loading entrypoint providing automated resilience.

---

## 3. Data Structures & Type Definitions

```rust
/// Action taken during store loading and corruption recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

/// Comprehensive deep validation report across managed session specifications and statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionValidationReport {
    pub store_path: String,
    pub total_sessions: usize,
    pub valid_sessions: usize,
    pub invalid_sessions: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}
```

---

## 4. Mathematical & Consistency Invariants (`SSR1..SSR5`)

`SessionValidationReport::validate_invariants(&self) -> Result<(), String>` validates:

1. **`SSR1` (Mathematical Completeness)**:
   ```
   self.valid_sessions + self.invalid_sessions == self.total_sessions
   ```
2. **`SSR2` (Strict Health Definition)**:
   ```
   self.healthy == (self.errors.is_empty() && self.invalid_sessions == 0)
   ```
3. **`SSR3` (Error Cardinality Lower Bound)**:
   ```
   self.invalid_sessions > 0 => self.errors.len() >= self.invalid_sessions
   ```
4. **`SSR4` (Non-Destructive Quarantine & Permissions)**:
   Whenever an existing file is unreadable, corrupted, or reports `!healthy`, it is atomically backed up to `<store_path>.bak.<timestamp>` with POSIX `0600` permissions prior to initializing a fresh seeded store.
5. **`SSR5` (Seat & Process Consistency Checks)**:
   - At most one active session per seat may hold `SessionScope::Foreground`.
   - No two non-terminated sessions may share a `leader_pid`.

---

## 5. Function Contracts & Persistence Effects

### 5.1 `validate_session_store`
```rust
pub fn validate_session_store(
    store: &UserSessionStore,
    store_path: &Path,
) -> SessionValidationReport
```
- **Inputs**: Reference to in-memory `UserSessionStore` and logical `Path`.
- **Outputs**: `SessionValidationReport`.
- **Validation Steps**:
  1. Store capacity: checks `total_sessions <= 10_000` (error if exceeded).
  2. Map key consistency: checks map key equals `spec.session_id` and `status.session_id`.
  3. Spec integrity: calls `validate_user_session_spec(spec)`.
  4. Status integrity: calls `validate_user_session_status(status)`.
  5. Leader PID collision: detects duplicate `leader_pid` across non-terminated sessions.
  6. Seat arbitration: detects multiple `Foreground` sessions on the same seat.
- **Side Effects**: None (pure inspection).

### 5.2 `recover_session_store_with_backup`
```rust
pub fn recover_session_store_with_backup(
    path: &Path,
) -> (UserSessionStore, PathBuf)
```
- **Inputs**: Path to the existing (damaged or corrupted) store.
- **Outputs**: Tuple containing a fresh, seeded `UserSessionStore` (with `greeter-seat0`) and the `PathBuf` of the created quarantine backup file.
- **Side Effects**:
  1. Renames or copies `path` to `<path>.bak.<YYYYMMDD_HHMMSS_micros>`.
  2. Enforces `0600` permissions on the backup file.
  3. Writes fresh default `UserSessionStore` to `path` with `0600` permissions.

### 5.3 `load_or_recover`
```rust
pub fn load_or_recover(
    path: &Path,
) -> Result<(UserSessionService, SessionValidationReport, bool, Option<PathBuf>), String>
```
- **Inputs**: Path to session store.
- **Outputs**: `Ok((service, report, was_recovered, backup_path))` or `Err(reason)`.
- **Contract**:
  - If `!path.exists()`: Creates fresh seeded `UserSessionService`, saves to `path`, returns `(service, report, true, None)`.
  - If file exists and is valid: Loads `UserSessionService`, runs validation. If `report.healthy`, returns `(service, report, false, None)`.
  - If file exists and is invalid/corrupted: Triggers `recover_session_store_with_backup(path)`, returns `(fresh_service, fresh_report, true, Some(backup_path))`.

---

## 6. CLI & MCP Tool Surfaces

### 6.1 Operator CLI (`aiosh session`)
- **`aiosh session check [--store <path>] [--json]`**:
  - Invokes `load_or_recover` or `validate_session_store`.
  - Returns `SessionValidationReport` JSON.
  - Read-only; exit code 0 if healthy, exit code 1 if unhealthy.
- **`aiosh session recover [--store <path>] [--dry-run] [--json]`**:
  - Triggers explicit quarantine and recovery.
  - Emits audit record with previous store hash and backup path.

### 6.2 Autonomous Agent MCP Surface (`aios.session.*`)
- **`aios.session.check`**:
  - Input: `{ "store_path": "string" }`
  - Output: `{ "ok": true, "tool": "aios.session.check", "report": SessionValidationReport }`
  - PEP Gated: No (read-only diagnostic).
- **`aios.session.recover`**:
  - Input: `{ "store_path": "string", "grant_id": "string" }`
  - Output: `{ "ok": true, "tool": "aios.session.recover", "recovered": bool, "backup_path": string, "report": SessionValidationReport }`
  - PEP Gated: Yes (state-mutating, irreversible store reset).

---

## 7. Audit Logging & Security Guarantees
- Consequential recovery actions write an immutable audit row to `.aios/audit.db` via `dispatch::recorded_call()` recording:
  - Tool / Subcommand: `aios.session.recover` / `session recover`.
  - Actor: Invoking agent or user identity.
  - Detail: Reason for quarantine, backup path, and prior state checksum.
- ADR-0035 §D-4 gate ordering is preserved: Classifier -> PEP Gate -> Audit Gate.
