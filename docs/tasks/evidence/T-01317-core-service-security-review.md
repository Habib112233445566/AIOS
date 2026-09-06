# T-01317: Init & Service Supervision - Core Service: Security Review

## Metadata
- **Task ID:** `T-01317`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service_service`
- **Component:** Init & Service Supervision Core Service Security Review
- **Status:** Complete

## 1. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: State Machine Bypass & Masked Service Activation
- **Attack Vector:** An unauthorized actor or rogue script attempts to force-start a disabled or administrative-masked unit (`startup_mode == Masked`) to resurrect deprecated, unsafe, or vulnerability-prone daemons.
- **Mitigation:**
  - `ServiceStore::execute_action` strictly inspects `status.startup_mode`.
  - When `action` is `Start`, `Restart`, or `Reload`, if `startup_mode == ServiceStartupMode::Masked`, the transition is immediately blocked with an error (`cannot start masked service`).
  - Transitioning to `Masked` state automatically forces the service to `Inactive` if running.
- **Verdict:** Secure. Enforces strict administrative masking invariants.

### Abuse Scenario 2: Dependency Graph Cycle Attack / Stack Overflow DoS
- **Attack Vector:** A caller constructs or manipulates service dependencies into a cyclic graph (e.g., `svc-a -> svc-b -> svc-c -> svc-a`) attempting to trigger unbounded recursive traversal, call-stack exhaustion, or infinite execution loops in the service supervisor.
- **Mitigation:**
  - `ServiceStore::plan_service_order` uses an iterative Kahn's algorithm implementation with in-degree tracking instead of unbounded recursion.
  - Cycle detection logic verifies that the number of topologically sorted services matches the reachable dependency set. If an unresolved cycle remains, Kahn's algorithm halts immediately and returns an explicit descriptive error (`dependency cycle detected involving: ...`).
- **Verdict:** Secure. Immune to stack exhaustion; execution complexity is bounded to $O(V + E)$ where $V \le 10,000$ and $E \le 128 \times V$.

### Abuse Scenario 3: Store File Corruption & Atomic Write Hijacking
- **Attack Vector:** Interrupting store writes (crash, power loss, kill signal) during service registration or status updates to leave truncated, corrupted JSON on disk, causing denial of service on system reboot.
- **Mitigation:**
  - `ServiceStore::save_to_path` employs atomic filesystem semantics: serialized JSON is written to a temporary sibling file (`<path>.tmp.<timestamp>`), flushed to storage, and atomically renamed onto the target file.
  - Directory permissions and sandbox isolation prevent symlink redirection or path tampering.
- **Verdict:** Secure. Eliminates partial write window and protects store integrity.

### Abuse Scenario 4: Resource Exhaustion via Malicious Store Inflation (Zip/Decompression Bomb)
- **Attack Vector:** Supplying an excessively large or malformed service database file to exhaust host memory during deserialization.
- **Mitigation:**
  - `ServiceStore::load_from_path` enforces a strict 10 MiB (`10 * 1024 * 1024` bytes) hard ceiling on filesystem inputs prior to reading into memory.
  - Files exceeding the limit are rejected immediately with an I/O payload size error without allocating heap buffers.
- **Verdict:** Secure. Guarantees memory allocation limits on untrusted data.

### Abuse Scenario 5: Unauthorized Action Dispatch & Audit Bypass
- **Attack Vector:** Invoking state-changing lifecycle actions (`start`, `stop`, `restart`, `enable`, `mask`) without authorization or without generating immutable non-repudiation records.
- **Mitigation:**
  - CLI `aiosh service action` writes structured audit events to `audit.log` capturing actor, target service, requested action, and outcome.
  - MCP `aios.service.action` operates exclusively through `dispatch::recorded_call`, enforcing Policy Enforcement Point (PEP) token checks, caller capability grants, and SHA-256 hash-chained WAL audit ring entries for both successful and rejected transitions.
- **Verdict:** Secure. Zero bypass paths exist.

## 2. Policy Gating & Audit Trail Conformance
- **PEP Enforcement:** All MCP tool endpoints (`aios.service.list`, `aios.service.get`, `aios.service.action`, `aios.service.order`, `aios.service.validate`) route through `dispatch::recorded_call`.
- **Immutable Audit Trail:** State-changing and query operations emit SHA-256 hash-chained audit events recording actor identity, grant ID, tool arguments, and results.
- **Policy Bypass Assessment:** Complete code audit confirms no unauthenticated, unaudited, or unvalidated state transitions exist in the codebase.
