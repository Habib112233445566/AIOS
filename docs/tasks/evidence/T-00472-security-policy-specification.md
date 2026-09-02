# T-00472 — Documentation Index Control / security policy: Specification

## 1. Specification Overview
This specification formalizes the security policy contracts, vulnerability classifications, audit requirements, and CI verification rules for Documentation Index Control.

## 2. Policy Contracts & Threat Classification

### A. Vulnerability Class Definition (`SECURITY.md`)
- **Policy Statement**: Any of the following behaviors constitutes a security vulnerability in AIOS:
  1. Escaping repository boundaries via crafted relative documentation links or path traversal sequences (`../`) during documentation indexing or verification.
  2. Bypassing size caps (64 KiB config cap, 16 MiB document read cap) to cause uncontrolled memory consumption or denial of service.
  3. Evading the classifier → PEP → audit pipeline, including executing documentation index operations without recording structured audit entries.
  4. Tampering with documentation catalog definitions to obfuscate security policies, governance rules, or verification evidence.
- **Classification**: Path Traversal (CWE-22) / Resource Consumption (CWE-400) / Audit Circumvention (ADR-0035).

### B. Audit Ring Invariants
- **CLI Actions**:
  - `aiosh doc show`: Emits structured `doc.show` audit row with outcome and repository context.
  - `aiosh doc check`: Emits structured `doc.check` audit row with link verification report summary.
  - `aiosh doc search`: Emits structured `doc.search` audit row with query keyword and match counts.
- **MCP Tool Calls**:
  - `aios.doc.index.get`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.
  - `aios.doc.check`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.
  - `aios.doc.search`: Wrapped in `dispatch::recorded_call()`, writing structured JSON audit record.

### C. Automated Security Policy Verification (`tools/check_security_policy.py`)
- Criteria S1..S5 in `tools/check_security_policy.py` assert:
  - S1: `SECURITY.md` exists and contains no unresolved TODOs.
  - S2: Private reporting advisory URL is verbatim.
  - S3: Free-form prose floor (>1200 characters).
  - S4: Standard policy wording hits (`vuln`, `disclos`, day counts).
  - S5: All in-tree referenced paths in `SECURITY.md` resolve accurately without broken references.

## 3. Reused vs. New Interfaces
- **Reused Interfaces**:
  - `aiosh-core::audit` / `aiosh-cli::emit` / `aiosh-mcp::dispatch::recorded_call` for audit row append.
  - `tools/check_security_policy.py` for automated policy regression testing.
- **New Policy Clauses**:
  - `SECURITY.md` updates covering documentation index path traversal and resource exhaustion defenses.
  - Linked documentation index security reviews in the Security Knowledge Index.
