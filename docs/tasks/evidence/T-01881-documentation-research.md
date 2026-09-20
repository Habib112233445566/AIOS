# Task Evidence: T-01881 - Network Bootstrap / documentation: Research

## 1. Overview
- **Task ID**: `T-01881`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Research and establish architecture, invariants, and authoritative sources for the Network Bootstrap Documentation subsystem.

---

## 2. Research & Prior Art in AIOS
- Examined existing AIOS documentation index modules: `hardware_doc.rs` and `kernel_module_doc.rs`.
- Key design pattern:
  - Strongly-typed `NetworkDocCategory` (`Architecture`, `Discovery`, `Security`, `Observability`, `Configuration`, `Troubleshooting`).
  - Pre-populated offline topic index (`NetworkDocIndex`) containing comprehensive technical guides, Linux kernel specifications, RFC citations, sysfs paths, configuration guides, and troubleshooting recipes.
  - Scored search engine (`search()`, `get_by_id()`, `list_by_category()`).
  - Dynamic state documenter (`generate_state_markdown()`) that serializes live `NetworkState` into formatted Markdown reports and ASCII topology diagrams.

---

## 3. Fact vs. Assumption Matrix

| Aspect | Fact | Assumption / Decision |
|---|---|---|
| Offline Availability | Agents and operators may operate in air-gapped or bootstrap environments without internet | Documentation repository must be 100% self-contained in binary without external HTTP fetches |
| Search Performance | Fast lookup needed during agent loops and interactive CLI | In-memory tokenized keyword and tag matching provides sub-millisecond query responses |
| Output formats | Both Markdown reports and structured JSON are consumed | Provide both Markdown renderer and JSON serialization |
| Dynamic State Rendering | Network state changes dynamically during boot | Documentation generator must render live state alongside static reference documentation |

---

## 4. Invariants Formulated (`NDOC1..NDOC6`)
1. **`NDOC1` (Canonical Offline Index)**: Pre-populated repository containing canonical topics covering Network Bootstrap architecture, discovery, security policy, observability, and configuration.
2. **`NDOC2` (Deterministic Category Navigation)**: Six standard categories (`Architecture`, `Discovery`, `Security`, `Observability`, `Configuration`, `Troubleshooting`) with robust loose string parsing.
3. **`NDOC3` (Ranked Full-Text Search)**: Query scoring based on ID match, title match, tag matches, and body content matches.
4. **`NDOC4` (Markdown Export)**: Individual topics and complete index sections format cleanly into GitHub Flavored Markdown.
5. **`NDOC5` (Dynamic Network Topology Rendering)**: Generates human-readable Markdown tables and ASCII topology diagrams from live `NetworkState`.
6. **`NDOC6` (Bounded I/O & Path Hygiene)**: Persistent exports enforce path hygiene ($\le 1024$ chars, no `..`), 1 MB file ceiling, and atomic sibling persistence with `TempFileGuard`.
