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
- Service Research: [`docs/tasks/evidence/T-02011-capability-service-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02011-capability-service-research.md)
- Service Specification: [`docs/tasks/evidence/T-02012-capability-service-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02012-capability-service-specification.md)
- Service Scaffold: [`docs/tasks/evidence/T-02013-capability-service-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02013-capability-service-scaffold.md)
- Service Implementation: [`docs/tasks/evidence/T-02014-capability-service-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02014-capability-service-implementation.md)
- Service Unit Test: [`docs/tasks/evidence/T-02015-capability-service-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02015-capability-service-unit-test.md)
- Service Integration: [`docs/tasks/evidence/T-02016-capability-service-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02016-capability-service-integration.md)
- Service Security Review: [`docs/tasks/evidence/T-02017-capability-service-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02017-capability-service-security-review.md)
- Service Hardening: [`docs/tasks/evidence/T-02018-capability-service-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02018-capability-service-hardening.md)
- Service Documentation: [`docs/tasks/evidence/T-02019-capability-service-documentation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02019-capability-service-documentation.md)
- Service Formal Closure: [`docs/tasks/evidence/T-02020-capability-service-verification-evidenc.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02020-capability-service-verification-evidenc.md)
- CLI Research: [`docs/tasks/evidence/T-02021-capability-cli-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02021-capability-cli-research.md)
- CLI Specification: [`docs/tasks/evidence/T-02022-capability-cli-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02022-capability-cli-specification.md)
- CLI Scaffold: [`docs/tasks/evidence/T-02023-capability-cli-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02023-capability-cli-scaffold.md)
- CLI Implementation: [`docs/tasks/evidence/T-02024-capability-cli-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02024-capability-cli-implementation.md)
- CLI Unit Test: [`docs/tasks/evidence/T-02025-capability-cli-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02025-capability-cli-unit-test.md)
- CLI Integration: [`docs/tasks/evidence/T-02026-capability-cli-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02026-capability-cli-integration.md)
- CLI Security Review: [`docs/tasks/evidence/T-02027-cli-surface-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02027-cli-surface-security-review.md)
- CLI Hardening: [`docs/tasks/evidence/T-02028-cli-surface-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02028-cli-surface-hardening.md)
- CLI Documentation: [`docs/tasks/evidence/T-02029-cli-surface-documentation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02029-cli-surface-documentation.md)
- CLI Formal Closure: [`docs/tasks/evidence/T-02030-cli-surface-verification-evidenc.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02030-cli-surface-verification-evidenc.md)

---

## 7. Capability Service & Registry Custody (CSERV1 - CSERV6)

The `CapabilityService` is the central in-memory authority and persistence manager for capabilities in the AIOS Security Kernel.

### 7.1 Core Invariants & Operations

| Invariant | Operation | Description |
|---|---|---|
| **`CSERV1`** | **Registry Indexing** | In-memory indexing by capability ID (`HashMap<String, Capability>`), by subject (`HashMap<String, HashSet<String>>`), and by parent (`HashMap<String, HashSet<String>>`). |
| **`CSERV2`** | **Root Issuance Control** | `issue_root_capability` restricted strictly to `issuer == "kernel"` or `issuer.starts_with("admin:")`. Ambient or untrusted root issuance is rejected. |
| **`CSERV3`** | **Monotonic Attenuation** | `attenuate_capability` verifies parent delegation right, derives a strictly narrowed child capability, and updates lineage indexes. |
| **`CSERV4`** | **Cascade Revocation** | `revoke_capability` performs a breadth-first traversal over child capabilities with cycle detection (`visited: HashSet<String>`), revoking the target and all its transitive descendants. |
| **`CSERV5`** | **Safe Atomic Persistence** | `save_to_path` and `load_from_path` enforce path validation (`validate_service_path`), symlink checks, size bounds (max 10MB), capacity bounds (max 10,000 entries), and atomic file renaming via temporary files. |
| **`CSERV6`** | **Automated Pruning** | `prune_expired` cleanses expired leaf capabilities with no active child references, preventing memory leaks while retaining lineage integrity. |

### 7.2 Service Verification Commands
```bash
# Run Rust CapabilityService unit test suite
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_service -- --nocapture

# Run Python CapabilityService smoke test
python code/aiosh-mcp/tests/test_capability_service_smoke.py
```

