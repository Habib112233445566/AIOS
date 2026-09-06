# T-01357: Init & Service Supervision - Automated Tests: Security Review

## Metadata
- **Task ID:** `T-01357`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Automated Tests Security Review
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Security Architecture & Threat Modeling

This security review evaluates the automated integration test infrastructure (`code/aiosh-rust/aiosh-core/tests/test_service_automated.rs` and `tools/test_service_suites.py`) and its interactions with the core service store, supervision engine, topological dependency graph planner, filesystem, and audit rings.

---

## 2. Abuse Scenarios & Mitigations

### Abuse Scenario 1: Path Traversal & System File Overwrite via Store Paths
- **Threat**: An attacker or rogue test harness supplies traversal sequences (`../../etc/shadow`) or system targets (`/etc/systemd/system/malicious.service`) to overwrite host files.
- **Analysis**: The core configuration validation (`ServiceConfig::validate()`) enforces `SC1`: length $\le 1024$ bytes, no null bytes (`\0`), and no ASCII control characters. Furthermore, test suites create isolated per-process temporary paths cleaned up upon test completion.
- **Verdict**: PASS. No unauthorized file overwrites possible.

### Abuse Scenario 2: Cyclic Dependency Starvation (Infinite Loop / Stack Overflow)
- **Threat**: An attacker registers circular dependencies (`svc-a` requires `svc-b`, `svc-b` requires `svc-a`) to trigger infinite loops, thread lockup, or stack exhaustion during boot sequencing.
- **Analysis**: Invariant `CS3` implements Kahn's algorithm with explicit in-degree tracking and cycle detection (`ordered.len() != closure.len()`). If a cycle exists, planning aborts immediately with an explicit error identifying the circular graph. Verified in `test_st2_dependency_dag_order_and_cycle_detection`.
- **Verdict**: PASS. Cycle detected and rejected in $O(V + E)$ time; zero infinite recursion.

### Abuse Scenario 3: Process Exhaustion via Zero-Backoff Crash Loops
- **Threat**: Misconfigured or malicious services crashing repeatedly to induce fork-bombing or system PID starvation.
- **Analysis**: Invariant `SC5` bounds restart backoff to $[1 \dots 300]$ seconds and restart bursts to $[1 \dots 50]$ attempts. Zero or sub-minimum backoff values fail validation before store activation.
- **Verdict**: PASS. Supervision backoff strictly enforced.

### Abuse Scenario 4: Security Masking Bypass
- **Threat**: An attacker or unauthorized agent attempts to start or restart a disabled or masked service (e.g., re-activating disabled telnet or insecure daemons).
- **Analysis**: Invariants `CS2` and `SS1` strictly prohibit state transitions for units where `startup_mode == ServiceStartupMode::Masked`. Both `Start` and `Restart` actions check the mask flag upfront and return explicit error envelopes (`cannot start/restart masked service`). Verified in `test_st1_lifecycle_fsm_cohesion_and_masking`.
- **Verdict**: PASS. Masking cannot be bypassed.

### Abuse Scenario 5: PEP Capability & Audit Logging Parity (ADR-0035)
- **Threat**: Service actions executing without emitting cryptographic audit records to SQLite WAL.
- **Analysis**: Production surfaces (CLI `aiosh service action` and MCP `aios.service.action`) execute through `dispatch::recorded_call` and `classify_and_emit`, appending immutable SHA-256 hash-chained audit rows on all branches.
- **Verdict**: PASS. Full audit traceability maintained.

---

## 3. Findings & Resolution
- Zero policy bypasses identified.
- All boundary limits, cycle rejections, and masking assertions verified in `test_service_automated.rs`.
- Status: APPROVED for Hardening (T-01358).
