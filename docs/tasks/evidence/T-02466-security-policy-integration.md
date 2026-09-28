# T-02466: Sandbox Enforcement Security Policy Integration

## 1. Integration Scope
This task integrates `SandboxSecurityPolicy` into both the CLI surface (`aiosh sandbox policy`) and the MCP tool surface (`aios.sandbox.policy`), verifying cross-substrate parity, JSON envelope schema conformance, and audit logging.

---

## 2. Surfaces Wired
1. **CLI Surface (`aiosh-cli`)**:
   - Implemented `aiosh sandbox policy [--path <PATH>] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Supports structured JSON envelope (`{"code": 0, "data": ..., "error": null}`) and human-readable terminal output.
   - Emits classification event: `classify_and_emit(&mut ctx, "sandbox", "policy", ...)`.
2. **MCP Tool Surface (`aiosh-mcp`)**:
   - Registered tool `aios.sandbox.policy` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Gated through `dispatch::recorded_call`, enforcing policy evaluation and writing a SHA-256 hash-chained row to the SQLite WAL audit ring.
3. **Core Engine Integration (`aiosh-core`)**:
   - `SandboxService` evaluates security policy on every `execute` call, denying prohibited commands/variables before spawning child processes.

---

## 3. Verification Results
- `aiosh sandbox policy --json`: Code 0, valid JSON envelope returned.
- `code/aiosh-mcp/tests/test_sandbox_mcp.py`: 10/10 passed in 2.54s.