---

## 8. CLI Surface Reference (`aiosh capability` / `aiosh cap`)

The `aiosh capability` CLI provides an administrative and operator interface to the AIOS Capability Model and Security Kernel.

### 8.1 Command Overview

```bash
aiosh capability <list|show|issue|attenuate|revoke|check|prune> [options]
```

| Subcommand | Purpose | Example Invocations |
|---|---|---|
| `list` | Enumerate capabilities | `aiosh cap list`<br>`aiosh cap list --subject agent:worker --active-only` |
| `show` | Inspect capability attributes | `aiosh cap show cap_1726848000_a1b2c3` |
| `issue` | Issue root capability (kernel/admin) | `aiosh cap issue --issuer kernel --subject agent:admin --scope-type fs --scope-target /var/data --rights read,write,delegate` |
| `attenuate` | Derive child capability | `aiosh cap attenuate --parent cap_root --subject agent:sub --rights read` |
| `revoke` | Revoke capability and cascade | `aiosh cap revoke cap_root` |
| `check` | Verify subject authorization | `aiosh cap check --subject agent:sub --scope-type fs --scope-target /var/data --right read` |
| `prune` | Cleanse expired capabilities | `aiosh cap prune` |

### 8.2 Common Flags
- `--store <PATH>`: Custom capabilities store path (must have `.json` extension, $\le 1024$ chars, no `..`). Defaults to `$AIOSH_HOME/capabilities.json`.
- `--json`: Format output as a structured JSON envelope: `{ "code": 0/1/2, "data": ..., "error": ... }`.
- `-h, --help`: Display usage and subcommand help.

### 8.3 Exit Codes
- `0`: Success
- `1`: Operational error (not found, unauthorized issuer, access denied)
- `2`: Argument / syntax error (invalid flag value, path traversal, control characters)

---

## 9. MCP / API Tool Surface Reference (`aios.capability.*`)

In accordance with ADR-0035 §D-2, the Model Context Protocol (MCP) is the exclusive external tool protocol exposed to AI models and autonomous agents. The capability surface provides seven dedicated tools under the `aios.capability.*` namespace.

### 9.1 Tool Inventory

| MCP Tool Name | Purpose | Consequential | Gated by PEP |
|---|---|---|---|
| `aios.capability.list` | Enumerate capabilities with subject/active filters | No | No |
| `aios.capability.get` | Retrieve capability details by ID | No | No |
| `aios.capability.issue` | Issue root capability (restricted to authorized callers) | Yes | Yes |
| `aios.capability.attenuate` | Derive child capability with monotonic reduction | Yes | Yes |
| `aios.capability.revoke` | Cascade revocation to capability and all descendants | Yes | Yes |
| `aios.capability.check` | Fast permission check with optional quota consumption | Optional (`consume: true`) | Optional |
| `aios.capability.prune` | Prune expired leaf capabilities without active children | Yes | Yes |

### 9.2 Tool Usage Examples (JSON-RPC)

#### 1. Issue Root Capability (`aios.capability.issue`)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.issue",
    "arguments": {
      "issuer": "kernel",
      "subject": "agent:worker",
      "scope_type": "filesystem",
      "scope_target": "/var/data",
      "rights": ["read", "write", "delegate"],
      "max_invocations": 100,
      "expires_in_secs": 3600
    }
  }
}
```

#### 2. Attenuate Child Capability (`aios.capability.attenuate`)
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.attenuate",
    "arguments": {
      "parent_id": "cap_1726848000_a1b2c3",
      "new_subject": "agent:subworker",
      "subset_rights": ["read"],
      "max_invocations": 10
    }
  }
}
```

#### 3. Access Check with Quota Consumption (`aios.capability.check`)
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.check",
    "arguments": {
      "subject": "agent:worker",
      "scope_type": "filesystem",
      "scope_target": "/var/data",
      "right": "read",
      "consume": true
    }
  }
}
```

#### 4. Cascade Revocation (`aios.capability.revoke`)
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.revoke",
    "arguments": {
      "id": "cap_1726848000_a1b2c3"
    }
  }
}
```

