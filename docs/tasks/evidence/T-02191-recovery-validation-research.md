# Research Report: PEP Decision Engine Recovery & Validation (T-02191)

## 1. Executive Summary & Context
Sub-Epic 10 investigates the recovery and validation capabilities of the AIOS PEP Decision Engine (`aiosh_core::pep_recovery`). In high-assurance security environments, the Policy Enforcement Point (PEP) and Policy Decision Point (PDP) must never execute in an ambiguous or corrupt state. If a policy file is partially written, maliciously tampered with, syntactically malformed, or logically contradictory, the engine must safely detect anomalies, validate structural and semantic constraints, non-destructively quarantine bad state, and provide auditable recovery mechanisms without violating fail-closed security invariants.

---

## 2. Review of Existing Codebase
1. **Current Persistence Mechanisms**:
   - `PepDecisionService::save_to_path(&Path)`: Writes formatted JSON to `.tmp.<pid>` and performs an atomic rename. Validates paths against traversal (`..`, control chars, length > 1024).
   - `PepDecisionService::load_from_path(&Path)`: Validates file size ($\le 10 \text{ MiB}$), reads to string, and deserializes using Serde JSON.
   - `PepDecisionService::load_or_recover(&Path)`: When deserialization fails, renames the damaged file to `<name>.bak.<timestamp>` (mode `0600` on Unix) and instantiates a clean, empty store.
2. **Current Limitations Identified**:
   - **Coarse All-or-Nothing Quarantine**: If 1 rule out of 5,000 has a syntax or schema typo, `load_or_recover` discards all 5,000 rules into quarantine and boots an empty store.
   - **No Deep Semantic Validation**: Only tests if Serde deserialization succeeds. Does not validate semantic rule logic (e.g. invalid wildcards, duplicate IDs, circular references, or restricted resource privilege violations).
   - **No Checksum/Integrity Hash**: Cannot detect bit-rot or silent external disk corruption before parsing.
   - **No Dedicated Recovery / Inspection Tooling**: Operators have no CLI (`aiosh pep validate`, `aiosh pep recover`) or MCP (`aios.pep.validate`, `aios.pep.recover`) tools to inspect corruption, diagnose invalid rules, or salvage valid rules.

---

## 3. Authoritative Sources & Standards
1. **RFC 2904: AAA Authorization Framework**:
   - Establishes that policy repositories must maintain cryptographic integrity and fail-closed state during retrieval errors.
2. **XACML 3.0 Core Specification (§7: Policy Administration and Retrieval)**:
   - Mandates strict schema compliance for policy assertions.
   - Requires deterministic conflict resolution when corrupted or overlapping policies are encountered.
3. **NIST SP 800-162: Guide to Attribute Based Access Control (ABAC)**:
   - Section 4.2 emphasizes policy validation (syntax, semantic consistency, and conflict detection) before rule deployment into active decision points.
4. **AIOS Architecture Decision Records (ADR-0035 & ADR-0036)**:
   - §D-2: Immutable audit logs and strict fail-closed enforcement.
   - §F-2: Honest audit logging on all recovery/fallback paths.

---

## 4. Fact vs. Assumption Separation

| Topic | Fact | Assumption |
|---|---|---|
| **Integrity Checks** | Serde JSON parses valid syntax, but does not verify cryptographic hashes or logical rule consistency. | Computing a SHA-256 checksum over the policy payload provides tamper detection with negligible performance overhead. |
| **Quarantine Storage** | Files are copied to `.bak.<timestamp>` with mode `0600` on Unix. | Quarantined files must remain untouched by automatic cleanup until explicit operator intervention or salvage. |
| **Partial Salvage** | Discarding all rules on single-rule corruption causes total denial of service for all users. | A rule-by-rule salvage parser can safely recover well-formed rules into a repaired store while quarantining invalid rules. |
| **Audit Emissions** | All state-changing events must log to the SQLite audit ring. | Corrupted file detections and salvage operations must generate dedicated audit events with error details and file hashes. |

---

## 5. Decisions Needed Before Specification (`T-02192`)
1. **Validation Engine Scope**: Should validation be exposed as a standalone module `pep_recovery.rs` in `aiosh-core` that operates on raw JSON, file paths, and `PepDecisionService` instances?
   - *Decision*: Yes, implement `aiosh_core::pep_recovery` with `PepStoreValidator`, `PepValidationReport`, and `PepRecoveryManager`.
2. **Recovery Strategies**: What recovery strategies should be supported?
   - *Decision*: Support three explicit recovery strategies:
     - `StrictFailClosed`: Reject entire store and quarantine if any rule is corrupt.
     - `SalvageValidRules`: Extract valid rules, isolate corrupt rules into a `.dropped` manifest, and save the repaired store.
     - `DryRun`: Validate and report errors without writing any changes to disk.
3. **Integrity Metadata**: Should the policy file format support an optional integrity envelope (e.g. `{"version": 1, "sha256": "...", "rules": [...]}`)?
   - *Decision*: Yes, support both raw rule arrays / legacy stores and versioned integrity envelopes with SHA-256 verification.

---

## 6. Acceptance Confirmation
- [x] Evidence file exists and explicitly separates facts from assumptions.
- [x] No source code changed in this task.
- [x] Authoritative citations and required decisions documented.
