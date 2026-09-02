# T-00527 — Evidence & Audit Trail / core service: Security Review

## 1. Overview
This security review evaluates the cryptographic hashing service, evidence artifact verification routines, and PEP policy gates for Evidence & Audit Trail.

## 2. Threat Scenarios & Mitigations

### A. Memory Exhaustion via Oversized Artifacts
- **Threat**: Supplying multi-gigabyte files to `compute_file_sha256` to cause memory exhaustion.
- **Evaluation**: `compute_file_sha256` inspects file metadata length before reading bytes and enforces a hard 16 MiB size cap (`MAX_DOC_BYTES`).

### B. Directory Traversal and Path Injection
- **Threat**: Relative path escapes (`../../`) or absolute filesystem paths targeting unauthorized files outside the repository.
- **Evaluation**: `build_evidence_record` enforces strict relative path assertions, rejecting any path starting with `/`, containing `:`, or containing `..` path segments.

### C. Mutating Evidence Injection & PEP Gating
- **Threat**: Modifying or injecting evidence records without authorization.
- **Evaluation**: Mutating actions (`aios.evidence.record`, `evidence.set`) are registered in `pep.rs::is_irreversible` and require verified PEP grant tokens.

### D. Audit Ring Immutability
- **Threat**: Invoking evidence verification without logging execution events.
- **Evaluation**: All CLI and MCP invocations route through `aiosh_core::audit::AuditRing` emitting structured SHA-256 hash-chained records.

## 3. Findings & Verdict
The core service mechanisms enforce strict bounded I/O, path confinement, PEP authorization, and complete audit logging. No vulnerabilities remain open.
