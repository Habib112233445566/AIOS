# T-00368 — Dependency & Toolchain Pinning / automated tests: Hardening

## 1. Hardening Overview
This task hardens the Dependency & Toolchain Pinning automated test suites against execution hangs, child process leakage, unhandled exceptions, and unclosed temp file descriptors.

## 2. Hardening Measures Implemented

### A. Subprocess Execution Timeouts
- `code/aiosh-cli/tests/test_toolchain_cli_smoke.py`: Added explicit 30s timeout to `subprocess.run(cmd, ..., timeout=30)` to guard against CLI command hangs.
- `code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py`: Added `timeout=30` to `p.communicate(..., timeout=timeout_s)` with an explicit `subprocess.TimeoutExpired` handler that invokes `p.kill()` and `p.wait()` to reap zombie processes before failing loudly.

### B. Standardized Result Envelopes
- Verified that all negative tests (e.g. invalid subcommands, missing configuration files, malformed JSON, and toolchain version mismatches) assert standard JSON error envelopes with explicit failure descriptions rather than raw crashes or silent failures.

### C. Temporary File & Resource Durability
- Ensured all temporary files created for testing custom, malformed, or mismatched manifests use `try ... finally` blocks to ensure prompt file unlinking and eliminate disk leakage across test runs.

## 3. Verification
Executed both test suites in isolation:
- `code/aiosh-cli/tests/test_toolchain_cli_smoke.py` -> PASS (7 assertions + suite finish)
- `code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py` -> PASS (3 assertions + suite finish)
