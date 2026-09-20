# Task Evidence: T-02001 - Capability Model / data model: Research (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02001`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Research facts, constraints, prior art, and core architectural invariants for the AIOS Capability Model data model.

---

## 2. Prior Art & Industry Standards

1. **seL4 Microkernel Capabilities**:
   - First-class, unforgeable capability pointers stored in protected kernel CNodes.
   - Core properties: Attenuation (deriving weaker capabilities), Revocation trees (cascade revocation of delegated child capabilities), and Badging (immutable endpoint tagging).
2. **Capsicum (FreeBSD)**:
   - File-descriptor based capability rights bitmask (`CAP_READ`, `CAP_WRITE`, `CAP_CONNECT`, `CAP_BIND`).
   - Ambient authority elimination: processes enter capability mode where global namespaces cannot be accessed directly.
3. **WASI Component Model (Preview 2)**:
   - Resource handles passed explicitly to WebAssembly components rather than relying on ambient environment authority.
4. **Zircon / Fuchsia OS**:
   - Handle-based capability system with granular right masks (`ZX_RIGHT_READ`, `ZX_RIGHT_WRITE`, `ZX_RIGHT_EXECUTE`, `ZX_RIGHT_DUPLICATE`, `ZX_RIGHT_TRANSFER`).

---

## 3. Core Architectural Invariants for AIOS Capability Model (`CAP1..CAP6`)

1. **`CAP1` (Cryptographic Unforgeability & Identity)**:
   - Every capability possesses a globally unique identifier (UUIDv4 or cryptographic 256-bit token), an issuer identifier, issuance timestamp, and optional cryptographic signature.
2. **`CAP2` (Target & Rights Scoping)**:
   - Capabilities are strictly scoped to a target URI/resource namespace (e.g. `fs:///var/data`, `net://api.aios.org:443`, `tool://session.create`, `ipc://queue/worker`) and a set of explicit rights/actions (`Read`, `Write`, `Execute`, `Admin`, `Delegate`).
3. **`CAP3` (Monotonic Attenuation)**:
   - A child capability derived from a parent capability cannot possess rights or resource scopes exceeding the parent. Rights can only be narrowed, never amplified.
4. **`CAP4` (Temporal & Quota Constraints)**:
   - Optional lifecycle boundaries including `not_before`, `expires_at`, max execution/invocation count, and byte throughput quotas.
5. **`CAP5` (Lineage & Cascade Revocation)**:
   - Parent-child relationship tracking (`parent_id`) allowing the Security Kernel to perform transitive revocation across delegated capability trees.
6. **`CAP6` (Serialization & Type Safety)**:
   - High-fidelity, deterministic JSON and CBOR serialization with strict validation, zero ambient authority defaults, and cross-platform compatibility across Rust core and Python MCP tooling.
