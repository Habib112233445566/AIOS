# Task Evidence: T-01891 - Network Bootstrap / recovery & validation: Research

## 1. Overview
- **Task ID**: `T-01891`
- **Sub-Epic**: 10 (Network Bootstrap Recovery & Validation)
- **Goal**: Research Linux network drift detection, configuration corruption patterns, automated healing mechanisms, and state reconciliation invariants.

---

## 2. Research Findings

### 2.1 Failure Modes & Corruption Vectors
1. **Dangling / Orphan Routes**: Routes referencing network interfaces that have been unplugged, renamed, or failed to initialize, resulting in unreachable destinations and blackhole routing.
2. **Missing Loopback Interface**: Absence of the virtual `lo` interface (`127.0.0.1/8`) prevents inter-process communication, local daemon binding, and health checks.
3. **Unconfigured / Empty DNS Resolvers**: Truncated `/etc/resolv.conf` or absent DNS configuration leaving the host incapable of name resolution.
4. **Configuration File Corruption**: Partial writes, process crashes during write, or disk errors corrupting JSON configuration files on disk.
5. **Path Traversal & Injection Risks**: Untrusted configuration or store file paths with parent directory traversal (`..`) or control characters.

### 2.2 Recovery & Self-Healing Strategies
- **Non-Destructive Quarantine**: Before overwriting or resetting corrupted configuration files, copy the damaged file to `<filename>.bak.<timestamp>` to preserve forensic evidence.
- **Dangling Route Pruning**: Remove routes whose interface field does not match any known active interface in the inventory.
- **Loopback Self-Healing**: Automatically synthesize a healthy loopback interface (`lo`, `127.0.0.1/8`, `OperState::Up`, `InterfaceType::Loopback`) if missing.
- **DNS Fallback Injection**: When zero nameservers are configured, safely inject canonical default resolvers (e.g., `1.1.1.1`, `8.8.8.8`) to restore basic network functionality.
- **Atomic State Persistence**: Save reconciled states via sibling temporary files `.{name}.tmp.{pid}` with RAII drop guard and Unix `0600` permissions.

### 2.3 Proposed Invariants (`NVAL1..NVAL6`)
- `NVAL1` (Interface Count Parity): `valid_interfaces + invalid_interfaces == total_interfaces`.
- `NVAL2` (Route Integrity): Every route must associate with an existing, valid interface. Dangling routes are detected and flagged.
- `NVAL3` (DNS Health): Nameservers are validated as valid IP strings. Empty nameservers trigger a degraded/unhealthy status.
- `NVAL4` (Deterministic Health State): `healthy == true` if and only if `errors.is_empty() && invalid_interfaces == 0 && dangling_routes.is_empty() && dns_configured`.
- `NVAL5` (Non-Destructive Quarantine): Damaged files are cloned to timestamped `.bak` files prior to mutation or replacement.
- `NVAL6` (Path Hygiene & Bounds): Path $\le 1024$ chars, `.json` extension, no `..`, 1 MB size ceiling, atomic swap.

Status: Research completed. Ready for specification in `T-01892`.
