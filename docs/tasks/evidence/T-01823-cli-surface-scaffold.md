# T-01823: Network Bootstrap / CLI Surface: Scaffold

## Overview
- **Task ID**: `T-01823`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Create module skeleton and interfaces for the CLI surface of Network Bootstrap.

## Changes
1. **CLI Dispatch in `code/aiosh-rust/aiosh-cli/src/main.rs`**:
   - Registered `Some("net") | Some("network") => cmd_network(&args[1..]),` in main command dispatch match expression.
   - Added `net, network` to the top-level usage/help text.
2. **Function Skeleton**:
   - Implemented `cmd_network(args: &[String]) -> i32`.
   - Wired `aiosh_core::network::validate_interface_name` and `aiosh_core::network_service::NetworkService`.
   - Scaffolded subcommands: `list`, `show`, `routes`, `dns`, `state`, `up`, `down`, `--help`.
3. **Compilation Verification**:
   - `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli` succeeded with exit code 0.
