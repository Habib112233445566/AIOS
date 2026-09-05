# T-01218: Package Management - Core Service: Hardening

## Metadata
- **Task ID:** `T-01218`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Hardening
- **Status:** Complete

## 1. Hardening Defenses & Invariants

### 1. Payload & Storage Size Ceilings
- **10 MiB Store Size Cap**: Enforced 10 MiB (`10,485,760` bytes) hard ceiling on package store persistence and loading in `PackageStore::load_from_path`.
- **10,000 Package Entity Limit**: Capped total registered packages to 10,000 in `PackageStore::load_from_path` to prevent memory exhaustion and algorithmic degradation.
- **1 MiB Actions Payload Cap**: Enforced 1 MiB size ceiling on `--actions` file reads and inline JSON payloads in `aiosh package plan`, preventing memory bloat from oversized batches.
- **Transaction Bounds**: Re-verified bounds on transaction actions (`1..=256`), rejecting empty batches and overflowing lists in `plan_transaction`.

### 2. Resource Cleanup & Leaks Prevention
- **Temporary File RAII Cleanup**: In `PackageStore::save_to_path`, ensured that any failure during temporary file writing or atomic renaming triggers explicit removal of the `.tmp` file, preventing orphaned temporary files on error paths.
- **No Residual Locks**: Store operations operate purely in-memory and write atomically via transient file descriptors, ensuring no SQLite locks or file descriptor leaks.

### 3. Explicit JSON Result Envelopes
- Standardized all failure modes across the `aiosh package` CLI surface (`list`, `show`, `plan`) when invoked with `--json`:
  - `LOAD_STORE_FAILED`: Store loading or parsing errors.
  - `INVALID_ARGUMENT`: Unknown formats, unknown states, or missing arguments.
  - `PACKAGE_NOT_FOUND`: Package missing during `aiosh package show`.
  - `PAYLOAD_TOO_LARGE`: Action files or payloads exceeding 1 MiB.
  - `INVALID_JSON`: Malformed JSON in transaction actions.
  - `PLAN_FAILED`: Invariant violations (`CS3` unmet dependencies, `CS4` delta mismatch).

### 4. Honest Audit Trail (ADR-0035 / A F-2)
- Synchronous audit logging via `classify_and_emit` integrated across all failure and success branches:
  - Every rejected action, invalid argument, missing package, or planning error emits a structured failure event with SHA-256 integrity into the SQLite WAL audit ring.

## 2. Test Verification Matrix
- **Standalone Package Suites Runner**: Added criterion `PM2` to `tools/test_package_suites.py`.
  - `PM1`: package data model integrity & invariants (`PM1..PM5`) — PASS.
  - `PM2`: package core service integrity & invariants (`CS1..CS5`) — PASS.
- **Unit Suites**:
  - `code/aiosh-rust/aiosh-core/src/package_service.rs` (`test_package_store_hardening_bounds_and_cleanup` PASS).
  - `code/aiosh-rust/aiosh-core/tests/test_package_service.rs` (`test_package_store_hardening_and_error_paths` PASS).
- **CLI & MCP Regression**:
  - `aiosh test_cmd_package_flow` (PASS).
  - `aiosh-mcp test_mcp_package_tools` (PASS).
