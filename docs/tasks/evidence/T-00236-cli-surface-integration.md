# T-00236 — Phase 0 — Release Packaging & Backup / CLI surface: Integration

## Goal
Integrate the CLI surface of Release Packaging & Backup with the surrounding system.

## Completion Notes
1. **Call Path Wiring**:
   - The CLI commands (`release` and `backup`) were explicitly wired into the primary dispatch loop in `aiosh-cli/src/main.rs`.
   - The commands delegate execution into `aiosh-core`'s unified `release.rs` module, which is the canonical source of truth shared by both the CLI and MCP substrates.

2. **Discoverability**:
   - The help output (`--help` / `-h` or running without arguments) has been updated to include:
     - `aiosh release generate  Create bootable ISO`
     - `aiosh backup create  Create system snapshot zip`

3. **Cross-Substrate Parity & Databases**:
   - Substrate parity is fully preserved. Both Python (`aiosh_mcp/release.py`) and Rust CLI execute conceptually identical flows, routing ultimately through the `AuditRing` schema to generate `audit_logs` entries compliant with `ADR-0035`.
   - The outputs match the `ok_out` / `err_out` envelope used by all other CLI functions (e.g. `aiosh agent`, `aiosh task`).

## Acceptance Criteria Verified
- [x] Feature reachable through its production surface (via `aiosh release` and `aiosh backup`).
- [x] Integration smoke passes end-to-end (tested in T-235).
