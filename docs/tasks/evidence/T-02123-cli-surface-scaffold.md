# T-02123: CLI Surface Scaffold — PEP Decision Engine

## Overview
- **Task ID**: `T-02123`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Scaffolding Implementation
1. **CLI Dispatch Routing**:
   - Added `Some("pep") => cmd_pep(&args[1..]),` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Updated `aiosh --help` overview text with `aiosh pep <evaluate|rule-add|rule-list|rule-remove|status>`.
2. **Function Skeleton**:
   - Defined `fn cmd_pep(args: &[String]) -> i32`.
   - Integrated store path hygiene check via `aiosh_core::pep_decision_service::validate_pep_service_path`.
   - Implemented subcommands:
     - `evaluate`
     - `rule-add`
     - `rule-list`
     - `rule-remove`
     - `status`
     - `--help` / `-h`
     - Unknown subcommand handling with exit code 2 and audit emission.
3. **Compilation Verification**:
   - Ran `cargo check -p aiosh-cli`.
   - Clean compilation with zero errors or warnings.
