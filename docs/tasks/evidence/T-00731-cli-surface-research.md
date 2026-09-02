# T-00731 — Secrets & Access Hygiene / CLI surface: Research

## 1. Prior Art & In-Tree Architecture
- **In-Tree CLI Precedents**: `code/aiosh-rust/aiosh-cli/src/main.rs` dispatches top-level commands through single-pass argument parsers (`parse_flag`, `has_flag`, `strip_flags`) and dispatches to typed command handlers (`cmd_repo`, `cmd_evidence`, `cmd_doc`, `cmd_toolchain`, `cmd_task`).
- **Core Ingestion Layer (`aiosh-core::secrets_service`)**:
  - `scan_file_for_secrets(&Path, &Path, u64) -> Result<Vec<SecretFinding>, String>`
  - `scan_workspace_for_secrets(&Path, u64, &[&str]) -> Result<SecretScanReport, String>`
  - Redaction: `redact_secret_value` safely preserves 4 prefix / 4 suffix chars with `****` masking for strings $\ge 12$ chars.
- **Audit & Governance Invariants**:
  - Every CLI subcommand invocation emits exactly one audit row into SQLite WAL (`AuditRing::write`).
  - Read-only diagnostics require no PEP grants but are recorded with immutable integrity hashes.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Command Grammar | Fact | `aiosh secrets <scan|check>` aligned with `repo`, `doc`, and `evidence` grammar. |
| Flag Support | Fact | Supports `--repo <path>`, `--file <path>`, `--max-bytes <n>`, and `--json`. |
| Exit Codes | Fact | Exit `0` for clean scans; exit `1` if secrets are detected in check mode or on error; exit `2` on usage error. |
| JSON Schema | Fact | `--json` outputs `{ "ok": bool, "subcommand": "secrets <subcommand>", "data": SecretScanReport }`. |
| Zero Leaks | Fact | All finding outputs display ONLY the `redacted_snippet` and cryptographic `fingerprint`. |

## 3. Decisions & Contracts Needed
1. **Subcommand Surface**:
   - `aiosh secrets scan`: Full report output with details per finding.
   - `aiosh secrets check`: Boolean gating check returning non-zero if critical/high secrets exist.
2. **Subcommand Dispatch**: Added to `code/aiosh-rust/aiosh-cli/src/main.rs` under `Some("secrets") => cmd_secrets(&args[1..])`.
