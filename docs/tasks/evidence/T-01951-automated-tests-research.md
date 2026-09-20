# Task Evidence: T-01951 (System Update / automated tests: Research)

## 1. Objective
Establish facts, constraints, testing vectors, and prior art for the comprehensive automated test suite of the AIOS System Update Mechanism (Sub-Epic 6, tasks T-01951 through T-01960).

## 2. Fact vs. Assumption Analysis

### Facts (Established Codebase & Upstream Standards)
1. **Existing Core Tests**:
   - `code/aiosh-rust/aiosh-core/tests/test_system_update.rs` covers isolated enum/struct validations (UPD1..UPD6), basic state transitions (`Idle` -> `Checking` -> `Downloading` -> `Verifying` -> `Applying` -> `ReadyToReboot` -> `Verified` -> `Idle`), and basic rollback transitions.
   - `code/aiosh-rust/aiosh-core/tests/test_system_update_config.rs` covers `SystemUpdateConfig` validation, path hygiene, env overrides, and atomic persistence.
   - `code/aiosh-mcp/tests/test_system_update_smoke.py` tests JSON-RPC MCP tools (`system_update_status`, `system_update_stage`, `system_update_apply`, `system_update_rollback`).
2. **Missing Test Coverage**:
   - Comprehensive multi-step end-to-end lifecycle execution with real temporary filesystems.
   - Fault-injection testing: payload corruption, truncation, tampered cryptographic digests, symlink redirection during staging, and storage quota/disk space exhaustion.
   - Boot failure and automatic rollback simulation: verifying that failure to confirm boot cleanly triggers rollback and restores functional slot pointers.
   - Concurrency & state collision testing: asserting that concurrent or out-of-order calls (e.g. stage during idle, apply during downloading) are strictly rejected without corrupting state.
3. **Upstream Architecture & Prior Art**:
   - **ChromeOS `update_engine`**: Uses isolated delta and full update mock harnesses with mock bootloaders that track `boot_times_left` counters and test automatic fallback.
   - **systemd-sysupdate / systemd-boot**: Specifies atomic partition flipping and boot counters (`try_boot` pattern) where successful boots mark the slot good and unsuccessful boots decrement counters until automatic fallback triggers.
   - **NIST SP 800-193 (Platform Firmware Resiliency)**: Mandates testing of three core pillars: Protection (unauthorized update prevention), Detection (tamper/corruption detection before boot), and Recovery (automatic rollback to known good state on boot failure).

### Assumptions
1. Integration tests can execute in isolated temporary directories without requiring root privileges or physical disk partitions.
2. End-to-end simulations can model mock boot cycles by invoking `apply_update()`, simulating a reboot boundary, and conditionally invoking `confirm_boot()` or `rollback()`.
3. Test harnesses should exist both as native Rust integration tests (`tests/test_system_update_e2e.rs`) for fast, zero-dependency CI execution, and as Python smoke tests for MCP/CLI integration.

## 3. Decisions & Test Vectors Identified
1. **Vector 1: Clean A/B Lifecycle (Happy Path)**
   - Slot A active -> check manifest -> stage kernel & rootfs -> verify digests -> apply update (target becomes Slot B, next boot slot B) -> confirm boot -> Slot B verified and active.
2. **Vector 2: Corrupted & Truncated Payload Fault Injection**
   - Inject bit flip in artifact data -> digest calculation mismatch -> transition to `Failed` -> assert staging files cleaned or quarantined -> assert slot remains Slot A.
   - Inject truncated artifact -> size mismatch -> immediate rejection before disk write.
3. **Vector 3: Symlink & Directory Traversal Injection**
   - Attempt staging into a symlink target -> immediate rejection (`symlink_metadata`).
   - Attempt staging with path traversal in filename (`../../etc/shadow`) -> rejected during manifest validation.
4. **Vector 4: Storage Quota & Overflow Injection**
   - Configure `max_payload_bytes` low -> stage payload exceeding quota -> rejection with quota error -> assert state not corrupted.
5. **Vector 5: Boot Failure & Rollback Simulation**
   - Apply update to Slot B -> simulate failed boot (no confirmation) -> trigger rollback -> slot restored to Slot A -> state returns to `Idle`.
6. **Vector 6: State Collision & Idempotency**
   - Attempt `apply_update` while in `Downloading` state -> rejected with `UPD_STATE_ERROR`.
   - Attempt concurrent staging operations -> rejected or properly sequenced.

## 4. References & Citations
- Chromium OS Update Engine Source & Test Architecture: `https://chromium.googlesource.com/chromiumos/platform/update_engine/`
- systemd Boot Loader Specification & Automatic Boot Assessment: `https://systemd.io/AUTOMATIC_BOOT_ASSESSMENT/`
- NIST Special Publication 800-193: "Platform Firmware Resiliency Guidelines"
