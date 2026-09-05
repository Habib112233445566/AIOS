# T-01282: Package Management Documentation Specification

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Documentation  
**Task ID:** T-01282  

---

## 1. Specification Overview
This document formally specifies the structural layout, interface contracts, error envelopes, audit effects, and validation constraints for the comprehensive Package Management documentation artifact: `docs/package_management.md`, alongside its automated verification runner `tools/test_package_doc.py`.

---

## 2. Document Structure & Content Contracts

The generated guide `docs/package_management.md` must strictly adhere to the following 9-section architecture:

### Section 1: Executive Overview & Architectural Role
- Context: Role within AIOS Phase 1 (Linux Base System & Bootable Target).
- Responsibilities: Unified, secure package management layer abstracting Debian (`.deb`), Alpine (`.apk`), Flatpak, and rootfs tarball archives. Supports reproducible transactional planning, dependency closure enforcement, and autonomous agent operations.

### Section 2: Core Data Model & Types
- Types defined in `code/aiosh-rust/aiosh-core/src/package.rs`:
  - `PackageSpec`: `name`, `version`, `architecture`, `format`, `state`, `description`, `installed_size_bytes`, `sha256`, `repository_url`, `dependencies`.
  - `PackageFormat`: `deb`, `apk`, `flatpak`, `tarball`.
  - `PackageState`: `available`, `installed`, `upgradable`, `pending_install`, `pending_removal`, `broken`.
  - `PackageDependency`: `name`, `version_constraint`, `optional`.
  - `PackageAction`: `action` (`install`, `upgrade`, `remove`, `reinstall`), `package_name`, `target_version`.
  - `PackageTransaction`: `id`, `actions`, `dry_run`, `total_size_delta_bytes`, `created_at`.
- Invariants PM1..PM5: Naming syntax, sizing bounds, dependency hygiene, SHA-256 integrity, state consistency.

### Section 3: Core Service, Store Registry & Transaction Lifecycle
- Architecture in `code/aiosh-rust/aiosh-core/src/package_service.rs`:
  - `PackageStore`: Canonical in-memory registry seeded with reference Debian and Alpine packages.
  - Transaction Planning: Deterministic dependency graph closure, topological action ordering, and delta size arithmetic.
  - Invariants CS1..CS5: Registry uniqueness, deterministic plan IDs, dependency closure verification, size delta anti-tamper detection, and atomic persistence with RAII `.tmp` cleanup.

### Section 4: Configuration Subsystem (`PackageConfig`)
- Architecture in `code/aiosh-rust/aiosh-core/src/package_config.rs`:
  - Resolution Precedence: Explicit file (`--config <path>`) > environment variables (`AIOS_PACKAGE_*`) > secure defaults.
  - Invariants PC1..PC6: Store path validation, store size limit ($[64\text{ KiB} \dots 100\text{ MiB}]$), entity bounds ($[10 \dots 100,000]$), repository transport security (`https://` or `file://`), 64 KiB config stream read cap.

### Section 5: Security Policy Subsystem (`PackageSecurityPolicy`)
- Architecture in `code/aiosh-rust/aiosh-core/src/package_policy.rs`:
  - Invariants PP1..PP6: Configuration bounds, prohibited package list (`telnet`, `rsh`, `rlogin`, `rexec`, `nis`, `yp-tools`), mandatory SHA-256 digests, transport encryption, architecture whitelists.
  - Modes: `Enforcing` (fail-closed), `Audit` (log only), and `Permissive`.

### Section 6: Observability Telemetry Subsystem (`PackageObservabilityReport`)
- Architecture in `code/aiosh-rust/aiosh-core/src/package_observability.rs`:
  - Invariants PO1..PO6: Inventory completeness, multi-dimensional distribution maps, footprint calculation with saturation arithmetic (`u64::saturating_add`), $O(1)$ memory dependency distribution histogram (`"0"`, `"1-5"`, `"6-10"`, `"11+"`), policy compliance summary, deterministic JSON formatting.

### Section 7: Operator CLI Surface Reference (`aiosh package *`)
- Detailed command reference, options, exit codes, and examples for:
  - `aiosh package validate (--name <name> | --spec <file_or_json>) [--json]`
  - `aiosh package list [--format <deb|apk|flatpak|tarball>] [--state <state>] [--pattern <str>] [--limit <n>] [--store <path>] [--json]`
  - `aiosh package show <name> [--store <path>] [--json]`
  - `aiosh package search <pattern> [--limit <n>] [--store <path>] [--json]`
  - `aiosh package plan --actions <file_or_json> [--dry-run] [--store <path>] [--json]`
  - `aiosh package apply (--actions <file_or_json> | --plan <file_or_json>) [--dry-run] [--yes] [--store <path>] [--json]`
  - `aiosh package config [--config <path>] [--json]`
  - `aiosh package policy [--package <name>] [--config <path>] [--json]`
  - `aiosh package stats [--store <path>] [--config <path>] [--json]`

### Section 8: Autonomous Agent MCP Tool Surface Reference (`aios.package.*`)
- JSON-RPC 2.0 interface contracts, schemas, parameters, and return payloads for:
  - `aios.package.validate`
  - `aios.package.list`
  - `aios.package.get`
  - `aios.package.plan`
  - `aios.package.search`
  - `aios.package.apply`
  - `aios.package.config`
  - `aios.package.policy`
  - `aios.package.stats`

### Section 9: Failure Modes, Error Envelopes, and Audit Trail
- Structured error codes (`INVALID_ARGUMENT`, `PACKAGE_NOT_FOUND`, `LOAD_STORE_FAILED`, `LOAD_POLICY_FAILED`, `PAYLOAD_TOO_LARGE`, `INVALID_JSON`, `FILE_READ_ERROR`, `PLAN_FAILED`, `PERSISTENCE_FAILED`, `CONFIG_RESOLUTION_FAILED`).
- Non-repudiation audit logging: CLI actions to `audit.db` via `classify_and_emit`; MCP tool actions to SQLite WAL ring buffer via `dispatch::recorded_call`.

---

## 3. Automated Documentation Unit Test Contract (`tools/test_package_doc.py`)

The verification script must implement standard checks D1..D6:
- **D1**: File existence and size within bounds $[1,000 \dots 5,242,880]$ bytes.
- **D2**: All 9 required section headings verbatim.
- **D3**: Zero forbidden rot markers (`TODO`, `FIXME`, `TBD`, `XXX`, `PLACEHOLDER`).
- **D4**: All 9 CLI subcommands documented with options and exit codes.
- **D5**: All 9 MCP tools documented with JSON-RPC examples.
- **D6**: Copy-pasteable syntax blocks for both Bash and JSON.
