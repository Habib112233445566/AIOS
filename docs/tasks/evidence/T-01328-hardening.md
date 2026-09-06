# T-01328: Init & Service Supervision - CLI Surface: Hardening

## Metadata
- **Task ID:** `T-01328`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Hardening
- **Status:** Complete

## 1. Hardening Deliverables
- **Strict Size Ceilings & Bounded Parameters**:
  - 1 MiB hard payload ceiling on `--spec` files and inline JSON input strings (`meta.len() > 1024 * 1024` or `spec_str.len() > 1024 * 1024`).
  - 1,024 character cap on `--store` paths, rejecting control characters.
  - 256 character cap on filter pattern strings (`--pattern`), rejecting control characters.
  - 128 character cap on service names, rejecting control characters.
  - Numerical limit parameter (`--limit`) constrained to $[1 \dots 10,000]$.
- **Standardized Result Envelopes (Zero Silent Failure)**:
  - In `--json` mode, all operational and syntax failures return deterministic envelopes:
    `{"code": <exit_code>, "data": null, "error": {"code": "<CODE>", "message": "<msg>"}}`
  - In human/text mode, all errors write to `eprintln!` and return non-zero exit codes (1 for operational failure, 2 for syntax/argument error).
- **Resource Cleanup & Temporary File Safety**:
  - Atomic persistence in `save_to_path` writes to unique PID-isolated files (`.tmp.<pid>`).
  - Any error during write, permission setting, or rename triggers an immediate file removal (`let _ = std::fs::remove_file(&tmp_path)`), preventing orphaned temporary files.
  - SQLite WAL database context handles are cleaned up via RAII upon function return.
- **Fail-Open Honest Audit Logging (ADR-0035 §F-2)**:
  - All command execution paths, including invalid arguments, payload overages, and FSM transition rejections, write an honest audit row to `audit.log` / SQLite WAL ring via `classify_and_emit`.

## 2. Test Verification Output
```text
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
```text
PASS: aiosh service --help
PASS: aiosh service unknown_cmd returns 2
PASS: aiosh service validate (name and json)
PASS: aiosh service list (prose, json, filters)
PASS: aiosh service show and status (prose, json, not found)
PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)
PASS: aiosh service order (valid topological plan and negative tests)

ALL SERVICE CLI SMOKE TESTS PASSED!
```
