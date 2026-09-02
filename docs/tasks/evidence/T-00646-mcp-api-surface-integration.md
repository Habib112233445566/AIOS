# T-00646 — Repository Health / MCP/API surface: Integration

## 1. Integration Scope
This task tests cross-substrate integration between `aiosh-mcp`, `aiosh-cli`, and core baseline invariant test suites.

## 2. Integrated Suites Executed
1. **`code/aiosh-mcp/tests/test_repo_mcp_smoke.py`**:
   - `tools/list` and `tools/call` JSON-RPC execution (PASS).
2. **`code/aiosh-cli/tests/test_repo_cli_smoke.py`**:
   - `aiosh repo health` and check commands (PASS).
3. **`tools/check_security_policy.py`**:
   - S1..S5 criteria (PASS).
4. **`tools/check_task_docs.py`**:
   - C1..C6 criteria (PASS).
5. **`tools/check_evidence.py`**:
   - E1..E4 criteria across 1,339 evidence files (PASS).
6. **`tools/test_ci_suites.py`**:
   - W1..W7 criteria (PASS).

## 3. Verification Output
```text
PASS: aios.repo.health present and valid in tools/list
PASS: aios.repo.health tool call on repository root
PASS: aios.repo.health tool call on temp directory

ALL MCP REPO HEALTH SMOKE TESTS PASSED!
PASS: aiosh repo health prose output
PASS: aiosh repo health --json output
PASS: aiosh repo check alias
PASS: aiosh repo health --repo custom path
PASS: aiosh repo invalid subcommand rejection

ALL REPO CLI SMOKE TESTS PASSED!
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
[+] E1 directory-health: found 1339 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1339 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)
```
