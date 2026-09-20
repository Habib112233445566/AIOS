# Task Evidence: T-02017 - Capability Model / core service: Security Review (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02017`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Perform comprehensive security review and threat modeling for `CapabilityService`.

---

## 2. Threat Model Analysis (THREAT-CSERV-01..06)

| Threat ID | Threat Description | Attack Vector / Trigger | Severity | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-CSERV-01` | **Unauthorized Root Capability Issuance** | Non-privileged caller invokes `issue_root_capability` without kernel authorization. | Critical | Restrict root capability issuance to authorized issuers (`kernel` or `admin:*`); reject ambient root grants. |
| `THREAT-CSERV-02` | **Lineage Cycle Denial of Service** | Malicious or corrupted store containing cyclic parent-child links induces infinite loop in `revoke_capability`. | High | Track `visited: HashSet<String>` in BFS queue during cascade revocation to guarantee loop termination. |
| `THREAT-CSERV-03` | **Registry Memory Exhaustion (OOM)** | Adversary floods the registry with millions of capabilities to exhaust host memory. | High | Enforce a hard ceiling `MAX_CAPABILITIES_IN_REGISTRY = 10_000` in `CapabilityService`. |
| `THREAT-CSERV-04` | **Storage Path Traversal & Injection** | Caller passes `../../etc/shadow` to `save_to_path` or `load_from_path`. | High | Implement `validate_service_path`: reject `..`, control characters, require `.json`, length $\le 1024$. |
| `THREAT-CSERV-05` | **Symlink Hijacking during Persistence** | Target file is a symlink pointing to sensitive system file. | High | Check `symlink_metadata()` and refuse to read or write symlinked files. |
| `THREAT-CSERV-06` | **Temporary File Leakage on Write Failure** | Failure during atomic write leaves orphaned `.tmp.<pid>` files on disk. | Low | Wrap write operations in error handlers that proactively remove temporary files before returning `Err`. |

---

## 3. Hardening Plan for T-02018
1. Implement `validate_service_path(path: &Path) -> Result<(), String>`.
2. Enforce `MAX_CAPABILITIES_IN_REGISTRY = 10_000` in `CapabilityService`.
3. Add cycle detection (`HashSet<String>`) in `revoke_capability`.
4. Validate issuer in `issue_root_capability` (`kernel` or `admin:` prefix).
5. Clean up temporary files on write failure in `save_to_path`.
