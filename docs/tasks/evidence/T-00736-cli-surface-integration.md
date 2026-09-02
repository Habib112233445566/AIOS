# T-00736 — Secrets & Access Hygiene / CLI surface: Integration

## 1. Integration Deliverables
- Fully wired `aiosh secrets <scan|check>` into the CLI command dispatcher in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Confirmed integration across real file system targets:
  - Clean target: `aiosh secrets scan --file code/aiosh-rust/Cargo.toml` returns `0` (CLEAN).
  - Check target: `aiosh secrets check` returns `1` when secrets/tokens are detected across the workspace.
- Emitted audit row on every invocation with honest outcome recording (`secrets.scan` / `secrets.check`).
- Discoverable in top-level `aiosh --help`.

## 2. Execution Log
```text
=== Secrets & Access Hygiene Scan: code/aiosh-rust/Cargo.toml ===
Timestamp: 2026-08-31T04:07:13.471932100+00:00
Status: CLEAN (1 files scanned, 0 findings: 0 critical, 0 high, 0 medium, 0 low)
```
