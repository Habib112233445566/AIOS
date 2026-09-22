# Task Evidence: T-02166 (PEP Decision Engine Security Policy: Integration)

## Overview
- **Task ID**: `T-02166`
- **Task Name**: security policy: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 7: Security Policy Subsystem
- **Timestamp**: 2026-09-21T02:44:50+05:00
- **Status**: COMPLETED

## Integration Summary

### 1. Production Call Path Wiring
- **CLI (`aiosh-cli`)**:
  - Wired `PepSecurityPolicy::default().validate_rule_addition(&rule, is_privileged)` directly into `cmd_pep` for the `rule-add` subcommand.
  - Rejects unprivileged additions of `Permit` rules targeting restricted resources (`sys:*`, `sec:*`, `kernel:*`) with exit code 2 (`POLICY_VIOLATION`), while allowing privileged additions (`--privileged`) and `Deny` rules.
- **MCP Server (`aiosh-mcp`)**:
  - Wired `PepSecurityPolicy::default().validate_rule_addition(&rule, false)` into the `aios.pep.rule_add` tool handler.
  - Prevents external MCP tool callers from injecting unauthorized `Permit` rules on restricted prefixes without administrator privilege (resolving finding N-35).

### 2. Verification Commands & Outputs
```bash
python code/aiosh-cli/tests/test_pep_cli_smoke.py
```
Output:
- `PASS: aiosh pep --help`
- `PASS: aiosh pep unknown_cmd returns 2`
- `PASS: aiosh pep path hygiene enforcement`
- `PASS: aiosh pep lifecycle and evaluation`
- `PASS: aiosh pep security policy privilege boundary`
- `=== All PEP CLI tests passed ===`
