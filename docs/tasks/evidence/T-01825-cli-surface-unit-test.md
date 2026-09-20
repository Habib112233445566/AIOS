# T-01825: Network Bootstrap / CLI Surface: Unit Test

## Overview
- **Task ID**: `T-01825`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Add focused automated tests for the CLI surface of Network Bootstrap.

## Test Implementation
Added `mod network_cli_tests` to `code/aiosh-rust/aiosh-cli/src/main.rs` covering:
1. `test_network_cli_help_and_subcommands`:
   - Empty args, `--help`, and `-h` return 0 and output usage text.
   - Unknown subcommand returns exit code 2 and standard error payload (`UNKNOWN_SUBCOMMAND`) in both text and JSON modes.
2. `test_network_cli_path_hygiene`:
   - Enforces length $\le 1024$ chars on `--sysfs`, `--procfs`, and `--resolv` flags (returns exit code 2, error code `PATH_TOO_LONG`).
   - Enforces absence of control characters (`\n`, `\t`, `\0`) in paths (returns exit code 2, error code `PATH_CONTAINS_CONTROL_CHAR`).
3. `test_network_cli_arg_validation`:
   - `show` requires interface name (missing name returns 2, `MISSING_INTERFACE_NAME`).
   - Interface names must satisfy `validate_interface_name` ($\le 15$ chars, valid characters; e.g. `eth0;bad` returns 2, `INVALID_INTERFACE_NAME`).
   - `up` and `down` require and validate interface name.
4. `test_network_cli_with_mock_fs`:
   - Populates isolated temporary directory hierarchy with mock `sysfs` (`eth0`), `procfs` (`route`), and `resolv.conf`.
   - Tests `list`, `show eth0`, `show nonexistent`, `routes`, `dns`, and `state` across both human-readable and `--json` modes.
   - Asserts exit code 0 on success, exit code 1 on not-found (`INTERFACE_NOT_FOUND`), and cleans up mock resources.

## Verification
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh -- network_cli_tests` executes and passes all tests.
