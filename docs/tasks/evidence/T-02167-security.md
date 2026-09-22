# Security Review & Abuse Scenarios: T-02167

## Scope
Security review of PEP Decision Security Policy Subsystem (`pep_security_policy.rs`, CLI and MCP rule addition gates).

## Abuse Scenarios Evaluated
1. **Abuse Scenario 1**: Attacker calls `aiosh pep rule-add --resource sys:kernel:module --effect permit` without `--privileged`.
   - Result: Rejected with exit code 2 (`POLICY_VIOLATION`), failure audit row emitted in SQLite.
2. **Abuse Scenario 2**: Attacker calls `aios.pep.rule_add` with `resource: "sys:kernel:module", effect: "permit"`.
   - Result: Rejected with `PEPPOL_ERR_PRIVILEGE`, audit row emitted via `recorded_call`.
3. **Abuse Scenario 3**: Attacker calls `rule-add` targeting non-restricted resources (`app:*`).
   - Result: Permitted, normal operation preserved.
4. **Abuse Scenario 4**: Attacker triggers obligation delivery failure under Strict criticality.
   - Result: Converted to Deny (`allowed: false`).

## Verdict
Zero policy bypasses open. Finding N-35 resolved. Security review PASSED.
