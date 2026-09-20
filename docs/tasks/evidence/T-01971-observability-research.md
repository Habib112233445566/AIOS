# Task Evidence: T-01971 (System Update / observability: Research)

## 1. Objective
Establish facts, constraints, prior art, and metric definitions for the AIOS System Update Observability Subsystem (Sub-Epic 8, tasks T-01971 through T-01980).

## 2. Fact vs. Assumption Analysis

### Facts (Codebase & Authoritative Standards)
1. **Existing Observability Architecture in AIOS**:
   - `aiosh-core` implements structured observability across subsystems (`hardware_observability.rs`, `network_observability.rs`, `kernel_module_observability.rs`, `distro_observability.rs`, `base_image_observability.rs`).
   - Standard characteristics:
     - Pure report generator taking inventory/service state + optional security policy.
     - Text sanitization (`sanitize_telemetry_text`: strip control chars, trim, length bound $\le 256$).
     - Deterministic telemetry reporting without mutating underlying system state.
     - Structured metrics: counts, ratios (e.g. progress percentage, staged bytes), compliance verdict, and timestamp.
2. **Current System Update Status State**:
   - `SystemUpdateService` tracks `SystemSlotStatus` and `SystemUpdateStatus`.
   - Missing: A unified observability report synthesizing partition slot health, download/staging progress metrics, disk capacity overhead, and policy evaluation results into a single queryable telemetry object.
3. **Upstream Prior Art**:
   - **ChromeOS `update_engine` D-Bus Status API**: Exposes `last_checked_time`, `progress`, `current_operation`, `new_version`, `new_size`, `active_slot`, and `is_reboot_needed`.
   - **OpenTelemetry (OTel) Specification**: Guidelines for metric counters, gauges (progress percent, free disk space), and structured event attributes.
   - **systemd-sysupdate inspection**: `sysupdate list` / `sysupdate status` outputting JSON telemetry detailing source, target, version, size, and current status.

### Assumptions
1. Free disk space telemetry can be queried from storage path metadata or provided via configuration defaults during user-space testing.
2. Observability reports must serialize to canonical JSON for consumption by MCP tools (`aios.update.status`) and CLI (`aiosh update status --json`).
3. Telemetry generation must be side-effect free: generating a report never advances state machine or locks resources.

## 3. Decisions & Observability Invariants (UOBS1 - UOBS6)
- `UOBS1`: Dual-Slot Status Integrity (reporting active slot, target slot, rollback slot, and per-slot versions/success flags).
- `UOBS2`: Lifecycle State & Progress Bound (state enum and progress clamped strictly in 0..100%).
- `UOBS3`: Staged Payload Metrics (staged artifact count, staged bytes vs declared manifest bytes).
- `UOBS4`: Telemetry Text Sanitization (sanitizing errors, version strings, and release notes to prevent log injection / control char pollution).
- `UOBS5`: Policy Compliance Integration (evaluating and including policy verdict and violation counts when policy is provided).
- `UOBS6`: Cross-Substrate JSON Parity (identical JSON schema across Rust and Python MCP/CLI consumers).

## 4. References & Citations
- Chromium OS update_engine D-Bus API: `https://chromium.googlesource.com/chromiumos/platform/update_engine/+/refs/heads/main/update_engine.proto`
- OpenTelemetry Metrics Specification: `https://opentelemetry.io/docs/specs/otel/metrics/`
- systemd-sysupdate Documentation: `https://www.freedesktop.org/software/systemd/man/latest/systemd-sysupdate.html`
