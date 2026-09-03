# T-01066 — Distro Selection & Justification / Security Policy: Integration

**Date:** 2026-09-03
**Type:** Integration
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Security Policy

## 1. CLI Integration & Verification
- Integrated `aiosh distro policy [<id>] [--json]` subcommand into `cmd_distro`.
- Emits structured audit records through `classify_and_emit`.
- Verified live command output:
```
$ aiosh distro policy
PROFILE ID                     STATUS     VIOLATIONS
------------------------------------------------------------
alpine-319-container-x86_64    REJECTED   binary compatibility score 0.65 below required floor 0.70
debian-12-minimal-x86_64       ALLOWED    -

Compliant profiles: 1/2
```

## 2. MCP Integration & Verification
- Registered `aios.distro.policy` in `aiosh-mcp` tools list.
- Implemented tool handler routing through `recorded_call` and audit ring buffer.
- Verified in unit tests and integration test runners.
