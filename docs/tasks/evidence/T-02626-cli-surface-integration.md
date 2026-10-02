# Task Evidence: T-02626 — Secrets Handling CLI Surface Integration

## 1. Summary
Integrated the CLI surface for Secrets Handling (`aiosh secret` and alias `aiosh sec`) into the production runtime and verification suites:
- Updated root CLI usage help banner in `code/aiosh-rust/aiosh-cli/src/main.rs` to register and describe `aiosh secret <store|get|list|rotate|revoke>`.
- Verified binary compilation: `cargo build --bin aiosh`.
- Created end-to-end integration test suite `code/aiosh-cli/tests/test_secret_cli.py` covering:
  - CLI help flags (`--help`, `-h`, empty dispatch)
  - Unknown subcommand exit code (2)
  - Path traversal defenses on `--store` arguments (`..` rejected with exit code 2)
  - `store` subcommand with full metadata, validation, and JSON envelope output
  - `get` subcommand with masked output by default (protecting plaintext in logs/terminals)
  - `get` with `--expose` returning raw payload
  - `list` metadata queries without disclosing secret payloads
  - `rotate` payload updating version metadata
  - `revoke` transition marking secret state as revoked and denying subsequent reads (exit code 1)
  - Subcommand alias parity between `aiosh secret` and `aiosh sec`.

## 2. Test Verification
```
Running aiosh secret CLI smoke test suite...
Testing secret help commands...
Testing unknown subcommands...
Testing path traversal defense...
Testing secret full lifecycle...
All aiosh secret CLI integration tests PASSED!
```

## 3. Invariants Verified
- **SECCLI1**: `aiosh secret` and `aiosh sec` are registered, discoverable, and enforce deterministic exit codes (0 = success, 1 = domain error / unauthorized, 2 = usage / validation error).
- **SECCLI2**: Default `get` and `list` operations never expose raw secret payloads unless explicit `--expose` flag is supplied.
- **SECCLI3**: Store path traversal attempts with `..` are strictly rejected with exit code 2.