### 9.3 Invariants & Security Constraints
- **Audit Invariant (ADR-0035 §A F-2)**: Every MCP tool call executes through `dispatch::recorded_call`, writing exactly one SHA-256 hash-chained audit row to the SQLite WAL audit ring.
- **Input Bounds**: String fields are constrained to $\le 128$ chars (IDs), $\le 256$ chars (subjects/issuers), and $\le 1024$ chars (paths/targets). Control characters are strictly rejected.
- **Atomic Persistence**: Backing store updates are committed atomically via temporary files (`.tmp.<pid>`), preventing corruption during unexpected termination.

---

## 10. Configuration Subsystem Reference (`CapabilityConfig`)

The Capability Configuration subsystem (`code/aiosh-rust/aiosh-core/src/capability_config.rs`) manages registry sizing, persistence locations, expiration defaults, and security policies for the Capability Model.

### 10.1 Schema & Defaults

| Field | Type | Default | Validation Range / Constraints |
|---|---|---|---|
| `version` | `String` | `"1.0.0"` | Non-empty, $\le 32$ chars, no control characters |
| `store_path` | `PathBuf` | `".aios/capability_store.json"` | Non-empty, $\le 1024$ chars, no control chars, no `..` traversal, `.json` extension |
| `max_store_bytes` | `u64` | `10_485_760` (10 MiB) | $1\,024 \le x \le 104\,857\,600$ (1 KiB to 100 MiB) |
| `max_capabilities` | `usize` | `10_000` | $1 \le x \le 1\,000\,000$ |
| `default_expires_secs` | `Option<u64>` | `None` | If `Some(x)`: $1 \le x \le 315\,360\,000$ (1s to 10 years) |
| `enforce_strict_monotonic` | `bool` | `true` | Boolean flag |
| `auto_prune_on_load` | `bool` | `true` | Boolean flag |

### 10.2 Configuration JSON Example
```json
{
  "version": "1.0.0",
  "store_path": ".aios/capability_store.json",
  "max_store_bytes": 10485760,
  "max_capabilities": 10000,
  "default_expires_secs": null,
  "enforce_strict_monotonic": true,
  "auto_prune_on_load": true
}
```

### 10.3 Environment Variable Overrides

| Environment Variable | Target Field | Description & Error Behavior |
|---|---|---|
| `AIOS_CAPABILITY_CONFIG` | Config File Path | Path to JSON configuration file (read bounded to 64 KiB; symlinks rejected) |
| `AIOS_CAPABILITY_STORE_PATH` | `store_path` | Path to capability persistence JSON file (validated against traversal and control chars) |
| `AIOS_CAPABILITY_MAX_CAPABILITIES` | `max_capabilities` | Maximum in-memory capability count (parsed as `usize`; rejects invalid non-numeric strings) |
| `AIOS_CAPABILITY_MAX_STORE_BYTES` | `max_store_bytes` | Maximum serialized store file size in bytes (parsed as `u64`; rejects non-numeric strings) |

### 10.4 Hardening & Security Controls
1. **Bounded File Ingestion**: Configuration file reads are bounded to `MAX_CONFIG_BYTES = 64 * 1024` (64 KiB) using `take()`.
2. **Symlink Defense**: `CapabilityConfig::from_path` verifies `symlink_metadata` and immediately rejects symlinks to prevent redirection attacks.
3. **Strict Path Hygiene**: Path traversal (`..`) components and ASCII control characters are rejected.
4. **Mandatory `.json` Extension**: Persistence file path must explicitly possess a `.json` extension.
5. **Fail-Closed Validation**: Any out-of-range value or parsing error halts initialization with an explicit error.

---

## 11. Automated Test Framework & Invariants Reference

The Automated Test framework for the Capability Model ensures end-to-end correctness, strict monotonic attenuation, cascade revocation completeness, quota enforcement, and cross-surface parity across the Rust core service, CLI, and MCP interfaces.

### 11.1 Test Invariants (`CAPTEST1..CAPTEST6`)

