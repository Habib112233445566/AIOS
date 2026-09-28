# Task Evidence: T-02391 - Audit Chain Extensions: Recovery & Validation Research

## Goal
Establish facts, constraints, failure modes, and architectural patterns for the Audit Chain Extensions recovery & validation subsystem.

## Research Findings

### 1. Invariants & Failure Modes of Audit Chains
An immutable SHA-256 hash ring and causal DAG audit chain can encounter several classes of failure:
1. **Cryptographic Hash Discontinuity**:
   - Row $k$ contains a `previous_hash` that fails to match the computed SHA-256 payload of row $k-1$. This indicates data corruption, disk bit-rot, or malicious truncation/tampering.
2. **Corrupted Extended JSON Fields**:
   - Truncated or malformed JSON payloads in `provenance_json`, `causal_links_json`, `signature_json`, or `extensions_json`.
3. **Cryptographic Signature Mismatch**:
   - An Ed25519 digital signature attached to an event fails cryptographic verification against the actor's public key.
4. **Dangling Causal Lineage References**:
   - A causal link points to a parent event hash or ID that does not exist in the ring database.
5. **Causal Lineage Cycles**:
   - Directed cycles within the DAG structure violating acyclic invariants.
6. **Temporal Monotonicity Inversions**:
   - Child events timestamped earlier than parent trigger events beyond allowable clock drift.

### 2. Prior Art in AIOS Core
- Analyzed `pep_grant_recovery.rs` and `pep_recovery.rs`:
  - Diagnostic severity levels: `Fatal`, `Error`, `Warning`.
  - Structured issue codes (`AuditChainIssueCode`).
  - Validation report (`AuditChainValidationReport`).
  - Atomic point-in-time database snapshotting before any repair action.
  - Forward-repair pattern: repairing the chain tip by emitting a synthetic repair anchor event that seals previous corruption without rewriting historical ledger rows (preserving non-repudiation and forensic evidence).

### 3. Decisions & Contracts
- **Non-Destructive Guarantee**: Recovery must never silently drop or rewrite existing rows without creating a pre-flight backup snapshot.
- **Audit-on-Repair**: Any execution of recovery or repair writes a formal audit record (`outcome: "recovery:repair"`).
- **Tool Surface**: CLI will expose `aiosh audit repair` and `aiosh audit validate`; MCP will expose `aios.audit.repair` and `aios.audit.validate`.
