# T-01464: User Session Bootstrap — Security Policy: Implementation

## Metadata
- **Task ID:** `T-01464`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Implementation Deliverables

1. **Core Policy Module (`code/aiosh-rust/aiosh-core/src/session_policy.rs`)**:
   - Codified `UserSessionSecurityPolicy`, `SessionPolicyMode`, `SessionPolicyViolation`, and `SessionPolicyVerdict`.
   - Enforced criteria `SSP1..SSP7`:
     - **SSP1**: UID 0 (root) restriction by default, greeter system user boundaries.
     - **SSP2**: Session type allowlist gating, Agent class requiring `AiAgent` type, Greeter class requiring explicit display and no remote host.
     - **SSP3**: Remote sessions forbidden from attaching to console `seat0`, bounded VT numbers $[1 \dots 64]$, display string sanitization.
     - **SSP4**: Stripping and rejection of dangerous dynamic linker and script variables (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`, `NODE_OPTIONS`, `PYTHONPATH`, `RUBYOPT`, `PERL5OPT`), bounded environment entries ($\le 256$).
     - **SSP5**: Per-user concurrency limits ($\le 32$) and global store capacities ($\le 1,024$).
     - **SSP6**: AI Agent sandboxing, disallowing root UID execution.
     - **SSP7**: Three operational modes (`Enforcing`, `Audit`, `Permissive`), bounded file loading ($\le 64\text{ KiB}$).

2. **Integration Test Suite (`code/aiosh-rust/aiosh-core/tests/test_session_policy.rs`)**:
   - Implemented 8 standalone unit/integration tests verifying all `SSP1..SSP7` criteria.
   - All tests passing in 0.00s.

3. **CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added subcommand `aiosh session policy [--policy <path>] [--spec <file_or_json>] [--store <path>] [--json]`.
   - Connected with SQLite WAL audit logger (`classify_and_emit`).
   - Verified output: `{"code":0,"data":{"allowed":true,"mode":"enforcing",...}}`.

4. **MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.session.policy` in `list_tools`.
   - Dispatched in `call_tool` with PEP validation and SQLite WAL audit recording.
   - Added automated tests in `test_mcp_session_validate_tools`.

## 2. Verification
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_session_policy`: 8 passed; 0 failed.
- `cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_session_validate_tools`: 1 passed (including all 20 tool assertions).
- `aiosh session policy --json`: returned exit code 0.
