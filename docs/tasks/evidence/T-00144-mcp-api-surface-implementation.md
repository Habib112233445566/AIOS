# T-00144 — CI Smoke Orchestration / MCP/API surface: Implementation

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration MCP/API surface

## 1. Minimal Working Behavior
- Replaced the scaffolded `call_ci` stub with the full implementation in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- The MCP `aios.ci` tool now extracts the `action` ("check", "show", or "failures") and optional `file` parameters.
- It dynamically falls back to the `AIOSH_CI_RESULTS` environment variable or the `/tmp/aiosh-ci-results.json` default if no file is provided.
- The `aiosh_core::ci::load_summary_with_retry` function is invoked to enforce validation and read constraints.

## 2. Audit/PEP Invariants (ADR-0035 §F-2)
- Read-only observational operations (`show` and `failures`) format and return the data directly over JSON RPC, emitting no audit row.
- The consequential `check` action invokes `dispatch::dispatch` and `dispatch::commit` against the system's `AuditRing` and `PepStore`, logging the file path and outcome (`success` or `failure`), identical to the `aiosh ci` CLI.
- Load or parse errors explicitly commit an `error` audit row to permanently record the failure.
