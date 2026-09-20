# T-01826: Network Bootstrap / CLI Surface: Integration

## Overview
- **Task ID**: `T-01826`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Integrate the CLI surface of Network Bootstrap with the surrounding system.

## Integration Details
1. **Production Surface Registration**:
   - `aiosh net` and `aiosh network` command dispatch hooked in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Subcommands available: `list`, `show <iface>`, `routes`, `dns`, `state`, `up <iface>`, `down <iface>`.
   - Options supported: `--sysfs <path>`, `--procfs <path>`, `--resolv <path>`, `--json`.
2. **Cross-Substrate Integration & Verification**:
   - Created `code/aiosh-cli/tests/test_network_cli_smoke.py` executing the real compiled binary (`aiosh.exe`).
   - Covered:
     - Help discovery and unknown subcommand error handling.
     - Path hygiene validation (length bounds, control character rejection).
     - Interface argument validation and injection prevention.
     - Mock filesystem operations for `list`, `show`, `routes`, `dns`, and `state` across both human-readable text and `--json` envelope outputs.
3. **Execution Results**:
   - `test_network_cli_smoke.py` passed all 4 test suites with 0 errors.
