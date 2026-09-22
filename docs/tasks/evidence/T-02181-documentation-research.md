# Task Evidence: T-02181 - PEP Decision Engine: Documentation: Research

## Task Metadata
- **Task ID**: `T-02181`
- **Sub-Epic**: Sub-Epic 9: Documentation Subsystem
- **Component**: `aiosh-core::pep_doc`
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Objective & Scope
Research the facts, constraints, architectural standards, and prior art necessary to implement an offline, self-contained documentation and help subsystem (`aiosh_core::pep_doc`) for the PEP Decision Engine.

## 2. Authoritative Sources & Citations
1. **RFC 2904 (AAA Authorization Framework)**:
   - Establishes the foundational reference model for Policy Enforcement Points (PEP), Policy Decision Points (PDP), Policy Administration Points (PAP), and Policy Information Points (PIP).
   - In AIOS, the PEP Decision Engine acts as a self-contained embedded PDP+PEP service with in-memory fast indexing and SQLite/JSON backing.
2. **OASIS XACML 3.0 Standard (eXtensible Access Control Markup Language)**:
   - Specifies standard rule combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`).
   - Specifies post-decision obligations as atomic side-effect directives that accompany access decisions.
3. **NIST Special Publication 800-162 (Attribute Based Access Control)**:
   - Prescribes strict target matching on subjects, resources, and actions, with default-deny evaluation semantics.
4. **Existing AIOS Subsystems**:
   - `code/aiosh-rust/aiosh-core/src/capability_doc.rs`: Demonstrates structured in-crate documentation indices (`DocIndex`, `DocTopic`, `DocCategory`, `DocSearchResult`) with BM25-like keyword scoring, UTF-8 snippet extraction, and zero heap runtime overhead.
   - `code/aiosh-rust/aiosh-core/src/hardware_doc.rs` and `network_doc.rs`: Establish consistent query sanitization (`MAX_DOC_QUERY_LEN = 256`), result caps (`MAX_DOC_SEARCH_RESULTS = 50`), and markdown formatting conventions.

## 3. Fact vs. Assumption Separation

### Facts (Established & Verified)
- **Fact 1**: AIOS must be fully operable in air-gapped, offline, and restricted boot environments without internet connectivity. Documentation must be embedded directly in the binary crate.
- **Fact 2**: PEP Decision Engine has 8 completed sub-epics with concrete invariants:
  - Sub-Epic 1: Research (`T-02101..T-02110`)
  - Sub-Epic 2: Specification (`T-02111..T-02120`)
  - Sub-Epic 3: Core Architecture (`T-02121..T-02130`)
  - Sub-Epic 4: Combining & Obligations (`T-02131..T-02140`)
  - Sub-Epic 5: Persistence & Service (`T-02141..T-02150`)
  - Sub-Epic 6: Automated End-to-End Test Suite (`T-02151..T-02160`)
  - Sub-Epic 7: Security Policy Governance (`T-02161..T-02170`)
  - Sub-Epic 8: Observability Subsystem (`T-02171..T-02180`)
- **Fact 3**: Search queries must be bounded to prevent ReDoS or excessive string allocations.
- **Fact 4**: Snippet extraction must respect UTF-8 character boundaries (`char_indices`) to prevent panics when slicing multi-byte characters.

### Assumptions (To be Formalized in Specification)
- **Assumption 1**: 6 primary topic categories (`Architecture`, `Evaluation`, `Policy`, `Observability`, `Security`, `Reference`) provide complete coverage for operators and autonomous agents.
- **Assumption 2**: A minimum of 6 comprehensive topics (`pep-arch`, `pep-algorithms`, `pep-obligations`, `pep-secpolicy`, `pep-observability`, `pep-cli-mcp`) provide necessary depth for operational queries.
- **Assumption 3**: Standard scoring (title match = 10, tag match = 5, body match = 1) yields intuitive ranking without needing external search engines.

## 4. Decisions Needed Before Implementation
1. **Module Name**: `aiosh_core::pep_doc` (consistent with `capability_doc.rs` and `system_update_doc.rs`).
2. **Query Constraints**: Query length capped at 256 bytes; max search results capped at 50; max snippet length 160 chars.
3. **Surface Integration**: Expose via `aiosh pep doc <list|show|search>` on CLI and `aios.pep.doc` tool on MCP.
