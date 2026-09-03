# T-01034 — Distro Selection & Justification / MCP/API Surface: Implementation

## 1. Implementation Summary
- Integrated tool dispatch handlers inside `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.distro.list`: Loads `DistroStore` and returns array of profile summaries.
  - `aios.distro.show`: Validates required `id` argument and returns matching profile.
  - `aios.distro.evaluate`: Computes multi-factor scores for single or all profiles.
  - `aios.distro.recommend`: Identifies recommended base OS profile for AIOS.
- Dispatched through `dispatch::recorded_call`, preserving:
  - Policy Enforcement Point (PEP) gating.
  - Evaluation of rules R-01..R-12 in `AI_CONSTITUTION.md`.
  - Immutable SHA-256 hash-chained recording in SQLite WAL `AuditRing`.
- Reused `aiosh_core::distro_service::DistroStore` without introducing any new external dependencies.

## 2. Test Verification Output
```
running 1 test
test server_tests::test_mcp_distro_tools ... ok

PASS: aiosh-mcp tools/list includes all 4 distro tools
PASS: aiosh-mcp tools/call aios.distro.list
PASS: aiosh-mcp tools/call aios.distro.show
PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope
PASS: aiosh-mcp tools/call aios.distro.evaluate
PASS: aiosh-mcp tools/call aios.distro.recommend

ALL DISTRO MCP SMOKE TESTS PASSED!
```
