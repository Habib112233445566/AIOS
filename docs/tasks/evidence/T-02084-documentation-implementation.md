# Task Evidence: T-02084 (documentation: Implementation)

## Overview
- **Task ID**: T-02084
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-core::capability_doc`
- **Objective**: Implement the full Capability Documentation Subsystem with 8 canonical topics, weighted multi-field search, UTF-8 safe snippet extraction, and defensive bounds.

## Implementation Details
1. **Canonical Topics Registered (`CAPDOC1`)**:
   - `cap-overview`: Architecture, zero ambient authority, and `CAP1..CAP6`.
   - `cap-rights-scopes`: `CapabilityRight` and `CapabilityScope` specifications.
   - `cap-attenuation`: Monotonic attenuation, delegation requirements, and subset checking (`CAP3`).
   - `cap-constraints`: Temporal bounds and quota enforcement with saturated arithmetic (`CAP4`).
   - `cap-revocation`: Immediate revocation and cascade revocation (`CAP5`).
   - `cap-policy`: MAC rules, policy modes, and `CAPSEC1..CAPSEC6`.
   - `cap-observability`: Observability invariants `CAPOBS1..CAPOBS6`, memoized depth, and sanitization.
   - `cap-mcp-tools`: Complete MCP tool surface reference (`aios.capability.*`).
2. **Search Engine (`CAPDOC3`, `CAPDOC5`)**:
   - Weighted multi-field ranking: ID match (+100/+40), Tag match (+50/+20), Title match (+25), Summary match (+15), Section title/content match (+10).
   - Defensive validation: Query length capped at 256 chars, control characters rejected, max 50 results.
3. **UTF-8 Safe Snippet Extraction (`CAPDOC4`)**:
   - `extract_utf8_snippet`: Uses `char_indices` to extract contextual snippets without byte-slicing panics (guarding against N-21 class multi-byte slicing bugs).
4. **Deterministic Lookup & Markdown Formatting (`CAPDOC2`, `CAPDOC6`)**:
   - `get_topic`: Case-insensitive ID lookup with length and control character validation.
   - `format_topic_markdown`: Produces clean GitHub-flavored markdown.

## Verification
- Verified compilation via `cargo check -p aiosh-core`.
