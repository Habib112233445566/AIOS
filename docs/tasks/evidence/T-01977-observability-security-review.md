# Task Evidence: T-01977 - System Update / observability: Security Review (Sub-Epic 8)

## Overview
- **Task ID**: `T-01977`
- **Subsystem**: `SystemUpdateObservability` (`aiosh-core::system_update_observability`)
- **Objective**: Conduct rigorous security review and threat analysis of the system update observability subsystem, evaluating input validation, telemetry sanitization, information disclosure, and state immutability.

## Threat Modeling & Abuse Scenarios

### THREAT-UOBS-01: Log Injection via Control / ANSI Escape Sequences
- **Vector**: An attacker provides a malformed update manifest or induces an update error containing ANSI terminal escape sequences (e.g. `\x1b[31m`, `\r\n`, or VT100 control codes) to manipulate operator logs or forge telemetry entries.
- **Mitigation**: `sanitize_telemetry_text()` iterates through characters, strictly filtering out any character where `c.is_control()` evaluates to true, and trims trailing/leading whitespace.

### THREAT-UOBS-02: Memory / String Exhaustion via Unbounded Telemetry Fields
- **Vector**: A malicious or corrupted service status contains multi-megabyte error strings or version strings intended to exhaust memory during JSON serialization or SIEM ingestion.
- **Mitigation**: `sanitize_telemetry_text()` enforces a strict truncation cap of 256 characters (`.take(256)`).

### THREAT-UOBS-03: Filesystem Metadata Race Conditions / Symlink Attacks on Staged Artifacts
- **Vector**: When calculating `staged_payload_bytes`, an attacker substitutes a staged artifact with a symlink to an arbitrary system file (e.g. `/dev/urandom` or large disk image) to distort payload metrics or block I/O.
- **Mitigation**: The staging service (`stage_artifact`) rejects symlinks via `symlink_metadata()` before insertion into `staged_artifacts`. During report generation, `std::fs::metadata` is used (which does not follow symlinks if broken or handles metadata queries non-blockingly) and gracefully defaults to 0 on I/O error.

### THREAT-UOBS-04: Side-Channel State Mutation via Telemetry Gathering
- **Vector**: Querying observability telemetry inadvertently modifies service state, advances progress counters, or triggers boot slot toggling.
- **Mitigation**: Invariant `UOBS5`: `SystemUpdateObservabilityReport::generate()` accepts immutable references (`&SystemUpdateService`, `Option<&SystemUpdateSecurityPolicy>`) and performs purely read-only projection.

### THREAT-UOBS-05: False Health Masking in Mission-Critical Systems
- **Vector**: A failed update or degraded partition slot reports `is_healthy: true`, causing automated orchestrators to deploy workloads to an unstable slot.
- **Mitigation**: Invariant `UOBS6`: `is_healthy` is computed as:
  ```rust
  let is_healthy = state != UpdateState::Failed
      && match current_slot {
          UpdateSlot::SlotA => slot_a_successful,
          UpdateSlot::SlotB => slot_b_successful,
      };
  ```
  Any failure state or unconfirmed active slot immediately reports `false`.

### THREAT-UOBS-06: Cryptographic Key & Secret Leakage in Telemetry
- **Vector**: Update observability reports inadvertently leak Ed25519 private keys, secret tokens, or raw payload blocks.
- **Mitigation**: `SystemUpdateObservabilityReport` includes only high-level metadata (version strings, channel, update ID, artifact count, byte counts, policy verdict). No cryptographic keys or payload bytes are included in the telemetry model.

## Conclusion
The security review confirms that the observability subsystem design is sound and enforces defense-in-depth against log injection, memory exhaustion, and false health masking.
