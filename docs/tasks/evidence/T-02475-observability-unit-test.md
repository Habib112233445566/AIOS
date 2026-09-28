# T-02475: Sandbox Enforcement Observability Unit Test

## 1. Unit Test Scope
This task creates and executes unit tests for the Sandbox Enforcement Observability subsystem in `code/aiosh-rust/aiosh-core/tests/test_sandbox_observability.rs`.

---

## 2. Test Cases Covered

| Test Name | Focus | Result |
|---|---|---|
| `test_observability_default_generation` | Validates default report generation on a clean service (3 factory profiles, 0 executions, healthy state). | **PASSED** |
| `test_observability_text_sanitization` | Asserts control characters/ANSI sequences are stripped and long strings are clamped to `MAX_TELEMETRY_TEXT_LEN` (256). | **PASSED** |
| `test_observability_with_executions_and_outcomes` | Executes commands through `SandboxService` and verifies execution tallies by outcome (`ok`, `error`) and by profile. | **PASSED** |
| `test_observability_validation_bounds` | Validates report validation against empty timestamps and invalid constraints. | **PASSED** |

---

## 3. Execution Results
```text
running 4 tests
test test_observability_text_sanitization ... ok
test test_observability_validation_bounds ... ok
test test_observability_default_generation ... ok
test test_observability_with_executions_and_outcomes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
```
