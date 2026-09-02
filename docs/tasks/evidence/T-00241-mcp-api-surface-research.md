# T-00241 — Phase 0 — Release Packaging & Backup / MCP/API surface: Research

## Goal
Establish facts, constraints, and prior art for the MCP/API surface of Release Packaging & Backup.

## Facts
1. **Core Data Models Exist**: The internal primitives `generate_release` and `create_backup` are fully implemented in `aiosh_mcp/release.py`. They natively emit exactly one immutable `audit` log row based on ADR-0035 standards upon both success and failure.
2. **Missing MCP Annotations**: Despite being documented in the `README.md` as available MCP tools (`aios.release.generate` and `aios.backup.create`), they are not yet decorated with `@mcp.tool()` nor imported in `aiosh_mcp/server.py`.
3. **Prior Art (Pentest Pattern)**: For other submodules (like `aiosh_mcp/pentest.py`), the `server.py` file delegates tool registration using a `register_pentest_tools(mcp_server)` pattern instead of hardcoding endpoints.

## Assumptions
- We should follow the prior art from the pentest module by exposing a `register_release_tools(mcp)` function within `release.py`.
- The MCP tools must invoke `dispatch_mod.dispatch()` to enforce PEP (Policy Enforcement Point) scoping against `grant_id` before execution.
- We must pass `classifier_kwargs` (e.g., `grant_id`, `policy_revision`) into the core `generate_release` logic so the generated audit row truthfully reflects the PEP verdict.

## Unknowns & Decisions Needed
1. **Registration Pattern**: Should `release.py` encapsulate its `@mcp.tool()` wrappers via `register_release_tools`, or should it be kept in `server.py`?
   *Decision Required*: Proceed with `register_release_tools(mcp)` to maintain separation of concerns.
2. **PEP Validation Location**: `generate_release` already writes the row. How do we prevent it from writing a *second* row if `dispatch_mod.dispatch()` rejects it at the gate?
   *Decision Required*: Standard framework behavior for `dispatch()` writes a `refused` row and returns `ok: False`. The tool wrapper must catch this and return the refusal envelop without executing `generate_release`, ensuring exactly one row is generated either way (the refusal, or the successful/failed operation).

## Acceptance Criteria Verified
- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
