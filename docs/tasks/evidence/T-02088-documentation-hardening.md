# Task Evidence: T-02088 (documentation: Hardening)

## Overview
- **Task ID**: T-02088
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-core::capability_doc`, `aiosh-mcp`
- **Objective**: Harden capability documentation subsystem against input variation, casing mismatches, and whitespace/control-character boundary bypasses.

## Hardening Implemented
1. **Case-Insensitive Category Normalization**:
   - In `aiosh-mcp::call_tool("aios.capability.doc")`, added `.trim().to_ascii_lowercase()` normalization to category argument strings, preventing rejection of valid category filters due to casing variations (e.g., `"Architecture"`, `"LIFECYCLE"`).
2. **Pre-Trim Control Character Detection**:
   - Ensured `id.chars().any(|c| c.is_control())` and `query.chars().any(|c| c.is_control())` are checked on raw input before trimming, completely preventing control-character sequences from bypassing filters via whitespace-trimming behavior.
3. **Snippet Alignment Guarantee**:
   - Reinforced UTF-8 char boundary slicing via `char_indices` in `extract_utf8_snippet` to prevent N-21 / H-6 class slicing panics across multi-byte characters.

## Verification
- Unit tests: 8/8 passing in `test_capability_doc.rs`.
- Smoke test: Validated via `test_capability_doc_smoke.py`.
