# T-02291: Grant Lifecycle Recovery & Validation Research

## 1. Objective & Scope
This research establishes the foundational requirements, constraints, prior art, and recovery strategies for capability grant stores in the AIOS PEP (Policy Enforcement Point) fabric. The target subsystem is responsible for diagnosing corruption, enforcing delegation invariants, repairing broken delegation graphs, and recovering from store tampering or accidental partial writes.

## 2. Analysis of Existing Codebase & Prior Art

### A. Existing State & Store Architecture
- In `pep_grant_service.rs`, `PepGrantService` manages `PepGrantStore` backed by a canonical JSON file (`grants.json`) with an advisory lock file (`.grants.lock`).
- Grants have rich relational semantics:
  - `grant_id`: Unique identifier (e.g. `grant-uuid`).
  - `parent_grant_id: Option<String>`: Points to issuer/delegator grant.
  - `depth: u32` and `max_depth: u32`: Enforces delegation limits.
  - `status: PepGrantStatus`: `Active`, `Suspended`, `Revoked`, `Expired`.
  - `scopes: Vec<String>` and `allowed_tools: Vec<String>`: Subject to attenuation.
  - `issued_at: u64`, `expires_at: u64`: Temporal bounds.

### B. Failure Modes & Invariant Violations
1. **Malformed JSON / Partial Writes**: Power failure or crash mid-write without atomic rename leads to truncated or invalid JSON syntax.
2. **Dangling Parent References (Orphan Grants)**: A parent grant is deleted or manually stripped from JSON, leaving child grants with an unresolved `parent_grant_id`.
3. **Delegation Depth Drift**: Child grant claims `depth: 1` while parent has `depth: 2`, or child `depth > parent.max_depth`.
4. **Broken Attenuation**: Child grant contains scopes or tools that do not exist in the parent grant.
5. **Inconsistent Cascade Revocation**: Parent grant is `Revoked`, but downstream children remain marked `Active` (e.g. if previous cascade crashed mid-transaction).
6. **Cyclic Delegation**: Malicious or corrupted store contains circular parent references ($A \to B \to A$), leading to infinite loops during tree traversal.
7. **Time Inversion**: `expires_at < issued_at` or grants valid in the past marked as `Active`.

### C. Prior Art & Industry Standards
- **macOS Seatbelt & Fuchsia Capabilities**: Capability trees must maintain single-source root of trust. Broken delegation chains immediately invalidate child handles.
- **Git fsck & SQLite Integrity Check**: Two-phase recovery:
  1. `Check / Dry-run`: Identify all issues with severity classification (`Fatal`, `Error`, `Warning`).
  2. `Repair / Salvage`: Create an atomic snapshot of original state, quarantine irreparable records, auto-cascade dangling or revoked trees, and atomically write clean store.

## 3. Key Design Decisions
1. **Two-Tier Operation**:
   - `validate`: Read-only scan returning diagnostic report with issue list, severity, and healthy count.
   - `repair` / `salvage`: Atomic repair creating `.bak.<ts>` backup, quarantining unparseable records, revoking orphans, and synchronizing cascade states.
2. **Safe Quarantine**: Any record failing structural deserialization is saved to `grants.quarantine.<timestamp>.json` rather than discarded.
3. **Strict Loop Detection**: Cycle detection via visited set during delegation tree validation.

## 4. Unknowns & Resolutions
- *Q: Should repair delete corrupt grants or mark them revoked?*
  - *Resolution*: Structural corruption (invalid JSON / schema) must be quarantined out of the live file to restore store parseability. Semantic issues (orphans, missing cascade) are safely repaired by marking status as `Revoked` with reason `"auto-recovery: orphaned parent"` or `"auto-recovery: cascade reconciliation"`.
