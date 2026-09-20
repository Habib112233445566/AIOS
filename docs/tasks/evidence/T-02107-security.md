# Security Review: T-02107 (PEP Decision Engine / data model: Security Review)

## 1. Threat Model for PEP Decision Engine Data Model

| Threat ID | Threat Description | Attack Vector | Severity | Mitigation |
|---|---|---|---|---|
| `THREAT-PEPDEC-01` | Pattern matching bypass via path evasion | Caller passes `fs:/secret/../secret/file` or redundant slashes `fs://secret//file` to bypass prefix rule `fs:/secret/*`. | **HIGH** | Resource URIs must be normalized via `normalize_path_str()` prior to pattern evaluation. Reject `..` components in resource URIs. |
| `THREAT-PEPDEC-02` | Combining algorithm ambiguity | Caller specifies unsupported or conflicting combining algorithm, causing unexpected permit. | **HIGH** | Strict default to `DenyOverrides` (`PEPDEC3`). Fail-closed default deny if no rule matches or if evaluation is indeterminate (`PEPDEC1`). |
| `THREAT-PEPDEC-03` | DoS via unbounded obligations | Malicious rule defines thousands of obligations, consuming memory and starving CPU during decision serialization. | **MEDIUM** | Enforce `MAX_PEP_OBLIGATIONS = 32` in `validate_invariants()`. Drop or reject rules with excessive obligations. |
| `THREAT-PEPDEC-04` | Delimiter injection in subject | Attacker crafts `agent:worker:admin` to exploit naive string prefix matching. | **MEDIUM** | Subject identifier format strictly enforced with alphanumeric, colons, underscores, dashes, and max length 256. |
| `THREAT-PEPDEC-05` | Control character injection | Null bytes or ANSI control codes in request parameters bypass string matching or poison audit logs. | **MEDIUM** | `validate_pep_string()` strictly rejects any `\0` or control characters (`c.is_control()`). |

## 2. Hardening Recommendations
1. Integrate `normalize_path_str` on filesystem resource URIs before matching in `match_pattern()`.
2. Ensure `validate_invariants()` is strictly invoked on all decision pathways.
3. Clamp max rules in a single evaluation to 1,000 to prevent evaluation exhaustion.
