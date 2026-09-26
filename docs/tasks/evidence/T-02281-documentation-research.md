# T-02281 Research: Grant Lifecycle Documentation

**Task:** Establish facts, constraints, and prior art for the documentation of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Documentation  

---

## 1. Executive Summary

This research establishes the requirements, topics, search mechanisms, and architectural prior art for the **Grant Lifecycle Interactive Documentation Subsystem** (`PepGrantDocIndex`). 

The documentation subsystem provides offline, in-kernel, queryable reference documentation for human operators and AI agents interacting with the AIOS Grant Lifecycle subsystem via CLI and MCP.

---

## 2. Established Facts vs Assumptions

### 2.1 Established Facts (Source Code & Prior Art)
1. **Decision Doc Precedent (DOC-FACT-1):** In `code/aiosh-rust/aiosh-core/src/pep_doc.rs`, AIOS implements an offline topic repository with scored lexical search and Markdown rendering.
2. **Grant Capabilities (DOC-FACT-2):** Grant Lifecycle spans 10 sub-epics: data model (`PepGrant`), constraints (`PepGrantConstraints`), states (`PepGrantState`), attenuation, revocation, persistence (`PepGrantStore`), configuration (`PepGrantConfig`), automated testing, security policies (`PepGrantSecurityPolicy`), and observability (`PepGrantObservabilityReport`).
3. **MCP Tool Integration (DOC-FACT-3):** MCP server exposes tools under the `aios.pep.grant.*` namespace (`issue`, `attenuate`, `list`, `inspect`, `validate`, `revoke`, `sweep`, `report`).
4. **Agent Consumption (DOC-FACT-4):** AI agents require structured snippets with examples to synthesize valid tool call parameters without human consultation.

### 2.2 Assumptions Requiring Design Decisions
1. **Dedicated Grant Doc Index (DOC-ASSUME-1):** A dedicated `PepGrantDocIndex` in `code/aiosh-rust/aiosh-core/src/pep_grant_doc.rs` provides domain-specific categorization and search without cluttering generic policy rule documentation.
2. **Topic Granularity (DOC-ASSUME-2):** Topics should cover architecture, state transitions, attenuation rules, revocation cascading, security policy, observability, and tool reference.
3. **Scored Keyword Search (DOC-ASSUME-3):** Lexical token matching across titles, summaries, tags, and sections enables fast retrieval via query strings.

---

## 3. Prior Art & Authoritative Sources

1. **RFC 8693 (OAuth 2.0 Token Exchange):** Conceptual models for delegated authorization, actor tokens, and subject credentials.
2. **Linux Man Pages (man 7 capabilities):** In-kernel authoritative documentation conventions for capability bounding sets and inheritance.
3. **The Design and Verification of a Capability-Based Operating System (Shapiro et al., EROS/KeyKOS):** Documenting capability mechanics, authority trees, and revocation guarantees.

---

## 4. Key Design Decisions for Specification

1. **Module Structure:** Create `code/aiosh-rust/aiosh-core/src/pep_grant_doc.rs` with `PepGrantDocTopic`, `PepGrantDocSection`, `PepGrantDocCategory`, `PepGrantDocSearchResult`, and `PepGrantDocIndex`.
2. **Core Topics:**
   - `grant-arch`: Grant structure, fields, and identifiers.
   - `grant-lifecycle`: State machine and valid transitions.
   - `grant-attenuation`: Monotonic derivation and depth bounds.
   - `grant-revocation`: Targeted and cascade tree revocation.
   - `grant-policy`: Governance via `PepGrantSecurityPolicy`.
   - `grant-observability`: Telemetry and health via `PepGrantObservabilityReport`.
   - `grant-mcp`: Reference for all 8 `aios.pep.grant.*` tools.
3. **Search Engine:** Implement `search(&self, query: &str)` with relevance scoring (tag match > title match > content match) and snippet extraction.

---

## 5. Acceptance Verification
- ✅ Facts and assumptions separated.
- ✅ Authoritative sources cited (RFC 8693, Linux capabilities man pages, EROS capability architecture).
- ✅ Decisions outlined for upcoming specification (T-02282).
- ✅ Zero code modifications in this research phase.
