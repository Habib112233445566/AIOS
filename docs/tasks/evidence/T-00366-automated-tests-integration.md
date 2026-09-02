# T-00366 — Dependency & Toolchain Pinning / automated tests: Integration

## 1. Integration Scope
This task wires the new Dependency & Toolchain Pinning automated smoke test suites into the centralized CI test orchestrator (`tools/ci_suites.py`) and verifies end-to-end execution.

## 2. Integrated Components
1. **Registry Addition (`tools/ci_suites.py`)**:
   - `toolchain_cli_smoke`: Executes `code/aiosh-cli/tests/test_toolchain_cli_smoke.py`
   - `toolchain_mcp_smoke`: Executes `code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py`
   - Total suites in CI registry updated from 20 to 22 maintaining canonical deterministic order.
2. **Registry Test Suite Updates (`tools/test_ci_suites.py`)**:
   - Canonical order array and suite count bounds updated to include `toolchain_cli_smoke` and `toolchain_mcp_smoke`.
3. **CI Service Verification (`tools/test_ci_service.py`)**:
   - Dynamic synchronization with `ci_suites.SUITE_NAMES` ensuring valid verification across synthetic run results.

## 3. Verification & Execution Evidence
All CI unit suites executed and verified:
- `python tools/test_ci_config.py` -> PASS
- `python tools/test_ci_run.py` -> PASS
- `python tools/test_ci_suites.py` -> PASS (W1..W7)
- `python tools/test_ci_service.py` -> PASS (X1..X7)
- `python code/aiosh-cli/tests/test_toolchain_cli_smoke.py` -> PASS
- `python code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py` -> PASS
