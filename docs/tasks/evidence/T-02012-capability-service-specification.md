# Task Evidence: T-02012 - Capability Model / core service: Specification (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02012`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Specify the complete contract, data structures, methods, error types, and operational behaviors for `CapabilityService`.

---

## 2. Service Specification

### 2.1 Struct Definition
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityService {
    capabilities: HashMap<String, Capability>,
    by_subject: HashMap<String, HashSet<String>>,
    by_parent: HashMap<String, HashSet<String>>,
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}
```

### 2.2 Core Service Methods
1. **`new() -> Self`**: Creates an empty capability registry.
2. **`issue_root_capability(issuer, subject, scope, rights, constraints) -> Result<Capability, CapabilityError>`**:
   - Validates inputs.
   - Enforces that only authorized issuers (e.g. `kernel`, `admin`) can issue root capabilities (`parent_id: None`).
   - Indexes into `capabilities` and `by_subject`.
3. **`get_capability(&self, id: &str) -> Option<&Capability>`**:
   - $O(1)$ lookup by capability ID.
4. **`get_capabilities_for_subject(&self, subject: &str) -> Vec<Capability>`**:
   - Returns all active (non-revoked) capabilities granted to the given subject.
5. **`attenuate_capability(&mut self, parent_id: &str, new_subject: &str, narrowed_scope, subset_rights, narrowed_constraints) -> Result<Capability, CapabilityError>`**:
   - Retrieves parent; calls `parent.attenuate(...)`.
   - Inserts child capability; updates `by_subject` and `by_parent` indexes.
6. **`revoke_capability(&mut self, id: &str) -> Result<Vec<String>, CapabilityError>`**:
   - Marks target capability as revoked.
   - Recursively traverses `by_parent` to revoke all transitive child capabilities.
   - Returns list of all revoked capability IDs.
7. **`check_access(&self, subject: &str, requested_scope: &CapabilityScope, required_right: CapabilityRight) -> Result<&Capability, CapabilityError>`**:
   - Finds any active capability held by `subject` that covers `requested_scope`, grants `required_right`, is not revoked, and satisfies temporal/quota constraints.
8. **`save_to_path(&self, path: &Path) -> Result<(), String>`**:
   - Persists registry via `.tmp.<pid>` with atomic rename, bounded size ($\le 10 \text{ MB}$), and symlink rejection.
9. **`load_from_path(path: &Path) -> Result<Self, String>`**:
   - Loads and rebuilds indexes from disk with strict validation.
10. **`prune_expired(&mut self, now: DateTime<Utc>) -> usize`**:
    - Removes expired capabilities that have no active children.
