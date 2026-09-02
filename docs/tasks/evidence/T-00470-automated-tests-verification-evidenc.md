# T-00470 — Documentation Index Control / automated tests: Verification & Evidence

## 1. Verification Overview
This task concludes the automated tests sub-epic (T-00461..T-00470) for Documentation Index Control, capturing complete verification evidence across the unified criteria runner, unit suites, standalone CLI and MCP smokes, and CI orchestrator integration.

## 2. Test Execution Results

### A. Unified Criteria Runner (`tools/test_doc_index_suites.py`)
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

### B. Unit Test Suite (`tools/test_doc_index_unit.py`)
```text
[+] U01: D1 manifest valid serialization
[+] U02: D1 query filtering
[+] U03: D1 negative query match
[+] U04: D1 check function succeeds
[+] U05: D2 check function succeeds
[+] U06: D2 oversized config detected
[+] U07: D3 title H1 extraction
[+] U08: D3 inline relative link extraction excluding external URLs
[+] U09: D3 check function succeeds
[+] U10: D4 check function succeeds
[+] U11: D5 CLI subcommands check succeeds
[+] U12: D6 MCP surface check succeeds
[+] U13: D7 hardening limits check succeeds
[-] D1 manifest model & query helpers: intentional test failure
[+] D2 configuration hierarchy & limits
[+] D3 title parsing & link extraction
[+] D4 link integrity & traversal detection
[+] D5 CLI subcommand execution & json mode
[+] D6 MCP tool execution & protocol schemas
[+] D7 hardening limits & negative error bounds
[+] S01: Sensitivity proof -- failing checker causes test runner failure

PASS: all 14 doc_index unit tests green
```

### C. CLI Standalone Smoke (`code/aiosh-cli/tests/test_doc_cli_smoke.py`)
```text
PASS: aiosh doc show prose
PASS: aiosh doc show --json
PASS: aiosh doc check prose
PASS: aiosh doc check --json
PASS: aiosh doc search
PASS: aiosh doc search --json
PASS: aiosh doc invalid subcommand
PASS: aiosh doc search missing query
PASS: aiosh doc check broken link detection negative test
PASS: aiosh doc custom config valid
PASS: aiosh doc custom config missing negative test
PASS: test_doc_cli_smoke.py
```

### D. MCP Standalone Smoke (`code/aiosh-mcp/tests/test_doc_mcp_smoke.py`)
```text
PASS: aios.doc tools present in tools/list
PASS: aios.doc.index.get
PASS: aios.doc.check
PASS: aios.doc.search
PASS: aios.doc.search missing query negative test
PASS: test_doc_mcp_smoke.py
```

### E. CI Suite Registry (`tools/test_ci_suites.py`)
```text
[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)
```

### F. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

## 3. Sub-Epic Closeout
- Tasks Completed: T-00461 .. T-00470 (10/10 tasks).
- All criteria verified green.
- Next sub-epic begins at T-00471 (Security Policy for Documentation Index Control).
