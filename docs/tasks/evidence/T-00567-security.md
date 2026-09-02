# T-00567 — Evidence & Audit Trail / automated tests: Security Review

## 1. Security Review Scope
This task conducts a threat model and abuse analysis of the Evidence & Audit Trail automated test harness (`tools/check_evidence.py`, `tools/test_check_evidence.py`, CLI smoke tests, and MCP tool handlers).

## 2. Threat Model & Abuse Scenarios

### Scenario A1: Path Traversal & Out-of-Tree File Access
- **Threat**: An adversary attempts to pass path traversal sequences (e.g. `../../etc/passwd` or `..\..\Windows\System32`) to `aiosh evidence hash` or `aios.evidence.hash`.
- **Finding & Mitigation**: 
  - `aiosh-core`'s `evidence_service` strictly resolves paths against the canonical repo root using `canonicalize()` and verifies containment.
  - Non-existent or inaccessible targets fail loudly returning `{ "ok": false, "error": "..." }` with exit code 1, never leaking out-of-tree file content.

### Scenario A2: Resource Exhaustion & Denial of Service (OOM)
- **Threat**: Submitting an oversized or unbounded file stream (e.g., gigabyte-scale logs or sparse files) to exhaust memory during SHA-256 computation.
- **Finding & Mitigation**:
  - `tools/check_evidence.py` enforces `MAX_DOC_BYTES = 16 * 1024 * 1024` (16 MiB max).
  - SHA-256 computation streams files in 64 KiB chunks (`f.read(65536)`), guaranteeing $O(1)$ memory consumption regardless of file length.

### Scenario A3: Artifact Poisoning & Encoding Attacks
- **Threat**: Injecting invalid UTF-8 byte sequences, null-bytes, or empty stub files to falsify task completion without actual deliverables.
- **Finding & Mitigation**:
  - `check_e3_file_bounds()` strictly tests UTF-8 decoding on every file and flags zero-byte files as fatal errors.
  - `tools/test_check_evidence.py` explicitly tests empty files (U10) and invalid non-UTF-8 bytes (U12) to prevent false passes.

### Scenario A4: Authorization & Audit Trail Transparency
- **Threat**: Bypassing PEP security gates or executing mutating operations without writing audit logs.
- **Finding & Mitigation**:
  - All evidence inspection actions (`hash`, `scan`, `verify`) are grant-free and read-only.
  - Every MCP execution emits standard JSON-RPC response structures. Mutating ledger operations (`tools/complete_task.py`) write deterministic events to `COMPLETIONS.jsonl` and append-only hash chains.

## 3. Verdict
- **Status**: PASS
- **Open Policy Bypasses**: 0
- **Residual Risks**: None identified for automated testing harness.
