# T-01096 — Distro Selection & Justification / Recovery & Validation: Integration

**Date:** 2026-09-03
**Type:** Integration
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Recovery & Validation

## 1. CLI Integration & Verification
- Added `aiosh distro check [--json] [--store <path>]` subcommand.
- Emits structured audit telemetry record via `classify_and_emit`.
- Reports status `HEALTHY` (exit 0) or `UNHEALTHY` with failure list (exit 1).

## 2. MCP Integration & Verification
- Registered `aios.distro.check` tool in `tools_list`.
- Added handler in `call_tool` dispatching through `recorded_call`.
- Verified in `tests::test_mcp_distro_tools`.

## 3. Automated Test Suite Integration
- Added criterion `D6` to `tools/test_distro_suites.py`.
- Verified live runner execution:
```
[+] D1 distro data model integrity & validation invariants
[+] D2 distro store lifecycle, registry querying & persistence
[+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)
[+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)
[+] D5 distro configuration resolution & hardening invariants
[+] D6 distro store corruption recovery & health check validation invariants

PASS: distro_suites criteria (D1..D6)
```
