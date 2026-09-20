# Task Evidence: T-02078 (observability: Hardening)

## Overview
- **Task ID**: T-02078
- **Sub-Epic**: Sub-Epic 8: Capability Observability & Telemetry Subsystem
- **Component**: `aiosh-core::capability_observability`
- **Objective**: Harden capability observability against algorithmic complexity attacks ($O(N \times D)$ traversal), cycle recursion, and malformed telemetry timestamps.

## Hardening Implemented
1. **Memoized Derivation Depth (`get_memoized_depth`)**:
   - Implemented depth memoization via `HashMap<String, usize>` during `CapabilityObservabilityReport::generate()`.
   - Included cycle detection via a recursion visited set (`visiting: HashSet<String>`) and max recursion limit of 256.
   - Reduced report generation traversal complexity from $O(N \times D)$ to $O(N)$ across all capability nodes.
2. **Timestamp Sanitization & Fallback**:
   - Hardened `sanitize_telemetry_text` handling so that inputs composed entirely of control characters or whitespace safely fall back to `Utc::now().to_rfc3339()`, preventing malformed or empty timestamp strings from propagating into telemetry streams.
   - Preserved 256-character length bound and control character stripping.

## Verification
- Added test `test_observability_hardening_edge_cases` in `tests/test_capability_observability.rs`.
- Validated with `cargo test --test test_capability_observability`.
