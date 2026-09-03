# T-01122 — Base Image Build / CLI Surface: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. CLI Command Specification
```bash
aiosh image list [--json] [--store <path>]
aiosh image show <id> [--json] [--store <path>]
aiosh image plan <id> [--json] [--store <path>]
aiosh image filter [--format <raw|qcow2|iso|tarball>] [--distro <id>] [--json] [--store <path>]
```

## 2. Invariants & Exit Codes
- `C1 (Success)`: Exit code `0` on successful execution and display.
- `C2 (Not Found)`: Exit code `1` when `<id>` does not exist in registry.
- `C3 (Usage Error)`: Exit code `2` on unknown subcommand or missing `<id>`.
- `C4 (Telemetry)`: Every invocation records a structured event into the audit ring buffer via `classify_and_emit`.
