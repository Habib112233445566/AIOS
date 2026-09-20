# Task Evidence: T-01820 - Network Bootstrap / Core Service: Verification & Evidence

## Metadata
- **Task ID:** `T-01820`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`, `code/aiosh-rust/aiosh-core/tests/test_network_service.rs`, `code/aiosh-cli/tests/test_network_service_smoke.py`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Sub-Epic 2 Closure Summary

Sub-Epic 2 (Network Bootstrap / Core Service) encompasses tasks `T-01811` through `T-01820`. All 10 tasks are fully implemented, hardened, documented, and verified with 100% test pass rate.

### Task Matrix

| Task ID | Task Description | Status | Evidence |
|:---|:---|:---:|:---|
| `T-01811` | Research | **DONE** | `docs/tasks/evidence/T-01811-core-service-research.md` |
| `T-01812` | Specification | **DONE** | `docs/tasks/evidence/T-01812-core-service-specification.md` |
| `T-01813` | Scaffold | **DONE** | `docs/tasks/evidence/T-01813-core-service-scaffold.md` |
| `T-01814` | Implementation | **DONE** | `docs/tasks/evidence/T-01814-core-service-implementation.md` |
| `T-01815` | Unit Test | **DONE** | `docs/tasks/evidence/T-01815-core-service-unit-test.md` |
| `T-01816` | Integration | **DONE** | `docs/tasks/evidence/T-01816-core-service-integration.md` |
| `T-01817` | Security Review | **DONE** | `docs/tasks/evidence/T-01817-core-service-security-review.md` |
| `T-01818` | Hardening | **DONE** | `docs/tasks/evidence/T-01818-core-service-hardening.md` |
| `T-01819` | Documentation | **DONE** | `docs/tasks/evidence/T-01819-core-service-documentation.md` |
| `T-01820` | Verification & Evidence | **DONE** | `docs/tasks/evidence/T-01820-core-service-verification-evidenc.md` |

---

## 2. Invariants Formally Verified

- **`NSERV1`**: Hermetic mockability with custom paths.
- **`NSERV2`**: Graceful fallback on missing sysfs attributes.
- **`NSERV3`**: `/proc/net/route` little-endian hex parsing and netmask-to-CIDR conversion.
- **`NSERV4`**: `/etc/resolv.conf` comment stripping, nameserver and search domain parsing.
- **`NSERV5`**: Link state mutations (`bring_up`, `bring_down`) and interface name validation.
- **`NSERV6`**: Bounded file reads (`read_bounded_string`), bounded collections, and deterministic sorting.

---

## 3. Sub-Epic Formal Closure

Sub-Epic 2 (Network Bootstrap / Core Service) is hereby **FORMALLY CLOSED**.
Proceeding to Sub-Epic 3: Network Bootstrap / CLI Surface (`T-01821`..`T-01830`).
