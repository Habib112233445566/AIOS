# T-00591 — Evidence & Audit Trail / documentation: Research

## 1. Goal
Establish facts, constraints, user/agent documentation requirements, and prior art for the documentation of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Invariants):
1. **Documentation Quality Invariants**: `tools/check_task_docs.py` mechanically enforces structural documentation criteria (`C1`..`C6`) on `docs/README.md` and related spec files.
2. **Dual-Audience Requirements**:
   - *Human Operators*: Require CLI syntax (`aiosh evidence hash`, `aiosh evidence verify`, `aiosh evidence scan`), JSON output flags (`--json`), configuration options (`config/evidence.config.json` / `AIOS_EVIDENCE_CONFIG_PATH`), and exit codes.
   - *Autonomous Agents*: Require exact MCP tool definitions (`aios.evidence.hash`, `aios.evidence.verify`, `aios.evidence.scan`), JSON-RPC schemas, telemetry schemas (`EvidenceTelemetry`), and PEP grant token requirements for mutating actions (`aios.evidence.record`, `aios.evidence.set`).
3. **Reference Contracts & Data Models**:
   - `TaskEvidenceManifest`, `EvidenceRecord`, `EvidenceStep`, `EvidenceConfig`, `EvidenceVerificationReport`, and `EvidenceTelemetry`.

### Assumptions:
1. Clear, copy-pasteable examples for both CLI and MCP surfaces reduce agent error rates and prevent hallucinations during autonomous repository tasks.
2. Explicitly stating known limitations (16 MiB max file cap, 64 KiB config cap, strict repository root containment) prevents misconfiguration in sandboxed environments.

## 3. Prior Art & Authoritative Standards
- **Diátaxis Documentation Framework**: Structuring documentation into Reference (data structures, config parameters), How-To Guides (CLI invocations, MCP tool calls), and Explanation (PEP authorization and hash-chained audit trails).
- **Model Context Protocol (MCP)**: Tool metadata schema, parameter descriptors, and JSON-RPC 2.0 error conventions.
- **NIST SP 800-218 (SSDF) & SLSA v1.0**: Supply-chain attestation and evidence documentation standards.

## 4. Decisions Needed
1. **Consolidated Documentation Sections**: Maintain comprehensive developer, operator, and agent guidance under the `Evidence & Audit Trail` section in `docs/README.md`.
2. **Human-Readable Summary Formatter**: Implement `format_evidence_summary` in `aiosh-core::evidence_service` for clean console and log formatting.
3. **Continuous Invariant Verification**: Ensure all documentation updates pass `tools/check_task_docs.py` (`C1`..`C6`) and `tools/check_security_policy.py` (`S1`..`S5`).

## 5. Next Steps
Advance to Specification (**T-00592**) to define the documentation schema, tables, and example invocations.
