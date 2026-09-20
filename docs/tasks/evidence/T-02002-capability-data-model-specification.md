# Task Evidence: T-02002 - Capability Model / data model: Specification (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02002`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Specify the exact types, data structures, invariants, error handling, and attenuation semantics for the AIOS Capability Model data model.

---

## 2. Type & Data Structure Specification

### 2.1 Rights & Permissions (`CapabilityRight`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRight {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
    Delegate,
}
```

### 2.2 Scoping (`CapabilityScope`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details", rename_all = "snake_case")]
pub enum CapabilityScope {
    Filesystem { path: String, recursive: bool },
    Network { host: String, port: Option<u16>, protocol: String },
    Tool { tool_name: String, allowed_actions: Vec<String> },
    Process { executable: String, max_memory_bytes: Option<u64> },
    Ipc { channel: String },
    System { subsystem: String },
}
```

### 2.3 Constraints (`CapabilityConstraints`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConstraints {
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub max_invocations: Option<u64>,
    pub current_invocations: u64,
    pub quota_bytes: Option<u64>,
    pub consumed_bytes: u64,
}
```

### 2.4 Core Entity (`Capability`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub parent_id: Option<String>,
    pub issuer: String,
    pub subject: String,
    pub scope: CapabilityScope,
    pub rights: Vec<CapabilityRight>,
    pub constraints: CapabilityConstraints,
    pub revoked: bool,
    pub created_at: String,
}
```

---

## 3. Operational Semantics & Invariants
1. **Creation**:
   - `Capability::new(...)`: Initializes capability with unique UUIDv4, active status, zero invocations, and creation timestamp.
2. **Monotonic Attenuation (`CAP3`)**:
   - `Capability::attenuate(...)`:
     - Derives a child capability.
     - Enforces that child rights are a strict or non-strict subset of parent rights (`child_rights.iter().all(|r| parent.has_right(r))`).
     - Enforces that parent must possess `CapabilityRight::Delegate`.
     - Ensures child constraints are at least as restrictive as parent constraints.
3. **Validation & Quotas (`CAP4`)**:
   - `is_valid_at(&self, now: &DateTime<Utc>) -> Result<(), CapabilityError>`
   - `check_right(&self, right: CapabilityRight) -> Result<(), CapabilityError>`
   - `consume_invocation(&mut self) -> Result<(), CapabilityError>`
   - `consume_bytes(&mut self, bytes: u64) -> Result<(), CapabilityError>`
4. **Revocation (`CAP5`)**:
   - `revoke(&mut self)` sets `revoked = true`.
