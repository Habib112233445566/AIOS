# T-00369 — Dependency & Toolchain Pinning / automated tests: Documentation

## 1. Documentation Scope
This task updates user, operator, and agent documentation regarding the automated test suites for the Dependency & Toolchain Pinning epic.

## 2. Documentation Updates
- **Document Updated**: `docs/README.md`
- **Section**: `## Dependency & Toolchain Pinning (T-00311..T-00370)`

### Added Content:
1. **CLI Commands Reference**:
   - `aiosh toolchain show`
   - `aiosh toolchain check`
   - `aiosh toolchain check --config <path>`
2. **MCP Tool Invocations**:
   - `aios.toolchain.config.get`
   - `aios.toolchain.check`
3. **Automated Tests Invocation**:
   ```bash
   # CLI test suite
   python3 code/aiosh-cli/tests/test_toolchain_cli_smoke.py

   # MCP test suite
   python3 code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py
   ```
4. **Honest Known Limitations**:
   - Subprocess version probing (15-second wall-clock bounds to account for cold rustup shims).
   - 64KB size caps on JSON configurations.
   - Optional Node.js verification for minimal environments.
5. **Evidence Chain**:
   - Linked research, specification, implementation, unit test, integration, security, and hardening evidence files.

## 3. Verification
`python tools/check_task_docs.py` executed with exit code 0, confirming C1..C6 documentation invariants are satisfied.
