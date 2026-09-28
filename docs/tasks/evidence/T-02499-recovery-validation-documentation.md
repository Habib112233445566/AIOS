# Task T-02499 Evidence: Sandbox Recovery & Validation Documentation

## Goal
Document the recovery & validation subsystem of Sandbox Enforcement for operators and autonomous agents.

## Delivered Artifacts
- Authored `docs/SPEC-SANDBOX-RECOVERY.md`:
  - 6 formal invariants (`SANDBOXRECV1`..`SANDBOXRECV6`).
  - Command-line syntax and options for `aiosh sandbox validate` and `aiosh sandbox recover`.
  - Model Context Protocol (MCP) schemas and payloads for `aios.sandbox.validate` and `aios.sandbox.recover`.
  - Comprehensive list of standard error codes.
  - Links to evidence files from T-02495 through T-02498.

## Usage Examples
```bash
# Validate sandbox health
aiosh sandbox validate

# Execute dry-run recovery check
aiosh sandbox recover --strategy dry_run --json

# Quarantine invalid manifests and restore pristine factory defaults
aiosh sandbox recover --strategy quarantine --dir /path/to/profiles
```
