# T-00246 — MCP/API surface: Integration
MCP tools `aios.release.generate` and `aios.backup.create` are wired into `server.py` via `register_release_tools(mcp)`. Cross-substrate parity confirmed: both CLI (Rust) and MCP (Python) use identical data models and audit schemas. Integration smoke passes: `test_release_smoke.py` (3/3) + `test_release_mcp_smoke.py` (6/6).
