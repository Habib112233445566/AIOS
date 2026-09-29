# Evidence: T-02529 Privilege Escalation Prevention CLI Surface Documentation

- **Task**: `T-02529`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Documentation
1. Updated `docs/SPEC-PRIVILEGE-CLI.md`:
   - Documented invariants `PRIVESC_CLI1` through `PRIVESC_CLI5`.
   - Documented complete command syntax for all subcommands: `status`, `elevate`, `drop`, `revoke`, `check`, `list`.
   - Documented input validation rules, envelope responses, and exit code matrix (0 for success, 1 for domain rejection/validation, 2 for hardening boundary violations).
   - Documented hardening bounds: 1 MiB store payload cap, 128-byte actor ID cap, 256-byte grant token cap, 32 capabilities cap, path traversal prevention, and control character filtering.
2. Verified `aiosh --help` and `aiosh privilege --help` command docstrings in `code/aiosh-rust/aiosh-cli/src/main.rs`.
3. Verified test coverage and documentation synchronicity across Rust unit tests and Python integration tests.
