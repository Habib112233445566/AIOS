# Task Evidence: T-01819 - Network Bootstrap / Core Service: Documentation

## Metadata
- **Task ID:** `T-01819`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `docs/network_bootstrap.md`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Documentation Updates

Authored Section 5 in `docs/network_bootstrap.md` covering:
- **`NetworkService` Architecture**: Full description of constructors, discovery methods, route parsing, and DNS configuration.
- **Invariants `NSERV1..NSERV6`**: Complete breakdown of mockability, graceful degradation, hex decoding safety, DNS sanitization, link mutation safety, and bounded resource limits.
- **Resource Limits**: Documented file read caps (`MAX_SYSFS_FILE_BYTES = 64 KB`, `MAX_ROUTE_FILE_BYTES = 1 MB`, `MAX_RESOLV_FILE_BYTES = 64 KB`).
