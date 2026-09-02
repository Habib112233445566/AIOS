# T-00471 — Documentation Index Control / security policy: Research

## 1. Goal
Establish facts, constraints, security policies, and prior art for the security policy governing Documentation Index Control within AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Read-Only Invariant**: All Documentation Index Control operations (`aiosh doc show/check/search`, `aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`) are read-only and emit structured audit rows via the centralized dispatch pipeline.
2. **Path Traversal Defenses**: `DocIndexConfig::validate()` strictly rejects `..` in `root_dirs`, and `validate_doc_links` verifies that all markdown relative links resolve strictly within the checkout tree.
3. **Hardening Bounds**: Config loading is capped at 64 KiB (`MAX_CONFIG_BYTES`), and document file ingestion is capped at 16 MiB (`MAX_DOC_READ_BYTES`). Subprocess calls are bounded by 15s/30s timeouts with process reap.
4. **Current Policy Framework**: `SECURITY.md` governs vulnerability definitions, reporting channels, and policy enforcement verified in CI via `tools/check_security_policy.py` (criteria S1..S5).

### Assumptions (To Be Formally Specified in T-00472):
1. Untrusted documentation link structures attempting out-of-bounds filesystem discovery or arbitrary path traversal represent security-critical risks that must be explicitly categorized under `SECURITY.md`.
2. The Security Knowledge Index in `SECURITY.md` should maintain traceability by linking the Documentation Index security review artifacts.

## 3. Prior Art & Authoritative Citations
- **OWASP ASVS v4.0 (Section V5.1 Input Validation & V5.5 File Path Canonicalization)**: Mandates strict path canonicalization and rejection of directory traversal sequences.
- **CWE-22 (Improper Limitation of a Pathname to a Restricted Directory)**: Canonical risks of path traversal in file parsers and indexers.
- **CWE-400 (Uncontrolled Resource Consumption)**: Resource exhaustion protections via bounded parsing caps.
- **AIOS Constitution (ADR-0035 §D-4 / §F-2)**: Classifier → PEP → Audit gate ordering and honest audit emission for all system surfaces.

## 4. Decisions Needed
1. **Scope Definition in `SECURITY.md`**: Should `SECURITY.md` explicitly list documentation catalog path traversal and resource exhaustion under "What Counts as a Vulnerability"?
   - *Decision*: Yes, add explicit wording to `SECURITY.md`.
2. **Security Knowledge Index Update**: Should `SECURITY.md` link the Documentation Index Control security review in its component index?
   - *Decision*: Yes, add the reference to maintain complete audit traceability.
3. **CI Enforcement Verification**: Does `tools/check_security_policy.py` require changes?
   - *Decision*: Ensure `tools/check_security_policy.py` continues to validate all S1..S5 invariants with zero regressions.

## 5. Next Steps
Advance to Specification (T-00472) to formalize the security policy requirements for Documentation Index Control.
