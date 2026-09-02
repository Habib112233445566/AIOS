# T-00538 — Evidence & Audit Trail / CLI surface: Hardening

## 1. Hardening Scope
This task hardens the `aiosh evidence` command-line surface against invalid arguments, missing directories, malformed manifest files, and unhandled errors.

## 2. Hardening Measures
1. **Usage & Syntax Guardrails**:
   - Subcommand dispatch returns exit code 2 on missing or unknown arguments with explicit usage help strings.
2. **Directory & File Validation**:
   - `aiosh evidence scan` verifies directory existence before reading entries, returning exit code 1 with structured error messages if absent.
   - `aiosh evidence hash` validates target file existence and enforces 16 MiB maximum file bounds.
3. **Structured Audit Emission on Error Paths**:
   - All failure states (`outcome: "error"`) write an honest audit entry to SQLite WAL with the exact error message.

## 3. Test Verification
- `python code/aiosh-cli/tests/test_evidence_cli_smoke.py` -> 8/8 tests pass (including missing files, missing args, unknown subcommands, and task filters).
