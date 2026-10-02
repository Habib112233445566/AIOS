# Task Evidence: T-02629 — Secrets Handling CLI Surface Documentation

## 1. Summary
Documented the command-line interface for the Secrets Handling subsystem (`aiosh secret` and `aiosh sec`):
- Updated [SPEC-SECRETS-CLI.md](file:///docs/SPEC-SECRETS-CLI.md) with comprehensive command specifications, subcommands (`store`, `get`, `list`, `rotate`, `revoke`), full argument descriptions, and exit codes table.
- Added copy-pasteable examples for each subcommand including masked output demonstrations and `--expose` requirements.
- Documented system constraints honestly: 64 KiB individual payload ceiling, 1,024 vault entry capacity, 1 MiB store file size cap, scope hierarchy enforcement, and Phase 2 local store vs Phase 3 TPM/HSM binding.
- Linked all task evidence files from `T-02621` through `T-02629`.

## 2. Verification
All examples in `SPEC-SECRETS-CLI.md` match the output produced by `code/aiosh-cli/tests/test_secret_cli.py` and rust tests in `aiosh-cli`.
