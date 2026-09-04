# T-01217: Package Management - Core Service: Security Review

## Metadata
- **Task ID:** `T-01217`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Security Review
- **Status:** Complete

## 1. Threat Modeling & Abuse Scenarios

### Scenario 1: Path Traversal & Uncontrolled File Overwrite
- **Vector:** Supplying malicious path parameters (e.g., `../../../../etc/passwd` or `/dev/null`) to `--store` or `store_path`.
- **Analysis:** `load_from_path` strictly validates file existence and bounds reading to 10 MiB. `save_to_path` creates atomic temporary files with explicit `0o644` permissions before atomic renaming.
- **Verdict:** Mitigated. In MCP, `dispatch::recorded_call` enforces PEP authorization and path scope checks.

### Scenario 2: Denial of Service via Storage & Memory Exhaustion
- **Vector:** Feeding multi-gigabyte store files or sending transactions with tens of thousands of actions to cause OOM.
- **Analysis:**
  - `load_from_path` verifies metadata length `<= 10 MiB` and streams via `io::Read::take(10 * 1024 * 1024 + 1)`.
  - Invariant PM2 & CS2: `actions` count is strictly bounded to `1..=256`.
  - Specs are checked against bounded string lengths (`name <= 128`, `version <= 64`, `desc <= 4096`).
- **Verdict:** Mitigated.

### Scenario 3: Dependency Resolution Hijacking & Graph Poisoning
- **Vector:** Declaring cyclic dependencies or unresolvable transitive trees to wedge transaction planning.
- **Analysis:**
  - `validate_package_spec` rejects self-dependencies and duplicate dependencies.
  - Invariant CS3: `plan_transaction` and `execute_transaction` strictly verify dependency closure: any non-optional dependency must already be `Installed` or present as an `Install` action in the transaction batch.
- **Verdict:** Mitigated.

### Scenario 4: Transaction Tampering & Disk Quota Bypasses
- **Vector:** Altering `total_size_delta_bytes` in a transaction payload to bypass storage quota checks while actually expanding rootfs.
- **Analysis:**
  - Invariant CS4: `execute_transaction` recalculates the delta independently from installed and removed package specs. If `tx.total_size_delta_bytes != calculated_delta`, execution is rejected.
  - Invariant CS2: Transaction ID is computed via SHA-256 over actions and delta bytes.
- **Verdict:** Mitigated.

## 2. Audit Trail & PEP Verification
- All CLI operations (`package list`, `show`, `plan`, `validate`) write audit records via `classify_and_emit`.
- All MCP operations (`aios.package.list`, `get`, `plan`, `validate`) route through `dispatch::recorded_call`, enforcing PEP authorization tokens and logging SHA-256 canonical call records into SQLite WAL.
