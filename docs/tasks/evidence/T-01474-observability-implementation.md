# Task Evidence: T-01474 - Session Observability Implementation

## Summary
Implements the comprehensive observability, telemetry metrics, and state breakdown suite across core engine, CLI, and MCP surfaces for the User Session Bootstrap Subsystem (`SSO1..SSO6`).

## Changes Delivered
1. **Core Integration Test Suite**:
   - `code/aiosh-rust/aiosh-core/tests/test_session_observability.rs`
   - Test coverage:
     - `test_sso1_state_and_class_distribution`: Validates zero-initialized breakdown and exact categorization across session lifecycle states and functional classes.
     - `test_sso2_seat_and_scope_arbitration`: Validates multi-seat assignment distribution and foreground/background focus tracking.
     - `test_sso3_idle_time_tracking`: Validates locked session counts, idle session enumeration, aggregate idle time, and peak idle duration.
     - `test_sso4_user_concurrency_breakdown`: Validates distinct usernames and per-user session concurrency tallies.
     - `test_sso5_policy_compliance_evaluation`: Validates integration with `UserSessionSecurityPolicy` returning compliant vs. violating session counts and IDs.
     - `test_sso6_canonical_serialization`: Validates round-trip deterministic serialization to canonical JSON.

2. **CLI Surface (`aiosh session stats`)**:
   - `code/aiosh-rust/aiosh-cli/src/main.rs`:
     - Added `aiosh session stats [--policy <path>] [--store <path>] [--json]` subcommand.
     - Bound to `aiosh_core::session_observability::SessionObservabilityReport::generate`.
     - Structured audit event emission to SQLite WAL audit ring via `classify_and_emit` (`session`, `stats`).
     - Supports `--json` machine-readable output and tabular human-readable summary.

3. **MCP Surface (`aios.session.stats`)**:
   - `code/aiosh-rust/aiosh-mcp/src/main.rs`:
     - Registered `aios.session.stats` in `list_tools()` with schemas for `policy_path`, `store_path`, and `grant_id`.
     - Handled tool dispatch in `call_tool()` through PEP authorization and audit ring recording.
     - Added in-tree unit test verification in `test_server_tool_listing_and_execution`.
