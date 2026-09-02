# T-00576 — Evidence & Audit Trail / security policy: Integration

## 1. Integration Scope
This task verifies the integration of Evidence & Audit Trail security policies across the real CLI command runner, MCP tool dispatch pipelines, and CI security policy test suites.

## 2. Integrated Paths Tested
1. **CLI Surface**:
   - `aiosh evidence hash`, `aiosh evidence verify`, `aiosh evidence scan` execute unauthenticated while adhering strictly to repo-relative boundaries.
2. **MCP Surface**:
   - `aios.evidence.hash`, `aios.evidence.verify`, `aios.evidence.scan` route through standard JSON-RPC handlers.
3. **CI Policy Harness**:
   - `tools/check_security_policy.py` validates `SECURITY.md` against criteria S1..S5.

## 3. Verification Output
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)

All 8 evidence CLI unit and smoke tests passed successfully!
All 8 evidence MCP unit and smoke tests passed successfully!
```
