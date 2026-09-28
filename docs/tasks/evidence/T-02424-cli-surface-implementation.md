# T-02424: Sandbox Enforcement CLI Surface Implementation

## 1. Implementation Overview
This document records verification evidence for task T-02424: Sandbox Enforcement CLI Surface Implementation.
The CLI subcommand router and implementation `cmd_sandbox` in `code/aiosh-rust/aiosh-cli/src/main.rs` have been fully integrated, validated, and verified with dedicated test coverage.

## 2. Implemented Capabilities
1. **Subcommand `profiles`**:
   - Lists pre-configured standard sandbox profiles (`standard`, `strict`, `permissive`).
   - Human-readable tabular output and structured JSON envelope format (`--json`).
2. **Subcommand `probe`**:
   - Queries `HostSandboxCapabilities::probe()` to detect host kernel containment features.
   - Reports Landlock LSM support, ABI version, Seccomp-BPF support, and `no_new_privs` status.
3. **Subcommand `exec`**:
   - Syntax: `aiosh sandbox exec [--profile <name>] [--grant <token>] [--cwd <path>] -- <bin> <args...>`
   - Enforces `--` delimiter separation.
   - Enforces directory traversal prevention on `--cwd` (`..` rejection).
   - Validates requested sandbox profile against service registry.
   - Dispatches execution to `SandboxService::execute` with full boundary checks, output capture limits, and audit ring emission.
   - Returns exit code matching the child process outcome or standard error codes (1 for auth, 2 for validation, 127 for missing binary).
4. **Subcommand `--help` / `-h`**:
   - Comprehensive help usage text.
5. **Audit Invariant**:
   - All invocations record through `classify_and_emit` to the SQLite WAL `audit_ring` table with classification and security context.

## 3. Test Verification
The test module `sandbox_cli_tests` in `aiosh-cli` passes all test cases:
- `test_sandbox_cli_help_and_subcommands`: Validates help screens and unknown subcommand handling.
- `test_sandbox_cli_profiles_and_probe`: Validates profile listing and capability probing in text and JSON modes.
- `test_sandbox_cli_exec_validation`: Validates rejection of missing delimiters, missing binaries, traversal in cwd, and unknown profiles.
- `test_sandbox_cli_exec_success`: Validates successful command execution in text and JSON envelope formats.

```text
running 4 tests
test sandbox_cli_tests::test_sandbox_cli_profiles_and_probe ... ok
test sandbox_cli_tests::test_sandbox_cli_help_and_subcommands ... ok
test sandbox_cli_tests::test_sandbox_cli_exec_validation ... ok
test sandbox_cli_tests::test_sandbox_cli_exec_success ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; finished in 0.74s
```
