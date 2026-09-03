# T-01033 — Distro Selection & Justification / MCP/API Surface: Scaffold

## 1. Scaffold Deliverables
- Registered tools in `list_tools()` under `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.distro.list`
  - `aios.distro.show`
  - `aios.distro.evaluate`
  - `aios.distro.recommend`
- Created dedicated integration and smoke suite `code/aiosh-mcp/tests/test_distro_mcp_smoke.py`.
- Verified build: `cargo build --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh-mcp`.

## 2. Compilation and Test Output
```
PASS: aiosh-mcp tools/list includes all 4 distro tools
PASS: aiosh-mcp tools/call aios.distro.list
PASS: aiosh-mcp tools/call aios.distro.show
PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope
PASS: aiosh-mcp tools/call aios.distro.evaluate
PASS: aiosh-mcp tools/call aios.distro.recommend

ALL DISTRO MCP SMOKE TESTS PASSED!
```
