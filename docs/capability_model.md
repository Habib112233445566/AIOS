# AIOS Capability Model & Security Kernel

## 1. Executive Summary & Architectural Overview

The **Capability Model** is the foundational security primitive of **Phase 2 (Security Kernel & PEP Fabric)** in AIOS. 

In conventional operating systems, access control is ambient and identity-based (e.g. ACLs, POSIX permissions, ambient user authority). In AIOS, all operations performed by autonomous agents, subagents, and user sessions are strictly **capability-based**. An entity possesses zero ambient authority: to inspect a file, open a network socket, invoke an MCP tool, or spawn a process, the caller must present an explicit, unforgeable, and valid `Capability`.

### Primary Architectural Pillars
1. **Zero Ambient Authority**: Processes and agents operate in restricted namespaces where ambient calls are denied. Access requires presenting a capability token.
2. **Unforgeable Tokens**: Capabilities are cryptographically bound to issuer, subject, and timestamp via SHA-256 digests (`cap_<ts>_<hash>`).
3. **Monotonic Attenuation**: Capabilities can be delegated to child agents, but rights can only be reduced or kept equal, never amplified.
4. **Temporal & Resource Bounding**: Capabilities include validity windows, max invocation counts, and byte quotas enforced with saturated arithmetic.
5. **Transitive Revocation**: Capabilities maintain parent-child lineage (`parent_id`) allowing the Security Kernel to cascade revocation across delegation trees.

---

## 2. Capability Model Invariants (CAP1 - CAP6)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`CAP1`** | **Cryptographic Unforgeability** | Every capability possesses a globally unique identifier derived from a cryptographic hash of issuer, subject, and timestamp. Empty issuers, subjects, and rights are rejected. |
| **`CAP2`** | **Target & Rights Scoping** | Capabilities bind to a specific resource scope (`Filesystem`, `Network`, `Tool`, `Process`, `Ipc`, `System`) and an explicit set of rights (`Read`, `Write`, `Execute`, `Delete`, `Admin`, `Delegate`). |
| **`CAP3`** | **Monotonic Attenuation** | Deriving a child capability requires the `Delegate` right on the parent. Child rights must be a subset of parent rights; child scope must be confined within parent scope. |
| **`CAP4`** | **Temporal & Quota Constraints** | Constraints enforce `not_before`, `expires_at`, `max_invocations`, and `quota_bytes` with saturated arithmetic and zero overflow risk. |
| **`CAP5`** | **Lineage & Immediate Revocation** | Explicit `revoke()` immediately marks the capability as revoked; all subsequent validity checks, invocations, and byte consumptions fail with `Revoked`. |
| **`CAP6`** | **Serialization Fidelity** | High-fidelity JSON serialization with identical schema representations across Rust core and Python MCP components. |

---

## 3. Data Model Specifications

### 3.1 Rights & Permissions (`CapabilityRight`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRight {
    Read,     // Inspect, read, or list resource
    Write,    // Create, update, or modify resource
    Execute,  // Execute binary or invoke tool
    Delete,   // Remove file or terminate process
    Admin,    // Manage policy or query audit logs
    Delegate, // Attenuate and grant child capability
}
```

### 3.2 Scoping (`CapabilityScope`)
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

### 3.3 Constraints & Lifecycle (`CapabilityConstraints`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityConstraints {
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub max_invocations: Option<u64>,
    pub current_invocations: u64,
    pub quota_bytes: Option<u64>,
    pub consumed_bytes: u64,
}
```

### 3.4 Core Entity (`Capability`)
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

## 4. Operational Semantics & Attenuation

### 4.1 Attenuation Rules
When deriving a child capability via `parent.attenuate(...)`:
1. The parent must hold the `CapabilityRight::Delegate` right.
2. The parent must not be expired or revoked.
3. Every right in `child_rights` must be present in `parent.rights`.
4. The `child_scope` must satisfy `parent.matches_scope(&child_scope)`.
5. The `child_constraints` cannot extend beyond parent `expires_at` or remaining quota limits.

### 4.2 Example JSON Representation
```json
{
  "id": "cap_1726848000000_a1b2c3d4e5f60718",
  "parent_id": null,
  "issuer": "kernel",
  "subject": "agent:orchestrator",
  "scope": {
    "type": "filesystem",
    "details": {
      "path": "/var/data",
      "recursive": true
    }
  },
  "rights": [
    "read",
    "write",
    "delegate"
  ],
  "constraints": {
    "not_before": null,
    "expires_at": "2026-12-31T23:59:59Z",
    "max_invocations": 500,
    "current_invocations": 0,
    "quota_bytes": 104857600,
    "consumed_bytes": 0
  },
  "revoked": false,
  "created_at": "2026-09-20T15:00:00Z"
}
```

---

## 5. Verification & Test Execution

### Run Rust Unit Tests
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_data_model -- --nocapture
```

### Run Python MCP Smoke Suite
```bash
python code/aiosh-mcp/tests/test_capability_smoke.py
```

---

## 6. Evidence Artifacts
- Research: [`docs/tasks/evidence/T-02001-capability-data-model-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02001-capability-data-model-research.md)
- Specification: [`docs/tasks/evidence/T-02002-capability-data-model-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02002-capability-data-model-specification.md)
- Scaffold: [`docs/tasks/evidence/T-02003-capability-data-model-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02003-capability-data-model-scaffold.md)
- Implementation: [`docs/tasks/evidence/T-02004-capability-data-model-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02004-capability-data-model-implementation.md)
- Unit Testing: [`docs/tasks/evidence/T-02005-capability-data-model-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02005-capability-data-model-unit-test.md)
- Integration: [`docs/tasks/evidence/T-02006-capability-data-model-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02006-capability-data-model-integration.md)
- Security Review: [`docs/tasks/evidence/T-02007-capability-data-model-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02007-capability-data-model-security-review.md)
- Hardening: [`docs/tasks/evidence/T-02008-capability-data-model-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02008-capability-data-model-hardening.md)
