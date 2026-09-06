# T-01347: Init & Service Supervision - Configuration: Security Review

## Metadata
- **Task ID:** `T-01347`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Init & Service Supervision Configuration Security Review
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Security Architecture & Threat Surface Overview

The Init & Service Supervision Configuration subsystem (`aiosh-core::service_config`) controls runtime parameters for service state storage, execution timeouts, entity quotas, restart throttling, and auto-persistence. It exposes both operator CLI (`aiosh service config`) and Autonomous Agent MCP (`aios.service.config`) surfaces.

### Core Security Controls:
1. **Bounded File Ingestion (`SC7`)**:
   Configuration files are subject to an upfront metadata length check and a strictly bounded stream reader (`take(65_536 + 1)`), precluding memory exhaustion (OOM) attacks from multi-gigabyte files.
2. **Strict Range & Type Invariants (`SC1..SC5`)**:
   - `store_path`: Enforces non-empty, $\le 1024$ bytes, and rejects ASCII control characters and null bytes (`\0`).
   - `default_timeout_start_secs` / `default_timeout_stop_secs`: Restricted to $[1 \dots 3,600]$ seconds, preventing zero-second instant failures and thread lockups.
   - `max_store_size_bytes`: Restricted to $[65,536 \text{ (64 KiB)} \dots 104,857,600 \text{ (100 MiB)}]$.
   - `max_entity_count`: Restricted to $[10 \dots 100,000]$.
   - `restart_backoff_secs` & `max_restart_burst`: Restricted to $[1 \dots 300]$s and $[1 \dots 50]$ attempts respectively, preventing fork-bombing or process exhaustion.
3. **PEP Capability & Audit Mediation (ADR-0035)**:
   - CLI invocations emit structured audit records via `classify_and_emit` to SQLite `audit.db`.
   - MCP tool calls route through `dispatch::recorded_call`, enforcing Gate #1 (classifier) and Gate #2 (PEP token), writing immutable SHA-256 hash-chained audit rows on all branches.

---

## 2. Abuse Scenarios & Mitigations

### Scenario 1: Path Traversal & Arbitrary File Access via `--config` / `config_path`
- **Attack Vector**: An attacker attempts to read `/etc/shadow` or private SSH keys by passing `--config /etc/shadow`.
- **Analysis & Defense**:
  - The configuration loader does not execute arbitrary code or shell commands.
  - Files are parsed strictly as JSON conforming to the typed `ServiceConfig` schema. Non-JSON files or schemas without required fields fail JSON deserialization, returning `CONFIG_RESOLUTION_FAILED`.
  - Ingestion is size-capped to 64 KiB, preventing buffer bloat.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 2: Service Thrashing & Fork-Bombing via Zero Backoff
- **Attack Vector**: A malicious or misconfigured config sets `restart_backoff_secs: 0` and `max_restart_burst: 1000000` to repeatedly spin up failing processes and exhaust PID space.
- **Analysis & Defense**:
  - Invariant `SC5` rejects backoff $< 1$s or $> 300$s.
  - Invariant `SC5` rejects restart burst $< 1$ or $> 50$.
  - Validation fails immediately upon loading before any supervisor or store state is initialized.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 3: Process Blocking & Thread Starvation via Infinite Timeouts
- **Attack Vector**: Specifying a multi-day or zero-second timeout to freeze execution queues or crash supervision workers.
- **Analysis & Defense**:
  - `SC2` strictly bounds both startup and stop timeouts between 1 second and 3,600 seconds (1 hour).
  - Both zero and excess values are rejected with explicit invariant violation errors.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 4: Resource Exhaustion via Pathological Store Configurations
- **Attack Vector**: Submitting zero or multi-terabyte sizing limits to crash downstream algorithms or allocate excessive memory buffers.
- **Analysis & Defense**:
  - `SC3` enforces maximum store size bounds between 64 KiB and 100 MiB.
  - `SC4` restricts entity counts between 10 and 100,000.
  - Out-of-bound values fail validation immediately before any store initialization.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 5: Audit Trail Circumvention
- **Attack Vector**: An operator or agent crafts an erroneous configuration to execute code without leaving an audit footprint.
- **Analysis & Defense**:
  - Both CLI and MCP paths record audit events on both success and error outcomes.
  - Failed configuration resolution logs the exact error reason, actor ID, and target path to the SQLite WAL audit ring.
- **Verdict / Residual Risk**: Fully Mitigated.

---

## 3. Review Conclusion
- Input validation: Comprehensive and verified (`SC1..SC7`).
- Gating and auditing: Verified across CLI and MCP surfaces.
- Policy bypass: No known policy bypasses or unhandled security flaws remain open.
- Status: APPROVED for Hardening (T-01348).
