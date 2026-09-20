# Task Evidence: T-02011 - Capability Model / core service: Research (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02011`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Establish facts, constraints, prior art, and architectural invariants for the Capability Model core service (`CapabilityService`).

---

## 2. Prior Art & Industry Architecture

1. **seL4 CNode and Capability Space**:
   - The seL4 kernel maintains CNode capability tables representing addressable capability spaces (CSpace).
   - The CDT (Capability Derivation Tree) tracks parent-child derivations to support deterministic, recursive revocation.
2. **E-Rights / Object-Capability Operating Systems (KeyKOS, EROS, Coyotos)**:
   - Capability directories hold references to capabilities for subjects.
   - Revocation pattern via revocable forwarders / caretakers: revoking the caretaker disables access for all downstream holders.
3. **AIOS Security Kernel Service Layer**:
   - `CapabilityService` acts as the authoritative custodian of active capabilities.
   - Maintains an in-memory index by ID and subject.
   - Handles root issuance, managed attenuation, transitive cascade revocation, and persistence.

---

## 3. Core Architectural Invariants (`CSERV1..CSERV6`)

1. **`CSERV1` (Registry Indexing & Retrieval)**:
   - Fast $O(1)$ retrieval by capability ID, and indexed retrieval of all active capabilities granted to a subject (`get_capabilities_for_subject`).
2. **`CSERV2` (Authorized Root Issuance)**:
   - Root capabilities (`parent_id: None`) can only be issued by the Security Kernel or authorized administrative principals.
3. **`CSERV3` (Atomic Attenuation & Lineage Registration)**:
   - Calling `attenuate_capability(parent_id, new_subject, ...)` validates the parent in the registry, constructs the child, links `parent_id`, and registers the child in a single atomic transaction.
4. **`CSERV4` (Cascade Transitive Revocation)**:
   - Revoking a capability automatically traverses the derivation graph and revokes all transitive child capabilities derived from it.
5. **`CSERV5` (Atomic Persistence & Bounded Storage)**:
   - Registry serialization to disk enforces symlink rejection, 10 MB size limits, and `.tmp.<pid>` atomic file replacement.
6. **`CSERV6` (Pruning & Garbage Collection)**:
   - Safe pruning of expired capabilities while maintaining lineage tombstones to prevent parent ID reuse or resurrection.
