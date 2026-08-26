# T-00083 — Task Ledger Control observability: Scaffold

**Date:** 2026-08-22
**Type:** scaffold
**Depends on:** T-00082 spec

## What shipped

`Metrics` action variant wired through TaskAction (parse/as_str/
requires_grant=false/needs_task_id=false) + `metrics_snapshot(p)`
stub (todo!) in task_service.rs; execute_with routes Metrics to it.
Zero warnings; 66 core tests green; no consumer wiring yet (CLI/MCP/
python = T-00084).

## Acceptance check
- [x] Builds zero-warning; new interface exists and is reachable via
      enum dispatch; loud body asserted by construction.
