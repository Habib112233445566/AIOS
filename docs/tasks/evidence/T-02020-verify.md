# Task Evidence: T-02020 - Capability Model / core service: Verification & Evidence (Sub-Epic 2 Formal Closure)

## 1. Overview
- **Task ID**: `T-02020`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Formally verify the CapabilityService across all invariants (CSERV1..CSERV6) and close Sub-Epic 2.

---

## 2. Verification Summary

### 2.1 Invariant Verification
| Invariant | Description | Verification Status |
|---|---|---|
| `CSERV1` | Registry indexing by ID, subject, and parent | Verified via `test_cserv1_root_issuance_and_indexing` |
| `CSERV2` | Root capability issuance with kernel authorization | Verified via `test_cserv7_hardening_issuer_authorization` |
| `CSERV3` | Monotonic attenuation and lineage tracking | Verified via `test_cserv2_attenuation_and_lineage` |
| `CSERV4` | Transitive cascade revocation with cycle detection | Verified via `test_cserv3_cascade_revocation` & `test_cserv9_hardening_cycle_detection_in_revoke` |
| `CSERV5` | Safe atomic persistence with path validation | Verified via `test_cserv5_persistence_atomic_roundtrip` & `test_cserv8_hardening_path_validation` |
| `CSERV6` | Expired capability pruning | Verified via `test_cserv6_prune_expired` |

### 2.2 Test Results
- **Rust Unit Tests**:
  - Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_service`
  - Result: 9 passed, 0 failed, 0 ignored.
- **Python Integration/Smoke Suite**:
  - Command: `python code/aiosh-mcp/tests/test_capability_service_smoke.py`
  - Result: All 4 smoke checks passed.

---

## 3. Sub-Epic 2 Formal Closure
Sub-Epic 2 (`core service`, T-02011 through T-02020) is hereby formally closed. All invariants are enforced, tested, hardened, and verified with zero regressions.
