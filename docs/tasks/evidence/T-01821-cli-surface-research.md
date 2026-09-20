# Task Evidence: T-01821 - Network Bootstrap / CLI Surface: Research

## Metadata
- **Task ID:** `T-01821`
- **Sub-Epic:** Sub-Epic 3: Network Bootstrap / CLI Surface
- **Component:** `code/aiosh-rust/aiosh-cli/src/main.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. CLI Surface Design & Research

The Network Bootstrap CLI (`aiosh net` / `aiosh network`) exposes operator commands for inspecting and modifying host networking state.

### Subcommands Researched

1. **`aiosh net list [--json] [--sysfs <path>]`**:
   - Lists discovered network interfaces with operational state, MAC address, MTU, and flags.
2. **`aiosh net show <iface> [--json] [--sysfs <path>]`**:
   - Displays detailed attributes for a specific interface.
3. **`aiosh net routes [--json] [--procfs <path>]`**:
   - Displays the IPv4 routing table sorted by metric ascending.
4. **`aiosh net dns [--json] [--resolv <path>]`**:
   - Displays configured nameservers and search domains.
5. **`aiosh net state [--json] [--sysfs <path>] [--procfs <path>] [--resolv <path>]`**:
   - Generates and outputs a unified network state snapshot.
6. **`aiosh net up <iface> [--sysfs <path>]`**:
   - Brings an interface up (mutating operstate).
7. **`aiosh net down <iface> [--sysfs <path>]`**:
   - Brings an interface down (mutating operstate).

---

## 2. Invariants Formulated (`NCLI1..NCLI6`)

- **`NCLI1` (Uniform Response Envelopes)**: When `--json` is specified, outputs `{"code": 0, "data": ..., "error": null}` on success, `{"code": 1, ...}` on domain error (e.g. `INTERFACE_NOT_FOUND`), and `{"code": 2, ...}` on validation/usage error.
- **`NCLI2` (Terminal Output Sanitization)**: Human-readable output must sanitize ANSI escape sequences (`sanitize_terminal`) to prevent terminal injection.
- **`NCLI3` (Path Hygiene Checks)**: `--sysfs`, `--procfs`, and `--resolv` flags are validated: length $\le 1024$, no ASCII control characters.
- **`NCLI4` (Interface Name Safety)**: Subcommand positional arguments (`<iface>`) are validated against `validate_interface_name` (`NET1`), preventing directory traversal or command injection.
- **`NCLI5` (Audit Trail Integration)**: Every invocation emits a structured audit record into the SQLite audit ring via `classify_and_emit`.
- **`NCLI6` (Hermetic Cross-Platform Execution)**: All subcommands accept `--sysfs`, `--procfs`, and `--resolv` overrides for mock testing on non-Linux hosts.
