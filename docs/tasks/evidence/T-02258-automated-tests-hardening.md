# T-02258 Evidence: Automated Tests — Hardening

**Task:** Harden the automated tests of Grant Lifecycle against failure and misuse.  
**Status:** COMPLETE  
**Date:** 2026-09-22  

## Hardening Measures Applied

### 1. Process Safety (MCP Smoke Test)
- **Timeout enforcement**: All `run_mcp()` calls enforce a 30-second timeout with `subprocess.TimeoutExpired` handling
- **Zombie process prevention**: `finally` block ensures `p.kill()` + `p.wait()` on every code path
- **Exit code validation**: Non-zero return codes from `aiosh-mcp` binary are detected and fail fast with diagnostic output
- **JSON parse safety**: Malformed stdout is caught with try/except and reports raw output for debugging

### 2. Test Isolation (Rust + Python)
- **Hermetic temp directories**: Both `tempfile.TemporaryDirectory()` (Python) and `tempdir()` (Rust) create isolated environments per test run
- **No shared mutable state**: Each test function creates its own grant store; no cross-test pollution
- **Deterministic ordering**: Tests run in fixed sequence with each step depending on verifiable prior state

### 3. Assertion Robustness
- **Defensive field access**: All assertions use `.get()` with fallback values to prevent `KeyError` on unexpected responses
- **Rich failure diagnostics**: Every assertion includes the full response payload in its error message (e.g., `f"Failed root issue: {res_root}"`)
- **Boundary guards tested**: The test suite explicitly covers:
  - Duplicate grant ID rejection
  - Non-existent grant inspection error handling
  - Right expansion rejection (monotonicity)
  - Subject mismatch rejection
  - Expired grant state inspection after sweep

### 4. Rust Test Hardening (`test_pep_grant_automated.rs`)
- **AUTOGRANT1**: Scale test with 1000 grants validates no panic or memory corruption at load
- **AUTOGRANT6**: Adversarial fuzzing with invalid identifiers (control chars `\x00`, empty string, 200-char overlong) — all rejected without panic
- **AUTOGRANT7**: Thread safety test with `Arc<RwLock<>>` validates no data races under concurrent access
- **AUTOGRANT5**: Persistence reload validates no data loss across save/load cycles

### 5. Failure Mode Coverage
| Failure Mode | Test Coverage | Hardening Status |
|---|---|---|
| Binary not found | `_find_binary()` fallback chain | ✅ Hardened |
| Subprocess timeout | `TimeoutExpired` handler | ✅ Hardened |
| Invalid JSON response | try/except with raw output | ✅ Hardened |
| Zombie process leak | `finally` cleanup block | ✅ Hardened |
| Cross-test state leak | Temp directory isolation | ✅ Hardened |
| Scale exhaustion | 1000-grant stress test | ✅ Hardened |
| Concurrent corruption | Thread safety test | ✅ Hardened |

## Conclusion
All automated tests are hardened against failure modes, resource leaks, and misuse scenarios. No additional hardening actions required.
