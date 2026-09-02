# T-00491 — Documentation Index Control / documentation: Research

## 1. Goal
Establish facts, constraints, user/agent documentation requirements, and prior art for the documentation of Documentation Index Control in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Docs):
1. **Documentation Quality Invariants**: `tools/check_task_docs.py` enforces structural documentation criteria (C1..C6) on `docs/README.md` and related spec files.
2. **Dual-Audience Requirements**:
   - *Human Operators*: Require CLI syntax (`aiosh doc show`, `aiosh doc check`, `aiosh doc search`), JSON formatting flags (`--json`), configuration options (`docs/doc_index_config.json` / `AIOS_DOC_INDEX_CONFIG`), and exit codes (0 = valid, 1 = broken links/errors).
   - *Autonomous Agents*: Require exact MCP tool definitions (`aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`), JSON-RPC schemas, telemetry schemas (`DocIndexTelemetry`), and PEP grant token requirements for mutating actions (`aios.doc.set`).
3. **Reference Contracts & Data Models**:
   - `DocIndexManifest`, `DocIndexEntry`, `DocIndexConfig`, `DocLinkValidationReport`, `BrokenDocLink`, and `DocIndexTelemetry`.

### Assumptions:
1. Clear, copy-pasteable examples for both CLI and MCP surfaces reduce agent error rates during autonomous repo navigation.
2. Explicitly stating known limitations (16 MiB max doc size, 64 KiB config cap, strict repo root link boundary) prevents misconfiguration in sandboxed environments.

## 3. Prior Art & Authoritative Standards
- **Diátaxis Documentation Framework**: Structuring docs into clear How-To guides, Reference definitions, and Architectural explanations.
- **Model Context Protocol (MCP)**: Tool metadata schema, parameter descriptors, and JSON-RPC 2.0 error conventions.
- **CommonMark & GitHub Flavored Markdown (GFM)**: Link reference parsing (`[text](target)`) and heading extraction standards.

## 4. Decisions Needed
1. **Consolidated Documentation Sections**: Maintain comprehensive developer, operator, and agent guidance under the `Documentation Index Control` section in `docs/README.md`.
2. **Test Suite Documentation**: Document the exact verification test runners (`tools/test_doc_index_suites.py`, `tools/test_doc_index_unit.py`, smoke scripts).

## 5. Next Steps
Advance to Specification (T-00492) to define the documentation schema, tables, and example invocations.