| Invariant | Name | Description |
|---|---|---|
| `CAPTEST1` | Hermetic Isolation | All test scenarios execute within isolated temporary directories via `MockCapabilityEnv`, preventing host state pollution or disk residue. |
| `CAPTEST2` | Multi-Tier Lineage Integrity | Lineage indexes (`by_parent`, `by_subject`) maintain consistency across multi-tier capability hierarchies (Root $\rightarrow$ Tier 1 $\rightarrow$ ... $\rightarrow$ Tier $N$). |
| `CAPTEST3` | Monotonic Attenuation Enforcement | Derived child capabilities cannot expand rights, widen filesystem or resource scopes, exceed parent quotas, or outlive parent expiration. |
| `CAPTEST4` | Cascade Revocation Completeness | Revoking an intermediate node in a capability tree transitively revokes 100% of its descendant sub-tree while leaving ancestor and sibling branches active. |
| `CAPTEST5` | Quota Atomicity & Bounded Enforcement | Invocation counters and byte consumption track accurately with `saturating_add`; operations fail closed with structured errors upon quota exhaustion. |
| `CAPTEST6` | Fault Tolerance & Path Protection | Corrupted JSON store files, non-`.json` extensions, symlink targets, and path traversal (`..`) attempts fail closed with `CSERV_VALIDATION_ERROR`. |

### 11.2 Rust Test Suite (`test_capability_automated.rs`)

| Test Function | Invariant | Description |
|---|---|---|
| `test_automated_mock_env_initialization` | `CAPTEST1` | Validates `MockCapabilityEnv` tempdir creation and fixture pre-population. |
| `test_automated_capability_lifecycle_matrix` | `CAPTEST2`, `CAPTEST4` | Validates 4-tier attenuation and cascade revocation across the hierarchy. |
| `test_automated_capability_attenuation_invariants` | `CAPTEST3` | Validates rejection of rights expansion, scope widening, quota increases, and expiry extension. |
| `test_automated_capability_quota_and_consumption` | `CAPTEST5` | Validates invocation limit decrement and byte quota accumulation and denial. |
| `test_automated_capability_persistence_and_reload` | `CAPTEST1`, `CAPTEST2` | Validates atomic disk save and reload with index rebuilding. |
| `test_automated_capability_pruning_and_temporal` | `CAPTEST4` | Validates pruning of expired leaf and child capabilities while preserving active parent nodes. |
| `test_automated_capability_fault_injection` | `CAPTEST6` | Validates rejection of corrupted store files, bad extensions, and path traversal. |
| `test_automated_capability_deep_hierarchy_stress` | `CAPTEST2`, `CAPTEST4` | Validates 50-level deep attenuation tree and subsequent cascade revocation without recursion failure. |

### 11.3 Cross-Surface Python E2E Smoke Suite (`test_capability_automated_smoke.py`)
- **Multi-tier Attenuation & Cascade Revocation**: Validates 4-tier lifecycle over MCP JSON-RPC (`aios.capability.issue` $\rightarrow$ `attenuate` $\rightarrow$ `check` $\rightarrow$ `revoke`).
- **Quota Exhaustion**: Validates quota consumption over MCP JSON-RPC with `consume: true`.
- **Fault Injection**: Validates path traversal and extension rejection over MCP JSON-RPC.
- **Process Safety**: Enforces leak-proof child process reaping (`p.kill()` and `p.wait()` in `finally` block).

---

## 12. Capability Security Policy (`CAPSEC1..CAPSEC6`)

The **Capability Security Policy** subsystem governs capability issuance and monotonic attenuation, enforcing Mandatory Access Control (MAC) rules across all subjects in the Security Kernel.

### 12.1 Policy Invariants (`CAPSEC1..CAPSEC6`)

| Invariant | Name | Description |
|---|---|---|
| `CAPSEC1` | Default Deny & Policy Modes | Policy operates in `Enforcing`, `Audit`, or `Permissive` mode. In `Enforcing` mode, any policy violation immediately blocks capability issuance or derivation. |
| `CAPSEC2` | Attenuation Depth Bound | Derivation depth cannot exceed `max_attenuation_depth` (default: 64, configurable: 1..=128) to prevent unbounded recursion or tree explosion. |
| `CAPSEC3` | Sensitive Resource Restrictions | Filesystem paths matching prohibited prefixes (`/etc`, `/proc`, `/sys`, `/dev`, `/root`, `C:\Windows`, etc.) and network hosts matching prohibited targets (`169.254.169.254`, `metadata.google.internal`) are blocked. |
| `CAPSEC4` | Subject Disallowed Rights | Subjects with specified prefixes (e.g. `untrusted:*`, `guest:*`) are strictly forbidden from receiving dangerous rights (`Admin`, `Delegate`, `Delete`, `Write`). |
| `CAPSEC5` | Mandatory Temporal Bounds | When `require_temporal_bounds` is active, capabilities issued to non-system subjects must specify `expires_at` within `max_validity_duration_seconds`. |
| `CAPSEC6` | Auditability & Determinism | Every policy check returns a deterministic `CapabilityPolicyVerdict` containing structured `CapabilityPolicyViolation` records for telemetry and auditing. |

