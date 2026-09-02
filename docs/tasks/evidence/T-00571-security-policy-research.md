# T-00571 — Evidence & Audit Trail / security policy: Research

## 1. Goal
Establish facts, constraints, security policies, and prior art for the security policy governing the Evidence & Audit Trail within AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Repository):
1. **Evidence Structure & Checksums**: Evidence artifacts are stored in `docs/tasks/evidence/` with strict naming `T-NNNNN-*.md` and verified deterministically using SHA-256 digests (`tools/check_evidence.py` E1..E4).
2. **Access Control & Invariants**: Read-only operations (`hash`, `scan`, `verify`) execute unauthenticated, while state-changing operations (`complete_task.py`) append immutable, hash-chained events to `docs/tasks/COMPLETIONS.jsonl`.
3. **Existing Security Policy**: Root `SECURITY.md` defines supported surfaces, private reporting via GitHub Security Advisories, 7-day acknowledgment / 90-day coordinated disclosure windows, and rule-pack version stamping.
4. **CI Policy Enforcement**: `tools/check_security_policy.py` validates `SECURITY.md` against OpenSSF Scorecard criteria (S1..S5) in continuous integration.

### Assumptions (To Be Formally Specified in T-00572):
1. **Vulnerability Classification**: Tampering with evidence artifacts, forging SHA-256 checksums, bypassing task execution order, or attempting directory traversal during evidence indexing constitutes a critical security vulnerability and must be explicitly recorded under `SECURITY.md`.
2. **Audit Traceability**: The Security Knowledge Index in `SECURITY.md` should maintain end-to-end traceability by referencing the Evidence & Audit Trail security review artifacts (`T-00567-security.md`, `T-00577-security.md`).

## 3. Prior Art & Authoritative Citations
- **NIST SP 800-218 (Secure Software Development Framework - SSDF v1.1)**:
  - *Task PS.2 (Verify Software Release Integrity)*: Mandates cryptographic verification of release components and build artifacts to prevent tampering.
  - *Task PS.3 (Archive and Protect Release Components)*: Requires archiving necessary integrity verification information and provenance data for every deliverable.
- **SLSA v1.0 (Supply-chain Levels for Software Artifacts)**:
  - Defines provenance requirements, non-falsifiable artifact generation, and automated verification of build integrity.
- **OpenSSF Scorecard (Security-Policy Criteria)**:
  - Standardizes vulnerability disclosure policies, communication channels, and timeline commitments.
- **AIOS Constitution (ADR-0035 / ADR-0036)**:
  - Enforces the immutable hash-chained audit ring, classifier $\to$ PEP $\to$ audit gate sequence, and the binding sequential task execution law.

## 4. Decisions Needed
1. **Explicit Vulnerability Definition**: Update `SECURITY.md` to include evidence artifact tampering, checksum forgery, and evidence catalog path traversal under "What Counts as a Vulnerability".
2. **Security Knowledge Index Update**: Add references to Evidence & Audit Trail security reviews in `SECURITY.md` §Security Knowledge Index.
3. **Automated Policy Enforcement**: Verify that `tools/check_security_policy.py` continues to pass cleanly (S1..S5) with zero policy rot.

## 5. Next Steps
Advance to Specification (**T-00572**) to formalize the security policy requirements for Evidence & Audit Trail.
