# T-01392: Init & Service Supervision Recovery & Validation Specification

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01392  

---

## 1. Executive Summary & Scope
This specification defines the formal contract for the **Init & Service Supervision Recovery & Validation** subsystem. It specifies the data structures, mathematical invariants, recovery routines, CLI subcommands, MCP tool interfaces, error handling behaviors, and audit guarantees required to ensure that AIOS service supervision remains resilient against corrupted or unparseable state.

---

## 2. Invariants & Mathematical Formulations

The subsystem guarantees five mathematical and operational invariants (`SR1..SR5`):

### Invariant SR1: Inventory Conservation
For any generated `ServiceValidationReport`:
$$\text{valid\_services} + \text{invalid\_services} = \text{total\_services}$$
The total service count must exactly equal the sum of individually validated compliant and non-compliant service units.

### Invariant SR2: Health Equivalence
$$\text{healthy} \iff (\text{errors.is\_empty}() \land \text{invalid\_services} == 0)$$
A store is considered healthy if and only if the accumulated error log is empty and no services are flagged as invalid.

### Invariant SR3: Error Attribution
$$\text{invalid\_services} > 0 \implies \text{errors.len}() \ge \text{invalid\_services}$$
Every invalid service specification must contribute at least one descriptive diagnostic error to the error vector.

### Invariant SR4: Non-Destructive Forensic Preservation
When repairing a damaged or invalid store on disk, the existing file must never be truncated or overwritten directly. It must be atomically quarantined as:
$$\langle\text{path}\rangle\text{.corrupt.}\langle\text{timestamp\_ms}\rangle\text{.bak}$$
with collision avoidance counter (`_<n>`) if a timestamp collision occurs.

### Invariant SR5: Canonical Reconstitution
Any store generated via automated recovery or default initialization must:
1. Contain the standard AIOS reference system services (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `ssh.service`).
2. Pass complete deep validation with $\text{healthy} == \text{true}$ and $\text{errors.is\_empty}() == \text{true}$.
3. Form an acyclic topological dependency graph.

---

## 3. Rust Core Data Types & Interface Specifications

### Module: `code/aiosh-rust/aiosh-core/src/service_recovery.rs`

#### 1. Enumeration: `ServiceRecoveryAction`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceRecoveryAction {
    /// Store loaded cleanly from disk without modification.
    LoadedExisting,
    /// Store file did not exist; created a fresh canonical store.
    CreatedDefaultFresh,
    /// Store was corrupted or invalid; quarantined to backup and reconstituted.
    RecoveredFromBackup {
        backup_path: String,
        reason: String,
    },
}
```

#### 2. Structure: `ServiceValidationReport`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceValidationReport {
    /// Filesystem path of the audited service store.
    pub store_path: String,
    /// Total number of service specifications in the store.
    pub total_services: usize,
    /// Count of fully valid service specifications.
    pub valid_services: usize,
    /// Count of services violating syntax or structural invariants.
    pub invalid_services: usize,
    /// List of descriptive error messages for validation failures.
    pub errors: Vec<String>,
    /// List of non-fatal warnings (e.g. optional unmet dependencies).
    pub warnings: Vec<String>,
    /// Overall health flag (true if errors is empty and invalid_services == 0).
    pub healthy: bool,
    /// UTC timestamp of evaluation in RFC 3339 format.
    pub evaluated_at: String,
}

impl ServiceValidationReport {
    /// Asserts internal consistency invariants SR1..SR3.
    pub fn validate_invariants(&self) -> Result<(), String>;
}
```

#### 3. Core Functions
```rust
/// Validates all specifications, runtime statuses, and dependency topology in a ServiceStore.
pub fn validate_service_store(store: &ServiceStore, store_path: &Path) -> ServiceValidationReport;

/// Recovers a ServiceStore from disk with non-destructive timestamped quarantine on corruption.
pub fn recover_service_store_with_backup(path: &Path) -> (ServiceStore, Option<PathBuf>);

/// High-level entrypoint: loads, validates, and optionally repairs a service store.
pub fn load_or_recover(
    path: &Path,
) -> Result<(ServiceStore, ServiceValidationReport, bool, Option<PathBuf>), String>;
```

