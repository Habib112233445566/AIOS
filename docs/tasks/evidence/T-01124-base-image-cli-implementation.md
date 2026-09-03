# T-01124 — Base Image Build / CLI Surface: Implementation

**Date:** 2026-09-03
**Type:** Implementation
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Implementation Deliverables
- Implemented `cmd_image` subcommands in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh image list [--json] [--store <path>]`: Lists all registered images.
  - `aiosh image show <id> [--json] [--store <path>]`: Displays deep specifications.
  - `aiosh image plan <id> [--json] [--store <path>]`: Synthesizes and prints 4-stage build plan.
  - `aiosh image filter [--format <fmt>] [--distro <id>] [--json] [--store <path>]`: Filters images by format/distro.
- Integrated structured audit emission via `classify_and_emit` for each command.
- Verified live command execution and formatting across human-readable and JSON modes.
