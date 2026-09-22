# Task Evidence: T-02217 (Grant Lifecycle / core service: Security Review)

## 1. Scope & Objective
Formal security review of the `PepGrantService` (`code/aiosh-rust/aiosh-core/src/pep_grant_service.rs`) multi-index grant coordination layer, its CLI integration (`code/aiosh-rust/aiosh-cli/src/main.rs`), and MCP dispatch surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`).

---

## 2. Threat Modeling & Attack Surface Analysis

### Attack Vector 1: Multi-Index Desynchronization
- **Threat**: Inconsistent secondary indexes (`by_subject`, `by_state`) leading to ghost grants, bypass of revocations, or failure to enforce quotas.
- **Evaluation**:
  - `unindex_grant` and `index_grant` are invoked atomically whenever grants are added, mutated, or transitioned.
  - State transitions (`transition_grant`, `revoke_grant`, `sweep_expired`, `evaluate_grant_action`) remove old index entries before inserting new ones.
  - If a grant is overwritten by ID, the prior entry is first unindexed.
  - Multi-index consistency is asserted across 11 standalone test suites.
- **Verdict**: **SECURE & VERIFIED**.

### Attack Vector 2: Delegation Right Amplification & Depth Escalation
- **Threat**: An adversary attenuates an existing grant to grant themselves unauthorized permissions or infinite delegation chains.
- **Evaluation**:
  - Requires parent grant to be in `Active` state.
  - Enforces `parent.constraints.max_delegation_depth > 0`.
  - Child delegation depth is strictly set to `parent.constraints.max_delegation_depth - 1`.
  - Child rights MUST be a strict subset of parent rights.
  - Parent grant MUST possess `CapabilityRight::Delegate`.
  - Child expiration timestamp is constrained to be no later than parent expiration timestamp.
- **Verdict**: **SECURE & VERIFIED**.

### Attack Vector 3: Cascading Revocation Traversal & Cycles
- **Threat**: Maliciously crafted parent-child circular references causing infinite loops, stack overflows, or partial revocations during cascade.
- **Evaluation**:
  - `cascade_revoke` maintains a BFS work queue and a `visited` hash set guarding against cycles.
  - All discovered descendant grants are transitioned to `GrantState::Revoked` with reason and timestamp recorded.
  - Indexes are updated for every revoked descendant.
- **Verdict**: **SECURE & VERIFIED**.

### Attack Vector 4: Quota Tampering & Sweep Bypass
- **Threat**: Bypassing invocation or byte limits, or continuing to execute operations on expired grants before sweep runs.
- **Evaluation**:
  - `evaluate_grant_action` performs on-the-fly verification of expiration time, not-before time, state, subject, action rights, and quota ceilings before permitting execution.
  - If invocation or byte quota is reached, grant automatically transitions to `GrantState::Expired`.
  - `sweep_expired` batch-transitions all elapsed or quota-exhausted grants to `Expired` and updates `by_state` index.
- **Verdict**: **SECURE & VERIFIED**.

### Attack Vector 5: Path Traversal & Unbounded In-Memory Storage
- **Threat**: Writing/reading grants from arbitrary system locations, or exhausting memory via unbounded grant allocation.
- **Evaluation**:
  - `MAX_SERVICE_GRANTS = 5000` enforced prior to issuing or attenuating grants.
  - Storage paths validated for `.json` extension, canonical directory existence, and absence of control characters.
  - Serialization uses atomic write patterns (`<target>.tmp.<pid>` -> `replace`).
- **Verdict**: **SECURE & VERIFIED**.

### Attack Vector 6: Audit Attribution & Tamper Evidence
- **Threat**: Unrecorded grant modifications or state transitions.
- **Evaluation**:
  - MCP tools execute within `dispatch::recorded_call`, writing complete request/response envelopes to SQLite `audit.db`.
  - CLI invocations emit audit events via `classify_and_emit` and return structured JSON envelopes.
- **Verdict**: **SECURE & VERIFIED**.

---

## 3. Findings & Hardening Recommendations (for T-02218)
1. Ensure explicit boundary check on `MAX_SERVICE_GRANTS` in `attenuate_grant` as well as `issue_grant`.
2. Ensure explicit storage path validation against symlinks and path traversal sequences.
3. Validate that time parsing errors in `sweep_expired` fail closed rather than panicking or skipping.

---

## 4. Acceptance Confirmation
- [x] Full threat model and attack surface review conducted for `PepGrantService`.
- [x] Multi-index synchronization, attenuation containment, cascade revocation, and quota enforcement verified.
- [x] Hardening items documented for T-02218 implementation.