---

## 4. Operator CLI Specification (`aiosh-cli`)

### Command Signature
```text
aiosh service check [--fix] [--store <path>] [--json]
```

### Parameter Semantics
- `--store <path>`: Specifies target store path (default: `/var/lib/aios/services.json`). Enforces max length 1,024 bytes and rejects control characters.
- `--fix`: Enables automated recovery mode. If the store file is unreadable, invalid, or corrupted, quarantines the damaged file and writes a fresh canonical store.
- `--json`: Outputs structured JSON envelope matching the standard AIOS envelope schema.

### Execution Flow & Exit Codes
1. **Healthy Store (Audit Mode):**
   - Output: `Service Store Validation: HEALTHY (N services verified)`
   - Exit Code: `0`
2. **Damaged Store (Audit Mode without `--fix`):**
   - Output: Diagnostic error list followed by `Hint: Run with --fix to automatically recover.`
   - Exit Code: `1`
3. **Damaged Store (Recovery Mode with `--fix`):**
   - Output: Reports quarantine action, backup filepath, and successful reconstitution.
   - Exit Code: `0`
4. **Audit Logging:** Every invocation emits an audit row to SQLite WAL (`audit.log` / `audit.db`) with action `service.check` or `service.repair`.

---

## 5. Autonomous Agent MCP Specification (`aiosh-mcp`)

### Tool Definition
- **Tool Name:** `aios.service.check`
- **Description:** `"Validate on-disk service store integrity and optionally perform non-destructive recovery"`

### Input Parameters (JSON Schema)
```json
{
  "type": "object",
  "properties": {
    "store_path": {
      "type": "string",
      "description": "Optional custom path to the service store JSON file"
    },
    "auto_recover": {
      "type": "boolean",
      "description": "If true, automatically quarantines damaged store and reconstitutes canonical defaults",
      "default": false
    }
  }
}
```

### Response Schema
```json
{
  "ok": true,
  "tool": "aios.service.check",
  "report": {
    "store_path": "/var/lib/aios/services.json",
    "total_services": 6,
    "valid_services": 6,
    "invalid_services": 0,
    "errors": [],
    "warnings": [],
    "healthy": true,
    "evaluated_at": "2026-09-09T17:45:00Z"
  },
  "recovered": false,
  "backup_path": null,
  "total_services": 6
}
```

---

## 6. Error Handling & Edge Cases

| Condition | Audit Mode (`--fix` false) | Recovery Mode (`--fix` true) |
|---|---|---|
| **Non-Existent File** | Returns healthy default store (read-only in memory) | Writes new canonical store, returns `CreatedDefaultFresh` |
| **Malformed JSON Syntax** | Reports `LOAD_STORE_FAILED`, exit code 1 | Quarantines to `.corrupt.<ts>.bak`, writes default store |
| **Invalid Service Spec (SS1..SS5)** | Reports spec errors in `report.errors`, exit code 1 | Quarantines to `.corrupt.<ts>.bak`, writes default store |
| **Cyclic Dependency Graph** | Reports `invariant CS3 violated: cyclic dependency detected`, exit 1 | Quarantines to `.corrupt.<ts>.bak`, writes default store |
| **Store Size > 10 MiB** | Reports ceiling overflow error, exit code 1 | Quarantines to `.corrupt.<ts>.bak`, writes default store |
| **Store Entities > 10,000** | Reports capacity overflow error, exit code 1 | Quarantines to `.corrupt.<ts>.bak`, writes default store |

---

## 7. Master Test Suite Integration Contract
The test runner `tools/test_service_suites.py` will integrate criterion `SS10`:
- **Criterion Name:** `SS10 service recovery subsystem & validation invariants (SR1..SR5)`
- **Assertion:** Runs `cargo test --test test_service_recovery` verifying healthy validation, corrupt file quarantine, dependency cycle detection, invariant conservation, and error attribution.
