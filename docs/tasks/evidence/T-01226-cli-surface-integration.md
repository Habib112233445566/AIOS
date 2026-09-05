# T-01226: Package Management - CLI Surface: Integration

## Metadata
- **Task ID:** `T-01226`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Integration
- **Status:** Complete

## 1. Integration Scope & Production Call Paths
The package management CLI surface has been fully integrated into the primary binary entry points of `aiosh`:
- **CLI Subcommand Registration**:
  - `aiosh package` is routed directly from `main()` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
  - Top-level help usage string updated to expose all package management actions:
    ```text
    aiosh package <list|show|search|plan|apply|validate>  Linux Package Management & Store Control
    ```
  - Subcommands supported:
    1. `validate` (`--name <name>` or `--spec <json_or_file>`)
    2. `list` (`[--format <fmt>] [--state <st>] [--pattern <pat>] [--limit <n>] [--store <path>] [--json]`)
    3. `show` (`<name> [--store <path>] [--json]`)
    4. `search` (`<pattern> [--limit <n>] [--store <path>] [--json]`)
    5. `plan` (`--actions <json_or_file> [--dry-run] [--store <path>] [--json]`)
    6. `apply` (`(--actions <json_or_file> | --plan <json_or_file>) [--dry-run] [--yes] [--store <path>] [--json]`)

## 2. Cross-Substrate Parity & Audit Logging
- **Canonical JSON Representation**: CLI JSON output schemas strictly mirror the `aiosh_core::package` model structures (`PackageSpec`, `PackageTransaction`, `PackageTransactionReport`).
- **Audit Integration (ADR-0035)**:
  - Both successful applications and rejected operations (such as missing dependencies, invalid formats, malformed JSON, and payload size overflows) call `classify_and_emit(...)` ensuring persistence into `audit.db`.
- **Atomic File Store Persistence**:
  - Store mutations executed via `apply` utilize atomic replacement via temporary file swap and permission retention in `PackageStore::save_to_path`.

## 3. Automated Test Runner Suite Integration
- Added criterion **`PM3`** to `tools/test_package_suites.py`:
  - `PM3`: `package CLI surface commands & options (validate/list/show/search/plan/apply)` invoking `cargo test --bin aiosh test_cmd_package_flow`.
- Execution output:
  ```text
  [+] PM1 package data model integrity & invariants (PM1..PM5)
  [+] PM2 package core service integrity & invariants (CS1..CS5)
  [+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)

  PASS: package_suites criteria (PM1..PM3)
  ```

## 4. End-to-End Production Surface Verification
Direct binary execution via CLI runner:
```bash
cargo run --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh -- package list --json
```
Output verified against canonical packages (`apk-tools`, `bash`, `busybox`, `coreutils`, `curl`, `libc6`, `libssl3`, `musl`).
