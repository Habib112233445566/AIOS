# Task Evidence: T-01871 - Network Bootstrap / observability: Research

## 1. Overview
- **Task ID**: `T-01871`
- **Sub-Epic**: 8 (Network Bootstrap Observability)
- **Goal**: Establish facts, constraints, and architecture for Network Bootstrap observability, telemetry, and health monitoring.

---

## 2. Research & Authoritative Sources

### 2.1 Linux Kernel Telemetry Interfaces
1. **`/proc/net/dev`**:
   - Colon-delimited multi-column table displaying cumulative rx/tx statistics:
     `bytes`, `packets`, `errs`, `drop`, `fifo`, `frame`, `compressed`, `multicast` for receive;
     `bytes`, `packets`, `errs`, `drop`, `fifo`, `colls`, `carrier`, `compressed` for transmit.
   - Authoritative source: `man 5 proc`, Linux kernel `net/core/net-procfs.c`.
2. **`/sys/class/net/<iface>/statistics/`**:
   - Per-interface sysfs hierarchy exposing granular 64-bit unsigned integer counters.
   - Virtual files: `rx_bytes`, `rx_packets`, `rx_errors`, `rx_dropped`, `tx_bytes`, `tx_packets`, `tx_errors`, `tx_dropped`.
   - `carrier` (0 or 1) indicating physical/virtual link carrier status.
   - Authoritative source: `Documentation/ABI/testing/sysfs-class-net`.
3. **Health & Connectivity Assessment**:
   - RFC 2863 (Interfaces Group MIB): operational status transitions (`up`, `down`, `testing`, `unknown`, `dormant`).
   - Route and resolver health: presence of active default gateway and responsive DNS resolvers.

---

## 3. Fact vs. Assumption Matrix

| Aspect | Fact | Assumption / Design Decision |
|---|---|---|
| Counters availability | Kernel maintains monotonic cumulative 64-bit counters in sysfs/procfs | Monotonic counters can reset upon link restart or module reload; rates should calculate diff between timestamps |
| Sampling overhead | Reading procfs/sysfs is non-blocking in kernel | Reads must be bounded in buffer size to prevent memory spike |
| Platform portability | `/proc` and `/sys` exist only on Linux | On non-Linux (e.g. Windows/macOS test harnesses) or missing paths, service must fall back cleanly without panicking |
| Health verdicts | Linux kernel does not synthesize a composite "health" rating | AIOS derives composite `Healthy`, `Degraded`, `Critical` based on carrier, link state, default routes, and drop rates |
| Telemetry history | Kernel does not store historical time-series in sysfs | In-memory ring buffer with fixed capacity (`MAX_SNAPSHOT_HISTORY = 60`) provides recent trend analysis |

---

## 4. Invariants Formulated (`NOBS1..NOBS6`)
1. **`NOBS1` (Non-Blocking Bounded Sampling)**: File I/O for telemetry reading must be bounded, never block indefinitely, and cap string reads at 64 KB.
2. **`NOBS2` (Graceful Degradation)**: Missing procfs/sysfs nodes or malformed counter entries fall back to zeroed values or empty collections without errors or panics.
3. **`NOBS3` (Diagnostic Health Classification)**: Composite health logic provides deterministic verdicts (`Healthy`, `Degraded`, `Critical`) with explicit issue descriptions.
4. **`NOBS4` (Bounded History Ring)**: In-memory history buffer is strictly capped at `MAX_SNAPSHOT_HISTORY` (60 entries) to guarantee zero memory leakage over indefinite runtime.
5. **`NOBS5` (Cross-Surface Parity)**: All telemetry data structures serialize to canonical JSON compatible with CLI, MCP, and external telemetry scrapers.
6. **`NOBS6` (Atomic Persistence & Path Hygiene)**: Metrics snapshots persisted to disk enforce path hygiene ($\le 1024$ chars, no `..`, no control characters), 1 MB size cap, and atomic sibling write/rename.

---

## 5. Decisions Needed for Specification (T-01872)
- Counter types: `u64` for all byte and packet counters.
- Health criteria thresholds: Packet drop rate > 5% or error rate > 2% flags `Degraded`.
- Metric history buffer size: default 60 snapshots.