### 12.2 Hardened Evaluation Logic

1. **Path Normalization & Traversal Defense**:
   - Rejects paths containing path traversal components (`..`) with `CAPSEC_PATH_TRAVERSAL`.
   - Collapses redundant slashes (`//etc///shadow` $\rightarrow$ `/etc/shadow`).
   - Normalizes Windows and POSIX separators.
2. **Host Sanitization**:
   - Strips IPv4/IPv6 square brackets (`[...]`).
   - Strips accidental port suffixes (`:80`).
   - Strips trailing dots (`metadata.google.internal.`).
3. **Cycle Detection in Derivation Trees**:
   - Uses a `HashSet` visited set in `get_derivation_depth()` to detect cycles and caps traversal iterations to 256.

### 12.3 MCP Usage Example

#### Request: Attempting Prohibited Path Issuance
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.issue",
    "arguments": {
      "issuer": "kernel",
      "subject": "agent:tester",
      "scope_type": "filesystem",
      "scope_target": "/etc/shadow",
      "rights": ["read"]
    }
  }
}
```

#### Response: Policy Rejection Envelope
```json
{
  "ok": false,
  "error": "CSERV_VALIDATION_ERROR: policy violation: CAPSEC_PROHIBITED_PATH: path '/etc/shadow' matches prohibited prefix '/etc'"
}
```

### 12.4 Known Constraints & Limitations
1. **Static Prefix Rules**: Prohibited paths use normalized prefix matching; symlink resolution at capability issuance time requires filesystem access and is enforced at execution/PEP dispatch time.
2. **In-Memory Cycle Guard**: Cycles are detected dynamically during depth calculation; structural parent-child acyclicity is guaranteed by monotonic UUID generation during issuance.

---

## 13. Capability Observability Subsystem (`CAPOBS1..CAPOBS6`)

The **Capability Observability Subsystem** provides point-in-time state aggregation, lineage depth analysis, quota consumption tracking, distribution metrics, and health evaluation for the AIOS Capability Registry.

### 13.1 Observability Invariants (`CAPOBS1..CAPOBS6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`CAPOBS1`** | **Complete State Aggregation** | The report must accurately partition the total capability count into active, revoked, expired, root, and attenuated sets. |
| **`CAPOBS2`** | **Derivation Lineage Depth** | Lineage depths are computed across all active and attenuated capabilities with memoized $O(N)$ evaluation and cycle bounds. |
| **`CAPOBS3`** | **Quota Consumption Aggregation** | Invocations and bytes consumed across all capabilities are aggregated using saturating 64-bit arithmetic to prevent integer overflow. |
| **`CAPOBS4`** | **Distribution Metrics** | The subsystem aggregates scope types (`filesystem`, `network`, `tool`, `process`, `ipc`, `system`), rights distribution, and counts of unique subjects and issuers. |
| **`CAPOBS5`** | **Health Evaluation** | System health evaluates to `true` if and only if registry capacity utilization is below 95% and maximum derivation depth does not exceed the policy's `max_attenuation_depth`. |
| **`CAPOBS6`** | **High-Fidelity Serialization & Sanitization** | Telemetry strings (timestamps and labels) are sanitized to eliminate control characters and bounded to 256 characters. Serialized outputs adhere to deterministic JSON schema. |

### 13.2 Architecture & Data Structures

