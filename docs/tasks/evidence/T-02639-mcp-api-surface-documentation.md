# Task Evidence: T-02639 — Secrets Handling MCP/API Surface Documentation

## 1. Summary
Documented the Model Context Protocol (MCP) tool surface for Secrets Handling in [SPEC-SECRETS-MCP.md](file:///docs/SPEC-SECRETS-MCP.md):
- Provided comprehensive JSON-RPC 2.0 request and response schemas and copy-pasteable examples for all 5 tools (`aios.secret.store`, `aios.secret.get`, `aios.secret.list`, `aios.secret.rotate`, `aios.secret.revoke`).
- Documented masking behavior: masked vs plaintext `expose: true` JSON output representations.
- Documented system constraints and limitations honestly (64 KiB payload limit, 1,024 vault capacity, 1 MiB store file limit, Phase 2 local store vs Phase 3 TPM 2.0 binding).
- Linked all task evidence files from `T-02631` through `T-02639`.

## 2. Verification
All schemas and example payloads match the behavior tested in `code/aiosh-mcp/tests/test_secret_mcp.py` and `code/aiosh-rust/aiosh-mcp/src/main.rs`.
