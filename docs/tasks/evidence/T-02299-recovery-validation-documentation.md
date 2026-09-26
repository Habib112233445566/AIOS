# T-02299: Grant Lifecycle Recovery & Validation Operator Guide

## Overview
The PEP Grant Lifecycle Recovery & Validation subsystem provides diagnostic inspection and non-destructive salvage tools for capability grant stores in AIOS.

## Operational MCP Tools

### 1. `aios.pep.grant.validate_store`
Validates structural invariants across all grants in the store:
- Delegation hierarchy cycles ($A \to B \to A$).
- Dangling / orphan parent references.
- Delegation depth consistency ($child.depth < parent.depth$).
- Monotonic attenuation (child rights must be a subset of parent rights; parent must have `delegate`).
- Cascade revocation consistency (if parent is revoked, all descendants must be revoked).
- Temporal consistency (`not_before <= expires_at`).

#### Invocations Example:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.validate_store",
    "arguments": {
      "store_path": "pep_grants.json"
    }
  }
}
```

#### Sample Response:
```json
{
  "ok": true,
  "tool": "aios.pep.grant.validate_store",
  "report": {
    "total_grants": 4,
    "healthy_grants": 4,
    "issues": [],
    "is_valid": true,
    "can_auto_repair": true
  }
}
```

---

### 2. `aios.pep.grant.recover`
Applies automated reconciliation and non-destructive salvage to restore a compromised or inconsistent grant store to a healthy state:
- Automatically backs up store to `<path>.bak.<timestamp>`.
- Quarantines unparseable JSON files to `<path>.quarantine.<timestamp>.json` and initializes clean storage.
- Revokes orphan grants whose parent records are missing.
- Propagates cascade revocations to any descendant of a revoked parent.
- Auto-expires active grants whose expiration timestamp has passed.
- Performs atomic replacement using temporary files.

#### Invocations Example (Dry Run):
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.recover",
    "arguments": {
      "store_path": "pep_grants.json",
      "dry_run": true
    }
  }
}
```

#### Invocations Example (Execution):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.recover",
    "arguments": {
      "store_path": "pep_grants.json",
      "dry_run": false
    }
  }
}
```

## Constraints & Limitations
- **Cyclic Delegation**: Recovery does not attempt to break or re-parent cyclic delegation loops automatically; stores with cyclic loops must be investigated manually.
- **Fail-Safe Revocation**: When repairing orphan or desynchronized grants, recovery always moves towards stricter states (`Active` $\to$ `Revoked`), never elevating privileges.

## Related Evidence Artifacts
- [T-02291 Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02291-recovery-validation-research.md)
- [T-02292 Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02292-recovery-validation-specification.md)
- [T-02293 Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02293-recovery-validation-scaffold.md)
- [T-02294 Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02294-recovery-validation-implementation.md)
- [T-02295 Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02295-recovery-validation-unit-test.md)
- [T-02296 Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02296-recovery-validation-integration.md)
- [T-02297 Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02297-recovery-validation-security-review.md)
- [T-02298 Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02298-recovery-validation-hardening.md)