The primary data structure is `CapabilityObservabilityReport`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityObservabilityReport {
    pub total_capabilities: usize,
    pub active_capabilities: usize,
    pub revoked_capabilities: usize,
    pub expired_capabilities: usize,
    pub root_capabilities: usize,
    pub attenuated_capabilities: usize,
    pub max_derivation_depth: usize,
    pub unique_subjects_count: usize,
    pub unique_issuers_count: usize,
    pub total_invocations_consumed: u64,
    pub total_bytes_consumed: u64,
    pub capabilities_by_scope_type: HashMap<String, usize>,
    pub capabilities_by_right: HashMap<String, usize>,
    pub policy_mode: CapabilityPolicyMode,
    pub max_capabilities_capacity: usize,
    pub capacity_utilization_percent: u8,
    pub is_healthy: bool,
    pub generated_at: String,
}
```

### 13.3 MCP Integration (`aios.capability.observability`)

The capability observability report is exposed via MCP tool `aios.capability.observability`.

#### Request Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.observability",
    "arguments": {
      "timestamp": "2026-09-20T18:30:00Z"
    }
  }
}
```

#### Response Example
```json
{
  "total_capabilities": 4,
  "active_capabilities": 3,
  "revoked_capabilities": 1,
  "expired_capabilities": 0,
  "root_capabilities": 2,
  "attenuated_capabilities": 2,
  "max_derivation_depth": 2,
  "unique_subjects_count": 3,
  "unique_issuers_count": 2,
  "total_invocations_consumed": 20,
  "total_bytes_consumed": 3000,
  "capabilities_by_scope_type": {
    "filesystem": 3,
    "network": 1
  },
  "capabilities_by_right": {
    "read": 4,
    "write": 1,
    "delegate": 2
  },
  "policy_mode": "enforcing",
  "max_capabilities_capacity": 10000,
  "capacity_utilization_percent": 0,
  "is_healthy": true,
  "generated_at": "2026-09-20T18:30:00Z"
}
```

### 13.4 Security & Operational Guarantees

1. **DoS & Algorithmic Complexity Protection**:
   - `get_memoized_depth` caches intermediate capability depths in $O(N)$ time, avoiding $O(N \times D)$ repeated traversals.
   - Cycle detection via `visiting: HashSet<String>` breaks recursion safely if corrupted or circular delegation chains are encountered.
2. **Telemetry Sanitization**:
   - Input timestamps are passed through `sanitize_telemetry_text()`, stripping control characters (`\0`, `\r`, `\n`, `\t`, ANSI escapes) and clamping string length to 256 characters.
   - Empty or all-control-character inputs fall back safely to valid RFC 3339 UTC timestamps.
3. **Overflow Protection**:
   - All consumption counters use saturating arithmetic (`saturating_add`), eliminating integer wrap-around vulnerabilities.

---

## 14. Capability Documentation Subsystem (`CAPDOC1..CAPDOC6`)

The **Capability Documentation Subsystem** provides an offline, self-contained reference repository covering zero ambient authority architecture, rights and scopes, monotonic attenuation, constraints, cascade revocation, security policy, observability, and MCP tools.

### 14.1 Documentation Invariants (`CAPDOC1..CAPDOC6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`CAPDOC1`** | **Canonical Coverage** | The index must contain comprehensive canonical topics covering all capability model components: Overview, Rights & Scopes, Monotonic Attenuation, Constraints & Quotas, Cascade Revocation, Security Policy, Observability, and MCP Tools. |
| **`CAPDOC2`** | **Deterministic Lookup** | Topics are retrieved by unique identifier (`get_topic`) using trimmed, case-insensitive matching in $O(1)$ or $O(N)$ time. |
| **`CAPDOC3`** | **Relevance-Scored Search** | Search queries score topics based on weighted field matches: ID = 100/40, Tags = 50/20, Title = 25, Summary = 15, Section Content = 10. Results are sorted descending by score. |
| **`CAPDOC4`** | **UTF-8 Snippet Safety** | Contextual snippet extraction around matching query terms must strictly align to UTF-8 character boundaries (`char_indices`), completely preventing multi-byte slicing panics (guarding against N-21 class bugs). Max snippet length is 160 characters. |
| **`CAPDOC5`** | **Defensive Bounds** | Query strings are capped at 256 characters (`MAX_DOC_QUERY_LEN`), topic IDs at 64 characters (`MAX_TOPIC_ID_LEN`), and search results at 50 (`MAX_DOC_SEARCH_RESULTS`). Control characters in inputs are rejected. |
| **`CAPDOC6`** | **Serialization Fidelity** | All topic and search result structures derive `Serialize` and `Deserialize` with deterministic snake_case JSON schemas. |

### 14.2 Architecture & Topic Hierarchy

