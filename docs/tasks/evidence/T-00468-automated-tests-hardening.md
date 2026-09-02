# T-00468 — Documentation Index Control / automated tests: Hardening

## 1. Hardening Overview
This task validates and strengthens the hardening protections across the Documentation Index Control automated test harnesses (`tools/test_doc_index_suites.py`, `tools/test_doc_index_unit.py`, `code/aiosh-cli/tests/test_doc_cli_smoke.py`, and `code/aiosh-mcp/tests/test_doc_mcp_smoke.py`).

## 2. Hardening Measures
1. **Bounded Subprocess Execution & Zombie Process Reap**:
   - `tools/test_doc_index_suites.py`: Enforces 15s wall-clock timeouts on all CLI and MCP subprocess invocations. On timeout, the child process is terminated with `p.kill()` followed by `p.wait()` to prevent orphan process leaks.
   - `code/aiosh-cli/tests/test_doc_cli_smoke.py`: Bounded with 30s timeouts on CLI executions.
   - `code/aiosh-mcp/tests/test_doc_mcp_smoke.py`: Bounded with 30s timeout on stdio communication with explicit child kill on timeout.
2. **Guaranteed Temp File Cleanup**:
   - All tests creating temporary config files (`tempfile.NamedTemporaryFile`, `tempfile.TemporaryDirectory`) use strict `try ... finally` blocks to ensure immediate file unlinking and eliminate disk/temp leaks.
3. **Structured Error Envelopes**:
   - Negative tests for missing paths, invalid subcommands, missing query parameters, and oversized configs (>64 KiB) verify standard structured error envelopes (`{"ok": false, ...}`) without silent failures.

## 3. Verification Output
```text
[+] D1 manifest model & query helpers
[+] D2 configuration hierarchy & limits
[+] D3 title parsing & link extraction
[+] D4 link integrity & traversal detection
[+] D5 CLI subcommand execution & json mode
[+] D6 MCP tool execution & protocol schemas
[+] D7 hardening limits & negative error bounds

PASS: doc_index test criteria (D1..D7)
```
