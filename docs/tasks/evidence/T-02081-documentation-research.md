# Task Evidence: T-02081 (documentation: Research)

## Sub-Epic 9 Launch: Capability Documentation Subsystem
- **Task ID**: T-02081
- **Component**: `aiosh-core::capability_doc`
- **Objective**: Research architecture, topic taxonomy, invariant coverage, and API contracts for the offline Capability Documentation & Reference Subsystem.

## Research Findings & Architecture Design

### 1. Existing Documentation Subsystem Precedents
Analysis of `kernel_module_doc.rs`, `hardware_doc.rs`, and `network_doc.rs` in `aiosh-core` reveals a robust, proven pattern:
- In-memory, offline, zero-network-dependency documentation index (`CapabilityDocIndex`).
- Structured topic hierarchy (`CapabilityDocTopic`, `CapabilityDocSection`, `CapabilityDocCategory`).
- Deterministic search engine with weighted multi-field relevance scoring (title match = 10 pts, tag match = 5 pts, summary match = 3 pts, body match = 1 pt).
- High-fidelity UTF-8 snippet extraction resilient to multi-byte character boundary slicing.
- Strict input sanitization and bounds checking (`MAX_DOC_QUERY_LEN = 256`, `MAX_DOC_SEARCH_RESULTS = 50`, `MAX_TOPIC_ID_LEN = 64`).

### 2. Topic Taxonomy for Capability Model
The documentation index will provide canonical documentation across 8 core topics:
1. `cap-overview`: Capability Model Overview, Architecture, Zero Ambient Authority, and `CAP1..CAP6` Core Invariants.
2. `cap-rights-scopes`: Detailed specifications for `CapabilityRight` (Read, Write, Execute, Delete, Admin, Delegate) and `CapabilityScope` (Filesystem, Network, Tool, Process, Ipc, System).
3. `cap-attenuation`: Monotonic Attenuation, Delegation Trees, Rights Subset Rules, and Scope Confinement.
4. `cap-constraints`: Temporal Lifecycles (`not_before`, `expires_at`), Max Invocations, and Byte Quotas with Saturated Arithmetic.
5. `cap-revocation`: Explicit Revocation, Lineage Graph Traversal, and Cascade Revocation Semantics.
6. `cap-policy`: Mandatory Access Control, Policy Modes (`Enforcing`, `Audit`, `Permissive`), and `CAPSEC1..CAPSEC6` Invariants.
7. `cap-observability`: Point-in-time State Aggregation, Derivation Depth Metrics, and `CAPOBS1..CAPOBS6` Observability Invariants.
8. `cap-mcp-tools`: Complete MCP Tool Surface Reference (`aios.capability.issue`, `attenuate`, `check`, `revoke`, `list`, `observability`, `doc`).

### 3. Categories
- `Architecture`: Foundational concepts, threat model, and zero ambient authority.
- `Lifecycle`: Issuance, attenuation, constraint validation, consumption, and revocation.
- `Security`: Policy enforcement, prohibited paths/hosts, and invariant verification.
- `Observability`: Metrics, health checks, reporting, and telemetry.
- `Reference`: Tool schemas, API parameters, error codes, and practical examples.

### 4. Integration Surface
- Rust Core: `aiosh_core::capability_doc::{CapabilityDocIndex, CapabilityDocTopic, CapabilityDocCategory, CapabilityDocSearchResult}`.
- MCP Server: `aios.capability.doc` tool providing `action: "list" | "get" | "search"` and `query: string`.
