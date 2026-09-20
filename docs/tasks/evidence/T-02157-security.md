# Security Review & Abuse Scenarios: T-02157

## Scope
Security evaluation of the Automated Test Subsystem for the PEP Decision Engine (`test_pep_decision_e2e.rs`, `test_pep_cli_smoke.py`, `test_pep_config_smoke.py`, `test_pep_decision_smoke.py`).

## Abuse Scenarios & Mitigations

### 1. Abuse Scenario 1: Path Traversal & Symlink Manipulation
- **Vector**: Adversary supplies test vectors with `..` or symlink redirection to read or corrupt sensitive host files.
- **Result**: Blocked by `validate_pep_service_path` and `PepConfig::validate()`. Symlinks rejected prior to reading.
- **Outcome**: `PEPSERV_ERR_VALIDATION` / `PEPCONF_ERR_HYGIENE` returned, zero filesystem escape.

### 2. Abuse Scenario 2: Memory Exhaustion via Rule Flooding
- **Vector**: Adversary passes a payload attempting to register $> 5000$ rules.
- **Result**: Rejected with `PEPSERV_ERR_CAPACITY`. Service memory bounded.

### 3. Abuse Scenario 3: Malformed JSON Fault Injection
- **Vector**: Truncated, unclosed, or corrupted JSON fed to policy loader.
- **Result**: Handled non-destructively by `load_or_recover()`. Damaged file renamed to `.bak.<timestamp>`, empty fail-closed service returned.

### 4. Abuse Scenario 4: Non-Deterministic First-Applicable Ordering
- **Vector**: Rules added in unpredictable order or hashed unpredictably across processes.
- **Result**: `candidate_rules()` sorts candidates deterministically by ID before evaluation.

## Verdict
Zero policy bypasses found. Security review passed.
