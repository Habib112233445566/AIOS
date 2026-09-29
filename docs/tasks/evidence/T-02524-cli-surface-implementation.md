# Evidence: T-02524 Privilege Escalation Prevention CLI Surface Implementation

- **Task**: `T-02524`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Implementation
1. Implemented complete CLI subcommand suite in `code/aiosh-rust/aiosh-cli/src/main.rs`:
   - `aiosh privilege status [--actor A] [--store S] [--json]`: Displays/outputs active context, active tier, elevation status, and capabilities.
   - `aiosh privilege elevate --to <tier> [--grant G] [--actor A] [--caps C1,C2] [--store S] [--json]`: Evaluates and executes dynamic privilege elevation, requiring grants for upward jumps.
   - `aiosh privilege drop --to <tier> [--actor A] [--store S] [--json]`: De-escalates privilege level and trims capabilities.
   - `aiosh privilege revoke [--actor A] [--store S] [--json]`: Reverts context to baseline level and clears elevation grants.
   - `aiosh privilege check --cap <capability> [--actor A] [--store S] [--json]`: Verifies capability assignment.
   - `aiosh privilege list [--store S] [--json]`: Lists all registered actors.
2. Enforced:
   - Path hygiene on `--store` (rejects traversal `..`, control characters, > 1024 bytes).
   - Actor identifier sanitization (rejects control characters, empty strings).
   - Audit event emission via `classify_and_emit` on every subcommand invocation.
   - Dual output support (ANSI-sanitized plain text and structured JSON envelopes).
3. Added `PrivilegeCapability::parse_capability` and `PrivilegeCapability::as_str` in `aiosh_core::privilege_data_model`.
4. Added `load_from_path`, `save_to_path`, and `get_or_create_context` in `aiosh_core::privilege_service`.
