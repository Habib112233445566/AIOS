# T-00243 — Phase 0 — Release Packaging & Backup / MCP/API surface: Scaffold

## Goal
Create the module skeleton and interfaces for the MCP/API surface of Release Packaging & Backup.

## Completion Notes
1. **Module Scaffolding (`aiosh_mcp/release.py`)**:
   - Exposed a new `register_release_tools(mcp)` hook mirroring the established `pentest_mod` pattern.
   - Scaffolded the `@mcp.tool(name="aios.release.generate")` and `@mcp.tool(name="aios.backup.create")` functions.
   - Applied precise Python type hints reflecting the Phase 0 specification (e.g., `target_path: str, include_audit: bool = True`).
   - Forced `NotImplementedError` inside both tools to ensure loud failure until implementation phase.

2. **Server Integration (`aiosh_mcp/server.py`)**:
   - Wired `register_release_tools(mcp)` into `server.py`'s registration boot phase.

3. **Build / Import Validation**:
   - Executed `python -c "import aiosh_mcp.server"`. The build correctly boots the FastMCP runtime and processes the tool signatures without warnings or crashes.

## Acceptance Criteria Verified
- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by `aiosh_mcp.server`.
