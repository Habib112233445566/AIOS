# T-01021 — Distro Selection & Justification / CLI Surface: Research

**Date:** 2026-09-03
**Type:** Research (no new dependencies added)
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / CLI Surface

## 1. Objectives & Scope
Research the command-line surface for operators and administrative tooling to query, inspect, evaluate, and select Linux distribution profiles for AIOS. The CLI must provide:
- High-visibility human-readable tabular output for interactive shell sessions.
- Strict canonical JSON output when invoked with `--json` for automated script and pipeline consumption.
- Immutable audit emission for every CLI action via `AuditRing` and `classify_and_emit`.

## 2. Command Grammar & Flag Taxonomy
```
aiosh distro <subcommand> [flags]

Subcommands:
  list                          List all registered distro profiles
  show <id>                     Display detailed metadata for a specific profile
  evaluate [<id>]               Run multi-factor evaluation scoring for one or all profiles
  recommend                     Display the designated production reference profile

Flags:
  --json                        Format output as canonical JSON
  --store <path>                Optional override path to a custom distro_store.json file
  --help, -h                    Display subcommand usage and arguments
```

## 3. Exit Code Contract
- `0`: Success (profile displayed, list generated, evaluation scored).
- `1`: Operational error (profile ID not found, store file unreadable or malformed).
- `2`: Syntax or invocation error (missing required positional argument, unrecognized flag or subcommand).

## 4. Concrete Invocation Examples
```bash
# Example 1: Query recommended profile in JSON
aiosh distro recommend --json

# Example 2: Evaluate Debian minimal profile against AIOS criteria
aiosh distro evaluate debian-12-minimal-x86_64
```

## 5. Security & Invariants
- Zero new third-party crates or dependencies required; leverages `aiosh-core::distro` and `aiosh-core::distro_service`.
- Memory and execution bounded defensively with exit code conformance.
