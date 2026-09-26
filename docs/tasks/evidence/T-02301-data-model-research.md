# T-02301: Audit Chain Extensions Data Model Research

## 1. Objective & Scope
This research establishes the foundational facts, architectural constraints, prior art, and cryptographic data model requirements for **Audit Chain Extensions** in the AIOS Security Kernel & PEP Fabric.

## 2. Analysis of Existing Codebase & Prior Art

### A. Current Audit Implementation (`audit.rs`, `types.rs`)
- **Linear Hash Chain**:
  - Each row stores `prev_hash` and `hash = sha256_hex(prev_hash || canonical_json(proto))`.
  - Genesis block is defined by `GENESIS_HASH` (64 zeros).
  - Backed by SQLite WAL table `audit_ring` with incremental schema migrations (Sprint 1 basic schema $\to$ Sprint 2 classifier columns).
  - Rotated into `audit_segments` with Merkle/Bloom metadata in Sprint 3.
- **Identified Limitations of the Current Model**:
  1. *Flat Sequential Topology*: Complex multi-agent executions create branched or parallel tool calls where linear chaining loses causal parent-child delegation ancestry.
  2. *Cryptographic Non-Repudiation*: The current hash chain proves immutability and order across database rows, but lacks asymmetric signatures binding actions to specific cryptographic identities (e.g., Ed25519 signing keys held by isolated agent sandboxes or remote enclaves).
  3. *PEP Grant Coupling*: While `grant_token` existed as a loose string, Phase 2 establishes formal `PepGrant` identifiers, delegation depths, and attenuation paths that require structured provenance links in the audit record.
  4. *Extensible Payload Schema*: Custom security assertions, runtime attestations, and telemetry require typed, canonicalized extension envelopes without requiring constant DDL migrations for every new plugin.

### B. Authoritative Standards & Prior Art
1. **RFC 6962 (Certificate Transparency)**:
   - Binary Merkle Trees with leaf domain separation (`0x00 || leaf_data`).
   - Signed Tree Heads (STH) with asymmetric digital signatures.
2. **Sigstore / Rekor**:
   - Schema-validated pluggable entry types with canonical JSON envelope and ECDSA/Ed25519 signatures.
3. **W3C PROV-DM (Provenance Data Model)**:
   - Entities, Activities, Agents, and `wasDerivedFrom` / `wasInformedBy` causal relationships.

## 3. Core Architectural Decisions for AIOS Audit Chain Extensions
1. **Backwards-Compatible Proto Extension**:
   - Extended fields are optional or nullable in SQLite schema; omission of nulls in canonical JSON preserves hash identity for legacy records.
2. **First-Class Provenance Identifiers**:
   - `session_id`: Associates events with an active interaction boundary.
   - `pep_grant_id`: Binds PEP capability grant to the executed action.
   - `parent_row_hash` / `causal_parent_id`: Tracks subagent or delegation ancestry.
3. **Cryptographic Signatures**:
   - Add `AuditSignature` containing `algorithm` ("ed25519"), `public_key` (hex), and `sig` (hex).
4. **Canonical Hashing**:
   - Maintain the invariant: `hash = sha256_hex(prev_hash || canonical_json(extended_proto))`.

## 4. Unknowns & Resolutions
- *Q: Should signatures be computed over the raw proto or the row hash?*
  - *Resolution*: Signing the calculated SHA-256 row hash (`sig = sign(row_hash)`) is computationally optimal and prevents double-canonicalization while guaranteeing non-repudiation over the entire event payload.