Topics are grouped into five distinct categories:
- **`architecture`**: `cap-overview`, `cap-rights-scopes`
- **`lifecycle`**: `cap-attenuation`, `cap-constraints`, `cap-revocation`
- **`security`**: `cap-policy`
- **`observability`**: `cap-observability`
- **`reference`**: `cap-mcp-tools`

```rust
pub struct CapabilityDocTopic {
    pub id: String,
    pub title: String,
    pub category: CapabilityDocCategory,
    pub summary: String,
    pub sections: Vec<CapabilityDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}
```

### 14.3 Search Engine & UTF-8 Safe Snippets

The search engine inspects topic identifiers, tags, titles, summaries, and section contents with weighted relevance scoring. To prevent multi-byte UTF-8 character slicing panics (N-21 / H-6 class bugs), snippet extraction uses `extract_utf8_snippet`:

```rust
pub fn extract_utf8_snippet(content: &str, byte_idx: usize, query_char_len: usize) -> String {
    let char_indices: Vec<(usize, char)> = content.char_indices().collect();
    // Aligns start and end byte offsets strictly to valid character boundaries
    // ...
}
```

### 14.4 MCP Interface (`aios.capability.doc`)

Exposes capability documentation over JSON-RPC via `aios.capability.doc`.

#### Example: Search Documentation
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.doc",
    "arguments": {
      "action": "search",
      "query": "attenuation"
    }
  }
}
```

#### Example: Get Topic Details
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.doc",
    "arguments": {
      "action": "get",
      "topic_id": "cap-overview"
    }
  }
}
```

---

## 15. Capability Store Recovery, Quarantine, and Deep Validation (`CAPREC1..CAPREC6`)

The **Capability Store Recovery Subsystem** provides automated self-healing, non-destructive quarantine of damaged or corrupted stores, and deep validation reports verifying capability invariants across the security kernel.

### 15.1 Recovery Invariants (`CAPREC1..CAPREC6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`CAPREC1`** | **Conservation of State** | `valid_capabilities + invalid_capabilities == total_capabilities`. The report must account for every capability in the registry. |
| **`CAPREC2`** | **Health Equivalence** | `healthy == (errors.is_empty() && invalid_capabilities == 0)`. A store is deemed healthy if and only if zero validation errors were detected and zero capabilities were found invalid. |
| **`CAPREC3`** | **Lineage Integrity & Acyclicity** | Every attenuated capability must reference an existing parent capability (`parent_id`). The delegation graph must be strictly acyclic, enforced via `HashSet<String>` cycle tracking with a recursion limit of 256. |
| **`CAPREC4`** | **Monotonic Attenuation Confinement** | For every child capability: child rights $\subseteq$ parent rights; child scope $\subseteq$ parent scope; child expiration $\le$ parent expiration. Any violation marks the capability invalid and reports a privilege escalation. |
| **`CAPREC5`** | **Non-Destructive Quarantine** | When a store fails validation or suffers deserialization corruption, the damaged file is never truncated or overwritten. It is backed up to `<store_path>.bak.<timestamp>` with restricted mode `0600` on Unix platforms. A fresh store is initialized in its place. |
| **`CAPREC6`** | **Atomic Persistence & Path Hygiene** | Store paths must satisfy length $\le 1024$, have a `.json` extension, contain no `..` traversal components, and not be symlinks. State persistence is performed via temporary file rename with directory auto-creation. |

### 15.2 Data Types & Architecture

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityValidationReport {
    pub store_path: String,
    pub total_capabilities: usize,
    pub valid_capabilities: usize,
    pub invalid_capabilities: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}
```

### 15.3 MCP Tools (`aios.capability.recover` & `aios.capability.validate`)

#### `aios.capability.recover`
Attempts to load the capability store. If healthy, returns `LoadedExisting`. If non-existent, creates a default store and returns `CreatedDefaultFresh`. If corrupted or invalid, quarantines the damaged file to `.bak.<timestamp>` and returns `RecoveredFromBackup`.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.recover",
    "arguments": {
      "store_path": ".aios/capability_store.json"
    }
  }
}
```

#### `aios.capability.validate`
Performs read-only structural validation against invariants `CAPREC1..CAPREC4` without mutating files on disk.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.capability.validate",
    "arguments": {
      "store_path": ".aios/capability_store.json"
    }
  }
}
```








