# T-01026 — Distro Selection & Justification / CLI Surface: Integration

## 1. Production Integration
- **CLI Call Path**: Integrated `aiosh distro` with subcommands (`list`, `show`, `evaluate`, `recommend`) into the primary Rust CLI binary (`code/aiosh-rust/aiosh-cli/src/main.rs`).
- **Audit Logging Parity**: Integrated with `AuditRing` and `classify_and_emit` guaranteeing consequential operations write tamper-evident rows into SQLite WAL storage.
- **Cross-Substrate Parity**: Built and executed `code/aiosh-cli/tests/test_distro_cli_smoke.py`, proving full JSON and text parsing parity with external Python and automation callers.

## 2. Integration Verification Output
```
[+] D1 distro data model integrity & validation invariants
[+] D2 distro store lifecycle, registry querying & persistence
[+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)
[+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)

PASS: distro_suites criteria (D1..D4)

PASS: aiosh distro list prose
PASS: aiosh distro list --json
PASS: aiosh distro show prose
PASS: aiosh distro show --json
PASS: aiosh distro evaluate --json
PASS: aiosh distro evaluate <id> --json
PASS: aiosh distro recommend --json
PASS: aiosh distro --help
PASS: aiosh distro show missing id returns 2
PASS: aiosh distro show nonexistent returns 1

ALL DISTRO CLI SMOKE TESTS PASSED!
```
