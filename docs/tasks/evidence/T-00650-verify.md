# T-00650 — Repository Health / MCP/API surface: Verification & Evidence

## 1. Sub-Epic Closure Summary
- **Component**: Phase 0 / Repository Health (`T-00611..T-00710`)
- **Sub-Epic 4**: `MCP/API surface` (`T-00641..T-00650`)
- **Status**: 10/10 Tasks COMPLETE
- **Deliverables**:
  - `aios.repo.health` MCP tool registered in `tool_manifest()` and handled in `call_tool()`.
  - JSON-RPC tool smoke test runner `code/aiosh-mcp/tests/test_repo_mcp_smoke.py` (3/3 PASS).
  - Security threat review MCP-1..MCP-3 with 0 open vulnerabilities.
  - Documentation updated in `docs/README.md`.

## 2. Multi-Suite Test Matrix Run Log
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

[+] E1 directory-health: found 1351 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1351 files bounded and valid UTF-8
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
