# T-01727: Hardware Detection — CLI Surface Security Review

## Metadata
- **Task ID**: `T-01727`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Threat Modeling & Attack Surface Analysis
Evaluated the CLI surface `aiosh hw <subcommand>` in `code/aiosh-rust/aiosh-cli/src/main.rs`:

### Scenario CS-1: Terminal Escape Injection in Human Output
- **Hazard**: Malicious or corrupted sysfs strings (e.g., custom firmware product strings or simulated hardware fixtures) could contain ANSI escape sequences (`\x1b[...]`) designed to rewrite terminal prompts or spoof successful verification.
- **Remediation**: Pass all user-facing strings through `sanitize_terminal` in `main.rs` before stdout/stderr emission.

### Scenario CS-2: Whitespace / Empty Argument Bypass on `show`
- **Hazard**: Providing a device ID composed entirely of whitespace (`aiosh hw show "   "`) could bypass simple empty checks and trigger unnecessary scans.
- **Remediation**: Check `target_id.trim().is_empty()` and return exit code 2 with `MISSING_DEVICE_ID`.

### Scenario CS-3: Unbounded Memory Allocation on `--file` Verification
- **Hazard**: An attacker passing a pseudo-node (e.g. `/dev/zero`) or multi-gigabyte file to `aiosh hw verify --file <path>` could exhaust memory.
- **Remediation**: Verify `meta.is_file()` and check `meta.len() <= 10 * 1024 * 1024` (10MB limit) before in-memory buffering.

### Scenario CS-4: Audit Ring Integrity
- **Hazard**: CLI invocations failing before parameter validation could bypass audit trail logging.
- **Remediation**: Ensure all early-return pathways (argument length errors, control character errors, missing subcommands) emit an audit event via `classify_and_emit` to the SQLite WAL audit ring.

---

## 2. Hardening Plan for T-01728
1. Implement `target_id.trim().is_empty()` validation on `show`.
2. Wrap device table and field outputs with `sanitize_terminal`.
3. Add unit test assertions in `test_hardware_cli_coverage` for whitespace device ID and terminal sanitization.
