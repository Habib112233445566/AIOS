# T-00557 — Evidence & Audit Trail / configuration: Security Review

## 1. Overview
This security review evaluates the configuration subsystem (`EvidenceConfig`) of Evidence & Audit Trail, covering size bounds, path sanitization, and environment precedence controls.

## 2. Threat Scenarios & Mitigations

### A. Config Poisoning & Memory Exhaustion
- **Threat**: Supplying an oversized or malicious config file via `AIOS_EVIDENCE_CONFIG_PATH` to induce high memory consumption.
- **Evaluation**: `EvidenceConfig::from_path` checks file metadata length before opening and rejects any file exceeding 64 KiB (`MAX_CONFIG_BYTES`).

### B. Directory Traversal via Configured Directory
- **Threat**: Setting `evidence_dir` to absolute paths (`/`, `C:\Windows`) or parent traversal paths (`../../etc`).
- **Evaluation**: `validate()` explicitly rejects any path starting with `/`, containing `:`, or containing `..` path segments.

### C. Unbounded File Read Size Escalation
- **Threat**: Overriding `max_file_bytes` to unbounded values to crash hashing services with multi-gigabyte files.
- **Evaluation**: `validate()` caps `max_file_bytes` at 64 MiB (`MAX_FILE_SIZE_LIMIT = 67_108_864`) and rejects `0` byte limits.

### D. Extension Bypass & Wildcards
- **Threat**: Providing wildcard extensions or empty extension arrays to index unwanted binaries.
- **Evaluation**: `validate()` requires at least one extension and requires every entry to start with a leading `.`.

## 3. Findings & Verdict
The configuration layer enforces strict bounded memory consumption, containment to relative repo paths, and defensive sanitization. No policy bypasses remain open.
