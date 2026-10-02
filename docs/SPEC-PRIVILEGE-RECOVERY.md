# Specification: Privilege Escalation Prevention Store Recovery & Invariant Validation (SPEC-PRIVILEGE-RECOVERY)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Scope**: Store validation diagnostics, corruption quarantine, and non-destructive self-healing recovery for Privilege Escalation Prevention.

## 1. Data Contracts

### 1.1 Issue Severity & Codes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeIssueSeverity {
    Fatal,
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivilegeIssueCode {
    InvalidJson,
    SchemaViolation,
    IllegalKernelTier,
    GrantInconsistency,
    InvalidCapability,
    CapacityExceeded,
    CorruptedActorId,
}
```

### 1.2 Diagnostic & Recovery Models
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeValidationIssue {
    pub actor_id: Option<String>,
    pub code: PrivilegeIssueCode,
    pub severity: PrivilegeIssueSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeValidationReport {
    pub total_contexts: usize,
    pub healthy_contexts: usize,
    pub issues: Vec<PrivilegeValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeRepairAction {
    pub actor_id: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub quarantine_path: Option<String>,
    pub actions: Vec<PrivilegeRepairAction>,
    pub repaired_count: usize,
    pub post_validation: PrivilegeValidationReport,
}
```

## 2. Invariants
- `PRIVRECV1`: Validation parses JSON up to 1 MiB (`MAX_PRIVILEGE_STORE_SIZE`). Files exceeding 1 MiB or containing directory traversal (`..`) are rejected.
- `PRIVRECV2`: Any actor context with `SystemKernel` tier is flagged `Fatal` with `IllegalKernelTier`. Repair demotes actor to `User` and clears grants.
- `PRIVRECV3`: Inconsistent elevation (active flag set without valid grant token) is flagged `Error`. Repair resets elevation flag.
- `PRIVRECV4`: Non-destructive repair creates a timestamped backup (`<store_path>.bak.<ts>`) before modifying disk state.
- `PRIVRECV5`: Unrecoverable/unparseable JSON is moved to `<store_path>.quarantine.<ts>` and replaced with a clean default store.
- `PRIVRECV6`: State writes use atomic temporary file replacement (`.tmp.<pid>`) with immediate unlink on error.
- `PRIVRECV7`: Total context capacity bounded at 16,384 entries to prevent denial of service and memory exhaustion.
- `PRIVRECV8`: Symlink rejection on privilege store targets prevents symlink attack vectors.

## 3. CLI Interface
```bash
# Validate privilege store invariants
aiosh privilege validate [--path <custom-path>]

# Repair store file, creating timestamped backup and quarantining unparseable payloads
aiosh privilege repair [--path <custom-path>]
```

## 4. MCP Tools Interface
- `aios.privilege.validate`:
  - Input: `{ "path": optional string }`
  - Output: Full JSON `PrivilegeValidationReport`
- `aios.privilege.repair`:
  - Input: `{ "path": optional string }`
  - Output: Full JSON `PrivilegeRecoveryResult`
