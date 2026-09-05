# T-01223: Package Management - CLI Surface: Scaffold

## Metadata
- **Task ID:** `T-01223`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Scaffold
- **Status:** Complete

## 1. Scaffold Deliverables
- Extended `cmd_package` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Added `Some("search")` handler: implements convenience querying by substring pattern (`PackageQuery::name_pattern`), supporting both tabular output and `--json`.
  - Added `Some("apply")` scaffold handler: emits audit event and returns structured code `2` with `NOT_IMPLEMENTED` envelope, establishing the contract skeleton for T-01224.
  - Updated CLI usage string under `--help` / `-h` with complete subcommand syntax (`validate`, `list`, `show`, `search`, `plan`, `apply`).
- Extended unit test suite in `aiosh-cli/src/main.rs` (`test_cmd_package_flow`):
  - Verified `search` with valid pattern (exit code `0`).
  - Verified `search` with `--json` format (exit code `0`).
  - Verified `search` missing pattern argument (exit code `2`).
  - Verified `apply` scaffold execution (exit code `2`).
- Compilation verified cleanly with `cargo test --bin aiosh test_cmd_package_flow` (PASS).
