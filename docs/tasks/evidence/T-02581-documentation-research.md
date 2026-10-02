# T-02581: Privilege Escalation Prevention Documentation Research

- **Task**: `T-02581`
- **Sub-Epic**: Privilege Escalation Prevention / documentation
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Objective & Scope
Research offline documentation architecture, topic categorization, relevance scoring, and rendering for the Privilege Escalation Prevention subsystem in AIOS.

## 2. Prior Art & Authoritative Sources
1. **AIOS Reference Documentation Subsystems**:
   - `PepGrantDocIndex` in `code/aiosh-rust/aiosh-core/src/pep_grant_doc.rs`: Categorized topics, section-based schema, ranked token search, markdown rendering.
   - `SandboxDocIndex` in `code/aiosh-rust/aiosh-core/src/sandbox_doc.rs`: Self-contained offline documentation repository for isolation tiers and sandbox controls.
2. **NIST SP 800-53 Rev. 5 (AC-6 Least Privilege)**:
   - AC-6(1): Authorize access to security functions.
   - AC-6(2): Non-privileged access for non-security functions.
   - AC-6(3): Network access to privileged commands.
3. **Linux / POSIX Capability & Privilege Escalation Models**:
   - `capabilities(7)` and `setuid(2)` documentation patterns: Explaining capabilities, bounding sets, inheritance rules, and drop mechanisms.

## 3. Facts vs Assumptions
- **Fact**: Operators and agents need offline, fast, zero-network access to privilege policies, tier hierarchies, capability lists, and troubleshooting steps.
- **Fact**: Token-based keyword scoring with weights (Tags +10, Title +5, Summary +3, Sections +1..2) provides deterministic, zero-dependency search without external search engines.
- **Assumption**: A minimum of 6 canonical topics is required: Architecture, Lifecycle & Transitions, SystemKernel Invariant, Security Policy, Observability & Telemetry, and Troubleshooting / Recovery.

## 4. Key Decisions & Invariants
1. `PRIVDOC1`: Pre-populated repository of structured topics (`PrivilegeDocIndex`).
2. `PRIVDOC2`: Multi-field relevance scoring with query clamping ($\le 128$ chars) and result limit ($\le 10$).
3. `PRIVDOC3`: Markdown export formatting with YAML metadata and code examples.
4. `PRIVDOC4`: Integrated into CLI (`aiosh privilege doc`) and MCP (`aios.privilege.doc`).
