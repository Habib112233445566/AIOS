# T-01391: Init & Service Supervision Recovery & Validation Research

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01391  

---

## 1. Executive Summary & Objective
Task `T-01391` establishes facts, architectural constraints, authoritative prior art, and recovery strategies for the **Init & Service Supervision** subsystem. In an operating system, the init system and service supervisor constitute the primary execution spine. If on-disk service configurations, state stores, or runtime tracking databases become damaged, corrupted, or truncated (e.g. resulting from sudden power interruption, filesystem unmounts, out-of-space writes, or concurrent process crashes), the supervisor must not enter an infinite crash loop or hang during system boot.

This research establishes the design for a dedicated recovery and validation module (`service_recovery.rs`), defining:
- Deep validation reports evaluating specification syntax, status invariants, dependency graph integrity, and capacity limits.
- Non-destructive forensic preservation, ensuring corrupted files are preserved as timestamped backups rather than overwritten.
- Automated self-healing fallbacks that reconstruct a canonical default service store (`aios-securityd.service`, `auditd.service`, `dbus.service`, etc.).
- Operator CLI auditing (`aiosh service check [--fix]`) and Agent MCP inspection (`aios.service.check`).

---

## 2. Codebase Audit: Existing Recovery Subsystems & Patterns

An audit of the AIOS Rust codebase reveals mature, established recovery subsystems in adjacent components:

| Subsystem | Module Path | Validation Report | Recovery Invariants | Non-Destructive Backup |
|---|---|---|---|---|
| **Linux Distro Selection** | `aiosh-core/src/distro_recovery.rs` | `DistroHealthReport` | `V1..V2` | `<path>.corrupt.<ts>.bak` |
| **Linux Base Image Build** | `aiosh-core/src/base_image_recovery.rs` | `BaseImageValidationReport` | `RV1..RV3` | `<path>.corrupt.<ts>.bak` |
| **Package Management** | `aiosh-core/src/package_recovery.rs` | `PackageValidationReport` | `RV1..RV4` | `<path>.bak.<ts>` |

### Key Architectural Commonalities Across AIOS Recovery Modules
1. **Mathematical Invariant Conservation:**
   $$\text{valid\_items} + \text{invalid\_items} = \text{total\_items}$$
   $$\text{healthy} \iff (\text{errors.is\_empty}() \land \text{invalid\_items} == 0)$$
   $$\text{invalid\_items} > 0 \implies \text{errors.len}() \ge \text{invalid\_items}$$
2. **Forensic Quarantine:** Damaged on-disk files are renamed using microsecond or millisecond Unix epoch timestamps (`.corrupt.<ts>.bak`) prior to writing fresh canonical stores, ensuring that forensic state is preserved for post-incident root cause analysis (RCA).
3. **Canonical Reference Seeding:** A recovered store is never empty; it is populated with guaranteed-valid reference entities so that the system remains functional.
4. **Dual-Mode Operation (`check` vs `repair`):**
   - Read-only audit mode checks store health and reports issues without modifying disk state.
   - Fix/Recovery mode triggers non-destructive quarantine and state reconstitution.

---

## 3. Authoritative Prior Art & Standards

### 1. systemd Service & Unit Verification (`systemd-analyze verify`, `systemctl`)
- **Static Verification:** `systemd-analyze verify` performs deep structural validation of unit files, checking for syntax errors, missing dependencies, cycles, and invalid execution paths without launching processes.
- **Fail-Safe Fallbacks:** When default targets or critical service definitions are unparseable, systemd drops to `emergency.target` or `rescue.target` to preserve operator access.
- **State Serialization (`daemon-reexec`):** During daemon reloads or supervisor crashes, file descriptors and PID states are serialized to `/run/systemd/` with strict fallback handling if state files are corrupted.

### 2. OpenRC Service Health & State Recovery
- **Crash Detection:** `rc-status -c` scans running daemons to detect processes that have died unexpectedly while recorded as active, clearing stale state locks in `/run/openrc`.
- **Fault Isolation:** Malformed runlevel scripts in `/etc/init.d/` do not prevent other runlevel services from starting; errors are logged to console and boot continues.

