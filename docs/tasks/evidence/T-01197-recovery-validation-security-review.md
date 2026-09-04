# T-01197: Base Image Build Recovery & Validation Security Review

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01197  

## 1. Security Review & Threat Analysis

### A. Memory & Resource Consumption (OOM / DoS Defenses)
- **Threat**: An attacker passes a gigantic corrupted file (>10 GB) to trigger unbounded memory allocation during parsing.
- **Mitigation**: `ImageStore::load_from_path` enforces a hard file size cap of 10 MiB before reading into memory. Files exceeding this ceiling are rejected immediately with an auditable error.

### B. Path Traversal & Backup Pollution
- **Threat**: Malicious `store_path` arguments attempt to manipulate external directories via `../` path traversal.
- **Mitigation**: MCP tool `aios.image.check` validates `store_path` length ($\le 4096$ chars) and rejects non-graphic/control characters. The backup file naming pattern (`<path>.bak.<timestamp>`) appends directly to the target path without evaluating relative escapes or directory traversing.

### C. Forensic Preservation & Anti-Tampering
- **Threat**: An adversary intentionally introduces corrupt manifests to trick the auto-repair mechanism into destroying audit logs or forensic evidence.
- **Mitigation**: Recovery is unconditionally non-destructive (`RV4`). Before reseeding with canonical profiles, `load_or_recover` copies the exact corrupt payload to `<path>.bak.<timestamp>`. Furthermore, every recovery invocation records an immutable SHA-256 hash-chained event to SQLite WAL (`audit.db`).

### D. Information Disclosure Audit
- **Threat**: Error strings in `BaseImageValidationReport` echo sensitive internal state or secrets.
- **Mitigation**: Validation errors are strictly scoped to manifest schema fields (e.g. invalid package names, dangerous kernel parameters, unsupported filesystems). No secret values or sensitive system environment variables are exposed.

## 2. Security Review Verdict
No security vulnerabilities or policy bypasses identified. Implementations adhere strictly to PEP authorization and SQLite WAL audit logging invariants.
