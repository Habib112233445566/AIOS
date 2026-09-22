# Task Evidence: T-02171 (PEP Decision Engine Observability: Research)

## Overview
- **Task ID**: `T-02171`
- **Task Name**: observability: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Timestamp**: 2026-09-21T02:48:10+05:00
- **Status**: COMPLETED

## Research: PEP Decision Engine Observability Subsystem

### 1. Authoritative Sources & Prior Art
- **OpenTelemetry Metrics Specification (OTel)**: Standards for Gauge and Counter metric instrumentation in security subsystems.
- **NIST SP 800-92**: *Guide to Computer Security Log Management*. Recommends real-time visibility into access control engine rules, capacity bounds, and anomaly indicators.
- **Prometheus Monitoring Principles**: Capacity saturation indicators, distribution histograms, and health thresholds.
- **Existing AIOS Observability Standards**: `CapabilityObservabilityReport`, `SystemUpdateObservabilityReport`, and `sanitize_telemetry_text` in `aiosh-core`.

### 2. Facts vs. Assumptions

#### Established Facts
1. `PepDecisionService` maintains in-memory rule sets with indexes (`by_subject`, `by_action`) and enforces `MAX_RULES_IN_SERVICE = 5000`.
2. `PepSecurityPolicy` governs enforcement mode (`Enforcing`, `Permissive`, `Disabled`), obligation criticality (`Strict`, `BestEffort`), and restricted prefixes.
3. `cmd_pep status` in `aiosh-cli` and `aios.pep.status` in `aiosh-mcp` currently return basic rule counts, but lack structured telemetry reports, capacity utilization percentages, obligation distribution tallies, and health diagnostics.
4. No centralized `PepObservabilityReport` currently exists for the PEP Decision Engine.

#### Assumptions
1. A dedicated `pep_observability.rs` module in `code/aiosh-rust/aiosh-core/src/` should implement `PepObservabilityReport` with a `generate()` method accepting references to `PepDecisionService` and `PepSecurityPolicy`.
2. The report should compute capacity saturation (`(rules * 100) / 5000`), track rule distribution by effect and obligation, count unique dimensions, and determine a composite `is_healthy` boolean status.
3. Telemetry strings must be sanitized using `sanitize_telemetry_text` to eliminate control characters and ANSI escape sequences.

### 3. Observability Invariants (`PEPOBS1..PEPOBS6`)
- **`PEPOBS1` (State Aggregation)**: Comprehensive aggregation of total rules, unique subjects, unique resources, unique actions, and store metadata.
- **`PEPOBS2` (Capacity Utilization & Saturation Warning)**: Accurate calculation of capacity utilization percentage; flags `is_healthy = false` when utilization $\ge 90\%$.
- **`PEPOBS3` (Distribution Metrics)**: Counts rules by effect (`Permit` vs `Deny`) and catalogs obligation frequencies by variant (`AuditLog`, `RateLimit`, etc.).
- **`PEPOBS4` (Security Policy & Mode Reflection)**: Reports active enforcement mode, obligation criticality, and restricted prefix counts.
- **`PEPOBS5` (Sanitization & Terminal Safety)**: Sanitizes text fields, stripping control characters to prevent ANSI injection in log aggregators.
- **`PEPOBS6` (Deterministic Serialization Parity)**: Lossless JSON serialization compatible with cross-substrate CLI and MCP consumption.
