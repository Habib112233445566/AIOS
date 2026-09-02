# T-00572 — Evidence & Audit Trail / security policy: Specification

## 1. Specification Overview
This specification formalizes the security policy contracts, vulnerability classifications, audit ring emission rules, and CI verification requirements for Evidence & Audit Trail.

## 2. Policy Contracts & Threat Classification

### A. Vulnerability Class Definition (`SECURITY.md`)
- **Policy Statement**: The following behaviors represent security vulnerabilities in AIOS and must be reported via private security advisories:
  1. Falsifying, bypassing, or forging SHA-256 evidence digests to simulate completion of unverified tasks.
  2. Tampering with or altering historical evidence markdown files in `docs/tasks/evidence/` or mutating event logs in `docs/tasks/COMPLETIONS.jsonl`.
  3. Path traversal escapes (`../`) outside the repository root during evidence discovery, hashing, or scanning.
  4. Causing denial of service or uncontrolled resource exhaustion by bypassing the 16 MiB evidence file cap or 64 KiB configuration cap.
  5. Evading the classifier → PEP → audit gate pipeline or executing state mutations without producing deterministic audit logs.
- **Classification**: Insufficient Verification of Data Authenticity (CWE-345) / Path Traversal (CWE-22) / Uncontrolled Resource Consumption (CWE-400) / Audit Circumvention (ADR-0035).

### B. Audit Ring Invariants
- **CLI Actions**:
  - `aiosh evidence hash <path>`: Emits structured `evidence.hash` audit row with targeted path and calculated digest.
  - `aiosh evidence verify`: Emits structured `evidence.verify` audit row with verification report summary (`is_valid`, `valid_records`, `missing_files`, `hash_mismatches`).
  - `aiosh evidence scan`: Emits structured `evidence.scan` audit row with discovered file counts and task filters.
- **MCP Tool Calls**:
  - `aios.evidence.hash`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.
  - `aios.evidence.verify`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.
  - `aios.evidence.scan`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.

### C. Automated Security Policy Verification (`tools/check_security_policy.py`)
- Criteria S1..S5 in `tools/check_security_policy.py` assert:
  - **S1**: `SECURITY.md` exists at the root and contains no unresolved TODO markers.
  - **S2**: Private advisory reporting URL is present verbatim.
  - **S3**: Free-form prose floor (>1200 characters).
  - **S4**: Standard policy terminology hits (`vuln`, `disclos`, day counts).
  - **S5**: All in-tree referenced paths in `SECURITY.md` resolve accurately without broken references.

## 3. Reused vs. New Interfaces
- **Reused Interfaces**:
  - `aiosh-core::audit` / `aiosh-cli::emit` / `aiosh-mcp::dispatch::recorded_call` for audit row persistence.
  - `tools/check_security_policy.py` for automated policy verification in CI.
- **New Policy Clauses**:
  - `SECURITY.md` updates covering evidence integrity, checksum protection, and evidence catalog path traversal defenses.
  - Linked Evidence & Audit Trail security reviews in `SECURITY.md` §Security Knowledge Index.
