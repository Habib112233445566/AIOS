# T-01228: Package Management - CLI Surface: Hardening

## Metadata
- **Task ID:** `T-01228`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Hardening
- **Status:** Complete

## 1. Hardening Measures Implemented
The CLI surface (`aiosh package`) has been hardened against unbounded input, control-character injection, resource exhaustion, and silent failures:

1. **Store Path Sanitization (`--store <path>`)**:
   - Evaluated at command entry before any filesystem interaction.
   - Enforces length ceiling of <= 1,024 characters and rejects any control characters (`c.is_control()`).
   - Returns structured `INVALID_ARGUMENT` (exit code 2) and records an audit failure row.

2. **Pattern & Query Sanitization**:
   - `list --pattern <pat>` and `search <pat>` enforce a 256-character length limit and reject control characters (`c.is_control()`).
   - Limits memory usage and prevents terminal control sequence injection.

3. **Limit Argument Bounds (`--limit <n>`)**:
   - In both `list` and `search`, `--limit` must parse into a strictly positive integer between 1 and 10,000.
   - Zero, negative numbers, non-numeric strings, or values exceeding 10,000 are rejected with `INVALID_ARGUMENT`.

4. **Package Name Bounds (`show <name>`)**:
   - Enforces a length ceiling of <= 64 characters and rejects control characters prior to store lookup.

5. **Payload Size Caps**:
   - Input payloads for `--actions`, `--plan`, and `--spec` (both file-based and inline strings) are strictly bounded to 1 MiB (1,048,576 bytes).
   - Rejections return `PAYLOAD_TOO_LARGE` (exit code 2) and emit structured audit rows.

6. **Standard Error Envelopes & Audit Guarantees**:
   - Every failure path emits structured JSON envelopes containing `{ "code": <exit_code>, "data": null, "error": { "code": "<ERR_CODE>", "message": "..." } }`.
   - All errors call `classify_and_emit` to guarantee complete audit trails in `audit.db` per ADR-0035 §F-2.

7. **Atomic Disk Persistence & Cleanup**:
   - `PackageStore::save_to_path` performs atomic write-rename via adjacent `.tmp` files with explicit permission settings (`0o644` on Unix) and cleans up orphaned temp files on failure.

## 2. Test Verification
Unit tests in `aiosh-cli::task_cli_tests::test_cmd_package_flow` were expanded to assert hardening boundaries:
- `aiosh package list --limit 0` -> exit code 2
- `aiosh package search curl --limit abc` -> exit code 2
- `aiosh package search "bad\0pattern"` -> exit code 2
- `aiosh package show "bad\nname"` -> exit code 2
- `aiosh package list --store "bad\0store.json"` -> exit code 2

Suite runner output:
```text
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)

PASS: package_suites criteria (PM1..PM3)
```
