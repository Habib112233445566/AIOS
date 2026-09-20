# Task Evidence: T-02052 (Capability Model / automated tests: Specification)

## Task Information
- **Task ID**: T-02052
- **Title**: Capability Model / automated tests: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Automated Test Specification (`test_capability_automated.rs` & `test_capability_automated_smoke.py`)

### 1. Invariants & Guarantees
- `CAPTEST1`: Hermetic Isolation — tests run in isolated temporary environments without polluting the workspace or OS.
- `CAPTEST2`: Lineage Integrity — parent-child references and transitive descendants remain consistent across mutations.
- `CAPTEST3`: Monotonic Reduction — attenuation strictly narrows or preserves rights, scopes, constraints, quotas, and expiry.
- `CAPTEST4`: Cascade Completeness — revoking an ancestor revokes 100% of all derived descendants.
- `CAPTEST5`: Quota Atomicity — invocation and byte quotas decrement accurately and cannot underflow or be evaded.
- `CAPTEST6`: Fault Resilience — malformed stores, symlinks, traversals, and corrupted files fail closed with structured errors.

### 2. Test Harness Structure (`MockCapabilityEnv`)
```rust
pub struct MockCapabilityEnv {
    pub dir: tempfile::TempDir,
    pub store_path: PathBuf,
}

impl MockCapabilityEnv {
    pub fn new() -> Self;
    pub fn populate_standard_fixtures(&self) -> Result<CapabilityService, CapabilityError>;
}
```

### 3. Test Matrix Definition

| Test Identifier | Category | Invariant | Description |
|---|---|---|---|
| `test_automated_capability_lifecycle_matrix` | Lifecycle | `CAPTEST1`, `CAPTEST2`, `CAPTEST4` | 4-tier capability issuance, attenuation, access checks, and cascade revocation. |
| `test_automated_capability_attenuation_invariants` | Security | `CAPTEST3` | Rejects privilege escalation, scope widening, quota increases, and expiry extension. |
| `test_automated_capability_quota_and_consumption` | Quotas | `CAPTEST5` | Exhaustion of invocation and byte quotas, verifying access denial upon exhaustion. |
| `test_automated_capability_persistence_and_reload` | Persistence | `CAPTEST1`, `CAPTEST2` | Atomic disk save and reload, verifying index rebuilding and state fidelity. |
| `test_automated_capability_pruning_and_temporal` | Temporal | `CAPTEST2`, `CAPTEST4` | Verification of expired leaf pruning while protecting parent nodes with active children. |
| `test_automated_capability_fault_injection` | Robustness | `CAPTEST6` | Testing corrupted JSON, symlinks, path traversal, and oversized files. |

### 4. Cross-Surface E2E Smoke Suite (`test_capability_automated_smoke.py`)
- Executes against compiled `aiosh-mcp.exe`.
- Validates 4-tier attenuation cascade revocation over MCP JSON-RPC.
- Validates quota consumption and exhaustion over MCP JSON-RPC.
- Validates fault injection and boundary protection over MCP JSON-RPC.
