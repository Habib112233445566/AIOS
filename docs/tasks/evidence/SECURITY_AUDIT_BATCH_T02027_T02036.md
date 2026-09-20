# Security Audit Report: Batch T-02027 .. T-02036

**Audit Date**: 2026-09-20  
**Scope**: Tasks `T-02027` through `T-02036`  
**Components Evaluated**:
- `code/aiosh-rust/aiosh-cli` (CLI Surface Security Review, Hardening, Documentation, and Sub-Epic 3 Formal Closure)
- `code/aiosh-rust/aiosh-core` (`CapabilityService::load_or_create`, `get_all_capabilities`)
- `code/aiosh-rust/aiosh-mcp` (MCP/API Surface Research, Specification, Scaffold, Implementation, Unit Tests, and Integration)
- `code/aiosh-mcp/tests/test_capability_mcp_smoke.py` (Cross-surface integration smoke suite)

**Auditor**: Antigravity Autonomous Security Subsystem  
**Overall Verdict**: **PASS — 100% Policy Compliant, Zero Vulnerabilities**

---

## 1. Threat Vector Analysis & Hardening Verification

| Threat ID | Threat Vector | Mitigation Strategy & Verified Implementation | Status |
|---|---|---|---|
| **THREAT-CAPCLI-01** | Path Traversal via `--store-path` | Validated by `validate_service_path`: rejects `..` components, enforces $\le 1024$ length, requires `.json` extension, and blocks symlinks. | **MITIGATED** |
| **THREAT-CAPCLI-02** | Unbounded Argument Injection | Enforced length bounds ($\le 128$ for IDs, $\le 256$ for subjects/issuers) and rejected ASCII control characters (`\0`, `\n`, `\r`, `\t`, ANSI escapes). | **MITIGATED** |
| **THREAT-CAPCLI-03** | Terminal Escape Sequence Injection (CWE-150) | Sanitized all user-controlled strings emitted to terminal output via `sanitize_terminal`. | **MITIGATED** |
| **THREAT-CAPCLI-04** | Quota / Numeric Overflow Evasion | Enforced strict `u64` parsing on `--max-invocations` and `--quota-bytes`; invalid/negative/overflow inputs trigger immediate exit code 2. | **MITIGATED** |
| **THREAT-CAPMCP-01** | Unauthorized Root Issuance via MCP | Evaluated `issuer` identity in `aios.capability.issue`: non-kernel/non-admin callers are strictly refused. | **MITIGATED** |
| **THREAT-CAPMCP-02** | Privilege Escalation in Attenuation | Attenuation validates that child capabilities cannot claim rights or scope broader than their parent. Attempted escalation fails with explicit error. | **MITIGATED** |
| **THREAT-CAPMCP-03** | Revocation Bypass / Subtree Orphanage | Revocation recursively traverses the entire delegation DAG via BFS and marks all descendants as revoked. Post-revocation access checks return `granted: false`. | **MITIGATED** |
| **THREAT-CAPMCP-04** | Unbounded Resource Exhaustion (DoS) | Backing store size bounded by `MAX_CAPABILITY_STORE_SIZE` (10 MB); in-memory registry capacity bounded by `MAX_CAPABILITIES_IN_REGISTRY` (10,000 entries). | **MITIGATED** |
| **THREAT-CAPMCP-05** | Audit Evasion / Silent Failure | All 7 MCP capability tools route through `dispatch::recorded_call`, writing exactly one SHA-256 hash-chained row to the Audit Ring per invocation (ADR-0035 §A F-2). | **MITIGATED** |

---

## 2. Automated Test Verification Results

1. **Rust Unit Tests (`aiosh-cli`)**:
   - `test_capability_cli_basic`: PASSED
   - `test_capability_cli_json_envelope`: PASSED
   - `test_capability_cli_audit_logging`: PASSED
   - `test_capability_cli_hardening_validation`: PASSED
2. **Rust Unit Tests (`aiosh-mcp`)**:
   - `test_capability_mcp_tools`: PASSED (0.21s)
     - Manifest discovery
     - Empty listing
     - Unauthorized issuance refusal
     - Valid root issuance
     - ID retrieval
     - Privilege escalation rejection
     - Monotonic attenuation
     - Invocation quota consumption
     - Access check denial for ungranted rights
     - Transitive cascade revocation
     - Denial of revoked access
     - Expired leaf pruning
3. **Python Smoke Suites**:
   - `code/aiosh-cli/tests/test_capability_cli_smoke.py`: 4/4 PASSED
   - `code/aiosh-mcp/tests/test_capability_mcp_smoke.py`: ALL TESTS PASSED against compiled `aiosh-mcp.exe`
4. **Task Ledger Integrity**:
   - Task progression strictly verified from 2027 to 2036.
   - `next_task: 2037`.

---

## 3. Compliance & Architectural Invariants

- **ADR-0035 §D-2**: MCP is confirmed as the sole external tool invocation protocol; tool schemas strictly match JSON Schema Draft-07.
- **ADR-0035 §A F-2**: Every consequential action writes an honest audit event with SHA-256 hash chaining before committing.
- **CAP1..CAP6 Invariants**: Unforgeability, monotonic reduction, cascade revocation, temporal/quota constraints, and atomic persistence are fully preserved across both CLI and MCP surfaces.
