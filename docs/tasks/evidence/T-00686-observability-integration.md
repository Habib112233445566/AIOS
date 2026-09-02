# T-00686 — Repository Health / observability: Integration

## Integration Summary
- **Call Paths**:
  - CLI: `aiosh repo health [--json]` & `aiosh repo check [--config <path>]`
  - MCP: `aios.repo.health` & `aios.repo.check`
  - Rust Core: `repo_health_service::check_repo_health(&config)`
- **Observability Data Invariants**:
  - `duration_ms` on individual checks and top-level summary report.
  - Aggregated check counts (`total_checks`, `passed_checks`, `warn_checks`, `failed_checks`, `skipped_checks`).
  - Structured UTC timestamp emission (`timestamp_utc`).
- **Cross-Substrate & Protocol Parity**:
  - Both CLI `--json` and MCP tool response return the exact same schema structure conforming to `RepoHealthReport`.
  - Non-mutating reads do not require PEP tokens and emit diagnostic status.
- **Verification**:
  - Verified via `tools/test_repo_health_suites.py` (H1..H7 PASS).
