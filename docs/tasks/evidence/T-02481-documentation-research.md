# Task T-02481 Evidence: Sandbox Documentation Research

## Goal
Establish facts, constraints, architectural requirements, and prior art for the documentation subsystem of Sandbox Enforcement.

## Facts vs. Assumptions

| Domain | Facts (Authoritative Sources) | Assumptions |
|---|---|---|
| Offline Availability | AIOS environments may run in air-gapped or network-restricted execution environments. Documentation must be embedded in binary data structures or local documentation indices without external HTTP dependencies. | Plain text / markdown docs can be compiled directly into `aiosh-core` static assets or structured structs with zero runtime overhead. |
| Invariant Coverage | Sandbox Enforcement is governed by strict invariants: SANDBOXPOL1..6 (Policy), SANDBOXOBS1..6 (Observability), AUTOSANDBOX1..8 (Automated Enforcement). | Operators and LLMs need targeted topic search (e.g. querying "landlock", "pep", "profiles", "resource limits") with concise guidance. |
| API & Surface Consistency | CLI surfaces (`aiosh sandbox doc [topic]`) and MCP surfaces (`aios.sandbox.doc`) need parity with other aiosh tools. | A typed query interface returning both Markdown text and structured JSON envelopes satisfies both human operators and LLM agents. |

## Authoritative References
1. **Linux Kernel Documentation**:
   - Landlock LSM: `Documentation/security/landlock.rst` (ABI v1-v3 filesystem restrictions).
   - Seccomp BPF: `Documentation/userspace-api/seccomp_filter.rst` (system call filtering).
   - Namespaces: `namespaces(7)` and `cgroups(7)` (resource quotas and isolation boundaries).
2. **Windows Platform Security**:
   - Job Objects and Restricted Tokens (`SetInformationJobObject`, `CreateRestrictedToken`).
3. **AIOS Security Architecture**:
   - ADR-0035: Policy Enforcement Point (PEP) Fabric, Fail-closed invariants, and audit emission requirements.

## Decisions & Design Plan
1. **Module Structure**: Add `code/aiosh-rust/aiosh-core/src/sandbox_doc.rs` providing `SandboxDocTopic` and `SandboxDocIndex`.
2. **Core Topics**:
   - `overview`: High-level architecture, threat model, and components.
   - `profiles`: Standard, strict, and permissive profile definitions.
   - `policy`: Declarative constraints, prohibited commands/env vars, PEP grants.
   - `observability`: Health probe, metrics, telemetry schemas.
   - `isolation`: OS containment primitives (Landlock, seccomp, Windows job objects).
   - `reference`: CLI and MCP API reference.
3. **Search & Retrieval**: Support exact topic retrieval by ID and fuzzy/lexical query search across titles, keywords, and content.
