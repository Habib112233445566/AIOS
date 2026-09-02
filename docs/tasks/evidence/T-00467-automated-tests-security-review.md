# T-00467 — Documentation Index Control / automated tests: Security Review

## 1. Overview
This review evaluates the security properties, threat vectors, input validation, and hardening guarantees of the automated test harnesses (`tools/test_doc_index_suites.py`, `tools/test_doc_index_unit.py`, `code/aiosh-cli/tests/test_doc_cli_smoke.py`, `code/aiosh-mcp/tests/test_doc_mcp_smoke.py`) created for Documentation Index Control.

## 2. Abuse Scenarios & Threat Vectors

### A. Subprocess Command & Argument Injection
- **Threat**: Malicious parameters in doc search queries or paths could attempt shell metacharacter injection (`|`, `;`, `&&`, `$()`).
- **Mitigation**: All subprocess calls across the test harnesses use vectorized argument lists (`[bin_path, "doc", "search", query]`) with `shell=False`. Arguments are passed directly to execve/CreateProcess without shell interpretation.

### B. Insecure Temporary File Handling
- **Threat**: Temporary test configuration files could be hijacked via symlink attacks or predictable paths.
- **Mitigation**: Tests exclusively use `tempfile.NamedTemporaryFile` and `tempfile.TemporaryDirectory` with randomized paths and strict permissions, wrapped in guaranteed cleanup `finally:` blocks and context managers.

### C. Path Traversal & Out-of-Bounds Discovery
- **Threat**: Indexing configurations or markdown link references attempt to traverse outside repository root using `../` components (e.g. `../../../../etc/passwd`).
- **Mitigation**: `DocIndexConfig::validate()` strictly forbids `..` in root directories, and `validate_doc_links` verifies all link targets resolve strictly within repository boundaries. Negative test cases in D4 and D7 assert that attempts to escape repository root are flagged and rejected.

### D. Resource Exhaustion & Large File Denial of Service
- **Threat**: Ingestion of huge files (> 100 MB) or deeply recursive directory trees could exhaust CPU and memory during indexing or test runs.
- **Mitigation**: Config file ingestion is capped at 64 KiB (`MAX_CONFIG_BYTES`), and document file ingestion is capped at 16 MiB (`MAX_DOC_READ_BYTES`). Test harnesses enforce strict 15s timeouts on subprocess executions.

### E. Audit Logging & Policy Enforcement
- **Threat**: Test executions bypass PEP token validation or avoid writing required audit records.
- **Mitigation**: All CLI and MCP doc commands route through the central dispatch pipeline with structured JSON audit records emitted for both success and refusal/error outcomes.

## 3. Findings & Verdict
All evaluated threat vectors are addressed with active mitigations and negative test cases. No security policy bypasses or resource exhaustion vectors exist in the test harness.
