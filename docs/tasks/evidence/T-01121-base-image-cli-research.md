# T-01121 — Base Image Build / CLI Surface: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. CLI Commands & Usability Research
- **Command Group**: `aiosh image`
- **Subcommands**:
  - `aiosh image list [--json] [--store <path>]`: Enumerates all registered base image manifests.
  - `aiosh image show <id> [--json] [--store <path>]`: Displays deep specifications (rootfs, kernel, format, packages, sizing).
  - `aiosh image plan <id> [--json] [--store <path>]`: Synthesizes and prints the 4-stage build plan with estimated duration and command templates.
  - `aiosh image filter [--format <format>] [--distro <id>] [--json] [--store <path>]`: Queries registry by target format or distro identifier.
- **Telemetry & Audit Emission**:
  - Emits telemetry events via `classify_and_emit` into audit log buffer.
- **Fail-Closed Diagnostics**:
  - Missing subcommands or invalid arguments exit with code 2.
  - Nonexistent image lookups exit with code 1 and descriptive stderr messages.
