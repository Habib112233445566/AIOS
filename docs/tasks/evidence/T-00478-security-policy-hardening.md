# T-00478 — Documentation Index Control / security policy: Hardening

## 1. Hardening Overview
This task hardens the security policy enforcement and governance subsystem for Documentation Index Control against policy bypasses, invalid token formats, silent failures, and resource exhaustion.

## 2. Hardening Measures
1. **Fail-Closed Token Validation**:
   - `check_doc_index_policy` trims candidate grant strings and rejects empty or whitespace-only values (`""`, `"   "`).
   - Any unauthenticated attempt to execute mutating commands produces an explicit `Err` string detailing the policy violation and requirement for a PEP token.
2. **Audit Logging Invariance on Policy Denial**:
   - Policy rejections in CLI and MCP handlers write structured audit rows with `outcome: "refused"` / `outcome: "error"` rather than failing silently.
3. **Automated CI Policy Invariant Guarding**:
   - `tools/check_security_policy.py` runs in the automated CI pipeline to ensure policy statements, reporting URLs, and reference links cannot be silently deleted or corrupted.

## 3. Verification Output
```text
running 1 test
test doc_index_service::tests::test_check_doc_index_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out; finished in 0.01s
```
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```
