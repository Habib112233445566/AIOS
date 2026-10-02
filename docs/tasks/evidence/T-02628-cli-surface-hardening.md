# Task Evidence: T-02628 — Secrets Handling CLI Surface Hardening

## 1. Summary
Hardened the CLI interface (`aiosh secret` / `aiosh sec`) against resource exhaustion, malformed payloads, oversized file injection, and terminal poisoning:
1. **Store File Limit & Symlink Rejection**:
   - Implemented strict 1 MiB (`1,048,576` bytes) ceiling check on target store files via `symlink_metadata`.
   - Any attempt to target an oversized store file records an explicit audit failure event (`classify_and_emit("secret", "store", ...)` with `failure` status) and returns exit code 1.
2. **Payload Size Bounds**:
   - Enforced `MAX_SECRET_PAYLOAD_SIZE` (64 KiB) limit on all CLI value flags to prevent memory bloat and DOS.
3. **Structured Non-Silent Error Propagation**:
   - All error branches output JSON error envelopes (`{ "code": <exit_code>, "data": null, "error": { "code": ..., "message": ... } }`) under `--json`.
   - Unformatted terminal errors pass through `sanitize_terminal` to strip escape sequences.
4. **Leak-Free Persistence Cleanup**:
   - Atomic store updates write to `.tmp.{pid}.{nanos}` tempfiles with explicit removal handlers on error, leaving zero dangling files.

## 2. Test Verification
- `cargo test -p aiosh-cli secret_cli_tests`: 3 passed, 0 failed.
- `python code/aiosh-cli/tests/test_secret_cli.py`: All 10 verification scenarios passed.
- `cargo check --workspace`: Clean with 0 errors, 0 warnings.
