# Evidence: T-02074 - observability: Implementation

## Task Overview
- **Task ID**: `T-02074`
- **Sub-Epic**: Sub-Epic 8: Observability (`T-02071`..`T-02080`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Implement minimal working behavior for Capability Model Observability (`CAPOBS1..CAPOBS6`).

## Implementation Details
1. **`CapabilityObservabilityReport` (`code/aiosh-rust/aiosh-core/src/capability_observability.rs`)**:
   - `sanitize_telemetry_text()`: strips control characters, trims, limits to 256 characters.
   - `generate()`:
     - Aggregates `total_capabilities`, `active_capabilities`, `revoked_capabilities`, `expired_capabilities`, `root_capabilities`, and `attenuated_capabilities`.
     - Tracks `max_derivation_depth` across all capability chains via `get_derivation_depth()`.
     - Computes cumulative `total_invocations_consumed` and `total_bytes_consumed` with saturating addition.
     - Calculates distributions for `capabilities_by_scope_type` and `capabilities_by_right`.
     - Determines capacity utilization percentage (`capacity_utilization_percent`).
     - Evaluates health status (`is_healthy`) ensuring utilization $< 95\%$ and depth $\le$ policy ceiling.
   - `to_json()`: Serializes report to pretty-printed JSON.
2. **`CapabilityService` (`code/aiosh-rust/aiosh-core/src/capability_service.rs`)**:
   - Added `pub fn capabilities(&self) -> &HashMap<String, Capability>` accessor.

## Verification
- `cargo check -p aiosh-core` passed with zero errors and zero warnings.
