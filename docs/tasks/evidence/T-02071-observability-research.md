# Research: Capability Model Observability (T-02071)

## Executive Summary
This document establishes the architectural foundations, metrics taxonomy, prior art, facts, assumptions, and key design decisions for the **Capability Observability** subsystem (`CAPOBS1..CAPOBS6`) within the AIOS Security Kernel.

---

## 1. Theoretical Foundations & Prior Art

### 1.1 Capability Space Introspection & Microkernel Observability
- **seL4 Kernel Debug & Benchmarking Interfaces**:
  - Microkernel capability spaces (CSpaces) provide deterministic introspection of capability node trees, depth limits, derivation slot counts, and badge tracking.
- **OpenTelemetry & Prometheus Metrics Conventions**:
  - OpenTelemetry Metrics Specification (Semantic Conventions for Resource and System Metrics).
  - Gauges for active token counts, capacity utilization percentages, derivation depth.
  - Monotonic counters for invocations consumed, bytes consumed, revocations executed.
- **Saltzer & Schroeder (1975)** — *Complete Mediation & Auditability*:
  - Every access authorization check and state mutation (issuance, attenuation, revocation, quota increment) must produce measurable telemetry without leaking confidential token identifiers or sensitive credential content.

---

## 2. Existing AIOS Observability Subsystems & Code Patterns

1. **`SystemUpdateObservabilityReport` (`code/aiosh-rust/aiosh-core/src/system_update_observability.rs`)**:
   - Synthesizes state, slot status, progress, policy verdict, and health metrics into a single unified telemetry struct.
   - Implements `sanitize_telemetry_text` to strip control characters and truncate strings to safe bounds.
2. **`SessionObservability` (`session_observability.rs`) / `BaseImageObservability` (`base_image_observability.rs`)**:
   - Provides point-in-time snapshot generation (`generate`), JSON serialization, and health status assessment.
3. **`CapabilityService` (`capability_service.rs`)**:
   - Registry maintains `HashMap<String, Capability>`, `by_subject`, `by_parent`.
   - Methods: `len()`, `is_empty()`, `get_derivation_depth()`, `config()`, `policy()`.

---

## 3. Facts vs. Assumptions

### 3.1 Facts
1. **Fact 1**: The capability registry in `CapabilityService` has dynamic state (active, revoked, expired, depleted) that operators and security audit agents need to monitor in real time.
2. **Fact 2**: High derivation depth ($> 32$) or high registry utilization ($> 80\%$) indicates potential runaway agent delegation or memory pressure requiring proactive warning.
3. **Fact 3**: Sensitive capability fields (such as specific internal capability tokens or cryptographic hashes) should not be leaked in broad telemetry dumps; reports should summarize counts, subjects, scopes, rights, and depths safely.

### 3.2 Assumptions
1. **Assumption 1**: Telemetry reports can be computed on-demand via a fast in-memory traversal over the registry (`capabilities` map).
2. **Assumption 2**: Health status can be derived deterministically: a registry is healthy if capacity utilization is below 90% and no corrupted state has been detected.
3. **Assumption 3**: Observability should be discoverable and exposed as both a Rust API on `CapabilityService` and an MCP tool `aios.capability.observability`.

---

## 4. Required Invariants (`CAPOBS1..CAPOBS6`)

1. **`CAPOBS1` (Comprehensive State Aggregation)**: Report must aggregate total, active, revoked, expired, root, and attenuated capability counts in a single payload.
2. **`CAPOBS2` (Lineage & Derivation Depth Metrics)**: Report must compute maximum derivation depth, average depth, and total derived child counts.
3. **`CAPOBS3` (Quota Consumption Tracking)**: Report must calculate total invocations consumed and total bytes consumed across all registered capabilities.
4. **`CAPOBS4` (Scope & Rights Distribution)**: Report must compute distributions of capabilities by scope type (filesystem, network, tool, etc.) and by right (read, write, execute, etc.).
5. **`CAPOBS5` (Policy & Capacity Health Evaluation)**: Report must include policy enforcement mode, registry capacity utilization percentage, and a boolean `is_healthy` flag.
6. **`CAPOBS6` (Sanitization & Telemetry Safety)**: All text fields in the report must be sanitized using `sanitize_telemetry_text` to prevent log injection and terminal escape corruption.

---

## 5. Decisions Needed Before Implementation

1. **Module Placement**: Create `code/aiosh-rust/aiosh-core/src/capability_observability.rs` and export in `lib.rs`.
2. **Integration with `CapabilityService`**: Add `pub fn generate_observability_report(&self) -> CapabilityObservabilityReport` in `CapabilityService`.
3. **MCP Tool Exposure**: Register `aios.capability.observability` tool in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
