# T-02124: CLI Surface Implementation — PEP Decision Engine

## Overview
- **Task ID**: `T-02124`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Implementation Summary
Implemented `cmd_pep` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
1. **Subcommands Implemented**:
   - `evaluate` (`eval`):
     - Parses `--subject`, `--action`, `--resource`, `--algorithm`, `--store`, `--json`.
     - Validates input strings via `PepRequest::new`, rejecting `..` traversal and control chars (exit code 2).
     - Evaluates request with specified or default combining algorithm (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`).
     - Emits audit row with outcome and decision metadata.
     - Returns exit code `0` on `Permit`, `1` on `Deny`, `2` on validation error.
   - `rule-add` (`add-rule`):
     - Parses `--id`, `--subject`, `--action`, `--resource`, `--effect`, `--priority`, `--desc`, `--store`, `--json`.
     - Enforces ID hygiene (non-empty, $\le 128$ chars, no control chars), resource path hygiene, and effect validity (`permit` or `deny`).
     - Adds rule to `PepDecisionService` and saves atomically via `save_to_path`.
     - Emits audit row.
   - `rule-list` (`list-rules`):
     - Lists all loaded rules with optional `--subject` and `--action` filters.
     - Supports structured JSON output (`--json`) or tabular human-readable output.
   - `rule-remove` (`remove-rule`):
     - Removes rule by ID.
     - Saves updated store atomically. Returns code `0` on success, `1` if not found.
   - `status`:
     - Displays metrics: rule count, unique subjects, unique actions, max capacity, default algorithm, and store existence.
   - `--help` / `-h`:
     - Prints usage text and returns exit code `0`.
   - Unknown subcommands:
     - Returns exit code `2` with structured error and audit emission.
2. **Path & Terminal Hygiene**:
   - Validates store path with `validate_pep_service_path`.
   - Passes all error messages through `sanitize_terminal` to prevent terminal injection.
3. **Build Status**:
   - `cargo check -p aiosh-cli` compiles cleanly with zero errors or warnings.
