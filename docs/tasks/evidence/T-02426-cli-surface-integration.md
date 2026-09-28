# T-02426: Sandbox Enforcement CLI Surface Integration

## 1. Integration Scope & Verification
This document records verification evidence for task T-02426: Sandbox Enforcement CLI Surface Integration.
The sandbox enforcement CLI surface (`aiosh sandbox` / `aiosh sb`) was integrated into the operational runtime and verified against real execution paths, output streams, and audit persistence.

## 2. Integration Pathways
1. **CLI Router Integration**:
   - Wired in `code/aiosh-rust/aiosh-cli/src/main.rs`:
     `Some("sandbox") | Some("sb") => cmd_sandbox(&args[1..]),`
   - Added to the master CLI interactive help synopsis.
2. **Subcommand Dispatch**:
   - `profiles`: Interacts with `SandboxService` catalog of standard containment profiles.
   - `probe`: Queries host OS containment capability status via `HostSandboxCapabilities::probe()`.
   - `exec`: Enforces `--` delimiter separation, validates profile specifications, strips directory traversal (`..`) in working directory paths, runs commands under supervision, and passes stdout/stderr streams cleanly.
3. **Terminal Output Sanitization**:
   - Child process standard output and standard error streams are filtered through `sanitize_terminal_output`, preventing ANSI escape and terminal injection attacks while preserving legitimate whitespace formatting (`\n`, `\r`, `\t`).
4. **Audit Trail Parity**:
   - Every execution writes an audit event through `classify_and_emit` to the SQLite WAL `audit_ring` table with tool identifier `sandbox`, execution duration, and outcome status.

## 3. Automated Smoke & Integration Verification
The end-to-end integration was validated using `scripts/test_sandbox_integration_smoke.py` and `code/aiosh-cli/tests/test_sandbox_cli.py`:
- Exit codes for valid executions return child status (`0`).
- Exit codes for invalid options return code `2`.
- JSON envelope output (`--json`) is canonical and parseable by external automation.
- All 6 smoke and integration tests pass cleanly (100% pass rate).
