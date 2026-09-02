# T-00261 — Automated Tests: Research

## Facts
1. **Existing Test Substrates**: Tests for Release Packaging & Backup span three layers:
   - **Core Logic (Rust)**: `code/aiosh-rust/aiosh-core/src/release.rs` contains unit tests for `generate_release` and `create_backup`, verifying the execution context, manifest parsing, and audit ring row emission.
   - **Physical Implementation (Python)**: `code/aiosh-mcp/tests/test_release_physical.py` validates the Python-side zip packaging and ISO mocking behavior, including path exclusion rules (e.g., omitting the `audit/` dir).
   - **CLI Smoke (Python wrapper)**: `code/aiosh-cli/tests/test_release_cli_smoke.py` invokes the Rust CLI binary using `cargo run`.
2. **Windows Skip Rule**: The CLI smoke tests are explicitly marked with `@pytest.mark.skipif(os.name == 'nt')` because the `aiosh-cli` binary relies on `aiosh-sandbox` which pulls in Linux-specific dependencies (`libc`, `bpf`) that intentionally break on Windows.
3. **Configuration Coverage Gap**: `release_config.rs` (hardened in `T-00258`) does not currently have dedicated unit tests for its OOM protection (64KB read limit) or path traversal validation logic.

## Assumptions
- We assume it is architecturally acceptable that `aiosh-cli` smoke tests skip on Windows, as the target OS for the Linux ethical-hacking pillar is inherently Linux.
- We assume `genisoimage` or a comparable binary will exist on the production deployment; all tests currently mock this via `b"AIOS_ISO_MOCK"`.

## Decisions Needed
- Do we need to backfill unit tests for `release_config.rs` in the upcoming implementation tasks? *(Yes, to ensure the 64KB cap and `output_dir` validations are mechanically enforced).*
- Do we need to extend the Python MCP tests, or focus only on the Rust shipping substrate for new tests? *(Focus on Rust, as per v2.1 course correction).*
