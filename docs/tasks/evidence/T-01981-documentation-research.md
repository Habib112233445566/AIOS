# Task Evidence: T-01981 - System Update / documentation: Research (Sub-Epic 9)

## 1. Overview
- **Task ID**: `T-01981`
- **Sub-Epic**: 9 (System Update Documentation Subsystem)
- **Goal**: Research and establish architecture, invariants, and authoritative sources for the System Update Documentation Subsystem in AIOS.

---

## 2. Research & Prior Art in AIOS
- Examined existing AIOS documentation index modules: `network_doc.rs`, `hardware_doc.rs`, and `kernel_module_doc.rs`.
- Key architecture patterns:
  - Strongly-typed `SystemUpdateDocCategory` (`Architecture`, `ABPartitioning`, `Security`, `Observability`, `Configuration`, `Troubleshooting`).
  - Pre-populated offline topic index (`SystemUpdateDocIndex`) containing technical guides, Linux kernel A/B partition specifications, NIST SP 800-193, TUF / RFC 8758 citations, and recovery playbooks.
  - Scored search engine (`search()`, `get_by_id()`, `list_by_category()`).
  - Dynamic state documenter (`generate_status_markdown()`, `generate_observability_markdown()`) that serializes live update status and dual-slot partition states into formatted Markdown reports and ASCII diagrams.

---

## 3. Fact vs. Assumption Matrix

| Aspect | Fact | Assumption / Decision |
|---|---|---|
| Offline Availability | Agents and operators may operate in air-gapped or bootstrap environments without internet connectivity | Documentation repository must be 100% self-contained in binary without external HTTP fetches |
| Search Performance | Fast lookup needed during agent loops and interactive CLI sessions | In-memory tokenized keyword and tag matching provides sub-millisecond query responses |
| Output Formats | Both Markdown reports and structured JSON are consumed by MCP agents and operators | Provide both Markdown renderer and JSON serialization |
| Dynamic State Rendering | System update status and slot health change dynamically during update lifecycle | Documentation generator must render live state alongside static reference documentation |

---

## 4. Invariants Formulated (`UDOC1..UDOC6`)
1. **`UDOC1` (Canonical Offline Index)**: Pre-populated repository containing canonical topics covering System Update architecture, A/B partitioning, cryptographic security policy, observability, configuration, and troubleshooting.
2. **`UDOC2` (Deterministic Category Navigation)**: Standard categories (`Architecture`, `ABPartitioning`, `Security`, `Observability`, `Configuration`, `Troubleshooting`) with robust loose string parsing.
3. **`UDOC3` (Ranked Full-Text Search)**: Query scoring based on exact ID match (100 pts), title match (50 pts), tag matches (20 pts each), and body content matches (5 pts each).
4. **`UDOC4` (Markdown Export)**: Individual topics and complete index sections format cleanly into GitHub Flavored Markdown.
5. **`UDOC5` (Dynamic Status & Partition Layout Rendering)**: Generates human-readable Markdown tables and ASCII diagrams from live `SystemSlotStatus`, `SystemUpdateStatus`, and `SystemUpdateObservabilityReport`.
6. **`UDOC6` (Bounded I/O & Path Hygiene)**: Persistent exports enforce path hygiene ($\le 1024$ chars, no `..`), 1 MB file ceiling, and atomic persistence.
