# Task Evidence: T-01852 - Network Bootstrap / automated tests: Specification

## 1. Overview
- **Task ID**: `T-01852`
- **Sub-Epic**: 6 (Network Bootstrap Automated Tests)
- **Goal**: Formally specify the automated testing suite covering happy path, failure path, audit effects, and cross-surface parity.

---

## 2. Specification: Test Suites & Architecture

### Suite 1: Rust Automated Integration Suite (`test_network_automated.rs`)
- **Location**: `code/aiosh-rust/aiosh-core/tests/test_network_automated.rs`
- **Fixture (`MockNetworkEnv`)**:
  - Encapsulates `tempfile::TempDir`.
  - Creates:
    - `<dir>/sys/class/net/lo/{operstate, address, mtu, flags}`
    - `<dir>/sys/class/net/eth0/{operstate, address, mtu, flags}`
    - `<dir>/sys/class/net/wlan0/{operstate, address, mtu, flags}`
    - `<dir>/proc/net/route` with header and rows for default gateway and local subnet.
    - `<dir>/etc/resolv.conf` with `nameserver` and `search` directives.
- **Test Cases**:
  1. `test_automated_mock_discovery_happy_path`:
     - Initializes `NetworkService` pointing to mock directories.
     - Scans interfaces: verifies `lo` (Loopback), `eth0` (Ethernet, Up, MAC `00:11:22:33:44:55`), `wlan0` (Wireless, Down, MAC `aa:bb:cc:dd:ee:ff`).
     - Scans routes: verifies default route to `192.168.1.1` metric 100 on `eth0`.
     - Scans DNS: verifies nameservers `["1.1.1.1", "8.8.8.8"]` and search domains `["localdomain", "example.com"]`.
     - Validates unified `NetworkState`.
  2. `test_automated_config_integration`:
     - Loads `NetworkConfig` with mock paths.
     - Validates configuration against invariants `NCONF1..NCONF6`.
     - Saves configuration atomically and reloads.
  3. `test_automated_fault_injection_missing_sysfs`:
     - Points `NetworkService` to non-existent sysfs path.
     - Asserts graceful empty interface list without crashing.
  4. `test_automated_fault_injection_corrupt_routes`:
     - Writes corrupt characters to mock `/proc/net/route`.
     - Asserts graceful fallback or partial parse without panicking.
  5. `test_automated_fault_injection_empty_resolv`:
     - Empty resolv.conf returns empty DNS nameservers or falls back to config.
  6. `test_automated_link_state_transitions`:
     - Executes `bring_up("wlan0")` and `bring_down("eth0")`.
     - Asserts state changes and validation.

---

### Suite 2: Python Cross-Surface E2E Smoke Suite (`test_network_e2e_smoke.py`)
- **Location**: `code/aiosh-cli/tests/test_network_e2e_smoke.py`
- **Test Cases**:
  1. `test_e2e_mock_filesystem_generation`: Generates hermetic sysfs/procfs/resolv.conf fixture.
  2. `test_e2e_cli_json_contract`: Validates CLI command responses against JSON schema.
  3. `test_e2e_mcp_json_contract`: Validates MCP tool call responses against tool schema.
  4. `test_e2e_cross_surface_parity`: Directly compares CLI and MCP JSON output for equivalence.
  5. `test_e2e_fault_injection`: Injects traversal interface names (`../bad`) and invalid IP strings, asserting standardized error envelopes.
  6. `test_e2e_audit_trail_assertions`: Asserts that consequential operations generate audit rows.

---

## 3. Invariants Coverage Matrix

| Invariant | Description | Tested By |
|---|---|---|
| `NTEST1` | Hermetic Isolation | `MockNetworkEnv` in temporary directories (no host mutation) |
| `NTEST2` | Cross-Surface Parity | `test_e2e_cross_surface_parity` in `test_network_e2e_smoke.py` |
| `NTEST3` | Fault Injection | `test_automated_fault_injection_*`, `test_e2e_fault_injection` |
| `NTEST4` | Audit Trail Integrity | `test_e2e_audit_trail_assertions` |
| `NTEST5` | Configuration Integration | `test_automated_config_integration` |
| `NTEST6` | Deterministic Resource Cleanup | Rust `TempDir` drop & Python `tempfile.TemporaryDirectory` context manager |