### 3. s6 / runit Supervision Trees
- **Directory Isolation:** Each service exists in an isolated directory. A syntax or permission failure in one service definition does not corrupt the supervisor daemon or halt adjacent supervise trees.
- **Supervision State Reconstitution:** If supervisory FIFOs or status files in `/etc/service/<name>/supervise` are damaged, `s6-svscan` recreates the supervisory context dynamically on startup.

### 4. NIST SP 800-53 Rev 5 Standards
- **SI-10 (Information Input Validation):** System configurations, service manifests, and command parameters must be validated prior to execution.
- **CP-10 (Information System Recovery & Reconstitution):** The system must support reconstitution to a secure, verified baseline state.
- **AU-9 (Protection of Audit & Forensic Information):** Corrupted state must be quarantined and preserved for incident investigation.

---

## 4. Facts vs Assumptions

### Established Facts
1. `ServiceStore::load_from_path` in `service_service.rs` currently returns `Err(String)` on malformed JSON or validation errors, leaving the caller with no recovery mechanism.
2. `ServiceStore::new()` already initializes a canonical set of five production-grade system services (`auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `aios-securityd.service`, `ssh.service`).
3. There is currently no `service_recovery.rs` in `aiosh-core`.
4. Neither `aiosh service` (CLI) nor `aios.service.*` (MCP) exposes a `check` or `repair` command to diagnose and repair service store integrity.
5. All adjacent AIOS subsystems follow the `load_or_recover` pattern with structured validation reports.

### Verified Assumptions
1. **Quarantine Safety:** Renaming a corrupted service store to `<path>.corrupt.<ts>.bak` is non-destructive and prevents accidental data loss during recovery.
2. **Cycle & Topology Validation:** Store validation should verify not only individual `ServiceSpec` instances (SS1..SS5) and `ServiceStatus` instances, but also run topological cycle detection (CS3) across the entire service dependency graph.
3. **Audit Trail Invariance:** Any recovery action (whether initiated via CLI or MCP) must emit an auditable event to the SQLite WAL audit ring recording the action (`service.check` or `service.repair`), result status, and backup file location.

---

## 5. Architectural Invariants for Service Recovery (`SR1..SR5`)

The new `service_recovery` module will enforce the following formal invariants:

- **SR1 (Conservation Law):**  
  $\text{valid\_services} + \text{invalid\_services} = \text{total\_services}$.
- **SR2 (Health Equivalence):**  
  $\text{healthy} \iff (\text{errors.is\_empty}() \land \text{invalid\_services} == 0)$.
- **SR3 (Error Accountability):**  
  $\text{invalid\_services} > 0 \implies \text{errors.len}() \ge \text{invalid\_services}$.
- **SR4 (Forensic Preservation):**  
  Corrupted or unreadable files must be renamed to a timestamped backup (`<path>.corrupt.<ts>.bak`) prior to writing a fresh store. Damaged data is never discarded silently.
- **SR5 (Canonical Reconstitution):**  
  A recovered or newly initialized default store must contain canonical services and achieve $\text{healthy} == \text{true}$ with zero validation errors.

---

## 6. Open Decisions & Implementation Roadmap

| Decision | Selected Approach | Rationale |
|---|---|---|
| **Module Location** | `code/aiosh-rust/aiosh-core/src/service_recovery.rs` | Consistent with `package_recovery.rs` and `distro_recovery.rs`. |
| **Quarantine Suffix** | `<path>.corrupt.<timestamp_ms>.bak` | Distinctive naming that clearly signals corrupted state while preventing name collisions. |
| **CLI Command Surface** | `aiosh service check [--fix] [--store <path>] [--json]` | Matches `aiosh package check` and `aiosh image check`. |
| **MCP Tool Surface** | `aios.service.check` (`auto_recover: bool`, `store_path: string`) | Standard JSON-RPC 2.0 tool matching `aios.package.check`. |
| **Suite Integration** | `SS10` in `tools/test_service_suites.py` | Extends master runner to 10/10 criteria across the Init & Service Supervision epic. |

---

## 7. Unknowns & Next Steps
- **Zero blocking unknowns.** All dependencies, data models, file I/O idioms, and test patterns are fully specified and established in the existing codebase.
- **Next Task:** `T-01392` (`recovery & validation: Specification`) — Formally specify the types, interfaces, error models, and audit contracts.
