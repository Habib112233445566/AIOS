# T-01219: Package Management - Core Service: Documentation

## Metadata
- **Task ID:** `T-01219`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Documentation
- **Status:** Complete

## 1. Documentation Deliverables
- Updated root specification in `docs/README.md` (Section 8.12) with full architectural description of `PackageStore`, core invariants `CS1..CS5`, operator CLI surfaces, autonomous agent MCP tools, and security hardening rules.
- Added copy-pasteable command invocations for:
  - `aiosh package validate --name "curl"`
  - `aiosh package list --format deb --state installed`
  - `aiosh package show curl`
  - `aiosh package plan --actions '[{"action":"install","package_name":"libssl3"},{"action":"install","package_name":"curl"}]' --dry-run --json`
- Documented system constraints and known limitations:
  - In-memory package store capacity bounded to 10,000 packages.
  - Multi-package dependency graphs must be declared in a single transaction batch (automatic transitive network fetching deferred to later Phase 1 tasks).
  - Transactions modify in-memory store states; physical filesystem unpack and native package manager integration will be driven by wrapper tasks.
- Linked full evidence chain from `tasks/evidence/T-01211-core-service-research.md` through `tasks/evidence/T-01220-core-service-verification-evidenc.md`.
- Verified structural consistency against `tools/check_task_docs.py` (C1..C6 PASS).
