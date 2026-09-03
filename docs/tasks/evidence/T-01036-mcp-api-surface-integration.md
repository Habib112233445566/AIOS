# T-01036 — Distro Selection & Justification / MCP/API Surface: Integration

## 1. Production Integration
- Registered tools in `list_tools()`:
  - `aios.distro.list`
  - `aios.distro.show`
  - `aios.distro.evaluate`
  - `aios.distro.recommend`
- Bound through Policy Enforcement Point via `dispatch::recorded_call` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Validated cross-substrate parity using `code/aiosh-mcp/tests/test_distro_mcp_smoke.py`.

## 2. Integration Verification Output
```
[+] D1 distro data model integrity & validation invariants
[+] D2 distro store lifecycle, registry querying & persistence
[+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)
[+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)

PASS: distro_suites criteria (D1..D4)

PASS: aiosh-mcp tools/list includes all 4 distro tools
PASS: aiosh-mcp tools/call aios.distro.list
PASS: aiosh-mcp tools/call aios.distro.show
PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope
PASS: aiosh-mcp tools/call aios.distro.show nonexistent profile returns ok: false
PASS: aiosh-mcp tools/call aios.distro.evaluate
PASS: aiosh-mcp tools/call aios.distro.recommend

ALL DISTRO MCP SMOKE TESTS PASSED!
```
