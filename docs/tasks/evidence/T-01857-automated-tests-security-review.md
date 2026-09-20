# Task Evidence: T-01857 - Network Bootstrap / automated tests: Security Review

## 1. Overview
- **Task ID**: `T-01857`
- **Sub-Epic**: 6 (Network Bootstrap Automated Tests)
- **Goal**: Perform comprehensive security review of the automated test harness, mock environment generators, and test runners.

---

## 2. Threat Modeling & Abuse Scenarios

### `THREAT-NTEST-01`: Mock Fixture Directory Traversal / Host Filesystem Escape
- **Scenario**: A malformed mock fixture generates paths that resolve outside the temporary test directory (e.g. symlinks pointing to `/etc` or `C:\Windows`).
- **Impact**: Accidental corruption of host files or unauthorized reads of host system files during testing.
- **Evaluation**:
  - `MockNetworkEnv` in Rust and `create_mock_environment` in Python create strictly contained subdirectories (`sys/class/net`, `proc/net`, `etc/resolv.conf`) inside a fresh `TempDir` / `TemporaryDirectory`.
  - No symlinks are created; only regular files and directories are populated.
  - Path traversal attempts in input parameters are explicitly tested and rejected by `validate_interface_name` and `NCONF1`.

### `THREAT-NTEST-02`: Insecure Temporary Directory Permissions
- **Scenario**: Temporary mock test directories are created with world-writable permissions (e.g., `0777`), allowing other local users or concurrent processes to modify mock network files during a test run.
- **Impact**: Race condition, test result tampering, or information disclosure.
- **Evaluation**:
  - Temporary directories are generated using standard OS-provided secure temp directory APIs (`tempfile::tempdir` in Rust, `tempfile.TemporaryDirectory` in Python) which enforce exclusive user ownership (`0700` on Unix).

### `THREAT-NTEST-03`: Unbounded Test Execution & Memory DoS
- **Scenario**: Corrupt route tables or large sysfs trees cause infinite loops or massive memory allocations during test execution.
- **Impact**: Test runner hang or CI runner out-of-memory crash.
- **Evaluation**:
  - `read_bounded_string` limits sysfs reads to 64 KB, route table reads to 1 MB, and resolv.conf to 64 KB.
  - `NetworkConfig` enforces `MAX_CONFIG_FILE_BYTES = 1,048,576` (1 MB) and `max_payload_bytes` bounds.
  - Invariant `NTEST3` verifies that corrupt rows and oversized files are handled in constant time without memory spikes.

### `THREAT-NTEST-04`: Test Residue & Resource Leakage
- **Scenario**: Tests fail or panic, leaving temporary directories, lock files, or child processes on disk.
- **Impact**: Disk exhaustion in CI environments.
- **Evaluation**:
  - Invariant `NTEST6` mandates deterministic resource cleanup.
  - In Rust, `tempfile::TempDir` implements `Drop` to automatically remove the directory when leaving scope.
  - In Python, `with tempfile.TemporaryDirectory()` guarantees directory cleanup on normal exit or exception.

---

## 3. Recommendations for Hardening (`T-01858`)
1. Ensure explicit RAII assertions and error handling in mock environment teardown.
2. In Python, wrap test execution in explicit try/finally or context managers to guarantee cleanup even if assertions fail.
3. Validate that error paths emit explicit, auditable messages rather than panics.
