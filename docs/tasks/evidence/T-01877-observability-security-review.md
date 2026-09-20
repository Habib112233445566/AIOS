# Task Evidence: T-01877 - Network Bootstrap / observability: Security Review

## 1. Overview
- **Task ID**: `T-01877`
- **Sub-Epic**: 8 (Network Bootstrap Observability)
- **Goal**: Perform comprehensive security review of `NetworkObservabilityService` in `code/aiosh-rust/aiosh-core/src/network_observability.rs`.

---

## 2. Threat Modeling & Abuse Scenarios

| Threat ID | Threat Vector | Mechanism / Attack Surface | Mitigation / Invariant | Status |
|---|---|---|---|---|
| `THREAT-NOBS-01` | Path Traversal & Injection | Malicious `snapshot_path` containing `..`, control characters, or excessive length | `validate_observability_path` rejects paths with `ParentDir`, length > 1024, or control characters (`NOBS6`) | Mitigated |
| `THREAT-NOBS-02` | Virtual Filesystem Parsing DoS | Enormous `/proc/net/dev` with millions of lines causing CPU exhaustion | File read bounded to 64 KB (`NOBS1`); line processing capped at 1,024 lines | Mitigated |
| `THREAT-NOBS-03` | Memory Exhaustion (OOM) | Unbounded snapshot history ring buffer or oversized snapshot JSON files | `history_capacity` clamped to $[1, 1000]$ (`NOBS4`); snapshots capped at 1 MB (`MAX_OBSERVABILITY_FILE_BYTES`) | Mitigated |
| `THREAT-NOBS-04` | Node Tampering / Missing Nodes | Missing or unreachable procfs/sysfs nodes causing process crash | Non-blocking reads; graceful degradation to zeroed counters on missing/inaccessible nodes (`NOBS2`) | Mitigated |
| `THREAT-NOBS-05` | Integer Overflow in Health Probes | Arithmetic overflow during error/drop rate ratio calculation | Saturated or checked multiplication (`saturating_mul`) prevents panic on extreme counter values | Mitigated |

---

## 3. Findings & Recommendations for Hardening (T-01878)
1. Add explicit line limit ($\le 1024$) in `collect_statistics()` when iterating over `/proc/net/dev`.
2. Use `saturating_mul(20)` instead of `* 20` when calculating error/drop thresholds in `evaluate_health()`.
3. Ensure `history_capacity` in `with_paths()` is strictly clamped to $[1, 1000]$.
