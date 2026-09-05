# T-01229: Package Management - CLI Surface: Documentation

## Metadata
- **Task ID:** `T-01229`
- **Subsystem:** `docs/README.md`
- **Component:** Package Management CLI Surface Documentation
- **Status:** Complete

## 1. Documentation Updates
Updated `docs/README.md` §8.12 (*Linux Package Management Subsystem*) to fully describe the operator CLI surface (`aiosh package`, tasks `T-01221` through `T-01230`):
- **Command Reference**:
  - `aiosh package validate --name <name> [--json]`
  - `aiosh package validate --spec <file_or_inline_json> [--json]`
  - `aiosh package list [--format <deb|apk|flatpak|tarball>] [--state <state>] [--pattern <str>] [--limit <n>] [--json] [--store <path>]`
  - `aiosh package show <name> [--json] [--store <path>]`
  - `aiosh package search <pattern> [--limit <n>] [--store <path>] [--json]`
  - `aiosh package plan --actions <json_or_file> [--dry-run] [--json] [--store <path>]`
  - `aiosh package apply (--actions <json_or_file> | --plan <json_or_file>) [--dry-run] [--yes] [--store <path>] [--json]`
- **Hardening & Security Defenses**:
  - Size ceilings: 1 MiB for `--actions`, `--plan`, `--spec`; 10 MiB and 10,000 entities for store files.
  - Bounds & sanitization: query/pattern length <= 256, package name <= 64, store path <= 1,024, `--limit` between 1 and 10,000, and rejection of ASCII control characters.
  - Standard error envelopes with explicit machine-readable codes.
  - Audit logging via `classify_and_emit` to SQLite `audit.db` on all paths per ADR-0035.
- **Honest Constraints & Known Limitations**:
  - Dependency resolution is batch-deterministic; network repository fetching will be introduced in subsequent milestones.
  - File unpacking and binary execution are deferred to container/system chroot runners.
  - Store size ceiling is capped at 10,000 packages.

## 2. Copy-Pasteable Usage Examples
```bash
# Validate a package name
aiosh package validate --name "curl"

# List all Debian packages currently installed
aiosh package list --format deb --state installed

# Show details of a specific package
aiosh package show curl

# Search for packages matching "lib"
aiosh package search lib --json

# Plan an installation transaction (dry run)
aiosh package plan --actions '[{"action":"install","package_name":"libssl3"},{"action":"install","package_name":"curl"}]' --dry-run --json

# Apply an installation transaction with store persistence
aiosh package apply --actions '[{"action":"install","package_name":"libssl3"},{"action":"install","package_name":"curl"}]' --store ./packages.json --json
```

## 3. Test Runner Invariant Documentation
Documented runner execution including criterion `PM3`:
```bash
python tools/test_package_suites.py
# [+] PM1 package data model integrity & invariants (PM1..PM5)
# [+] PM2 package core service integrity & invariants (CS1..CS5)
# [+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
# PASS: package_suites criteria (PM1..PM3)
```

## 4. Evidence Traceability
- Research: `docs/tasks/evidence/T-01221-cli-surface-research.md`
- Specification: `docs/tasks/evidence/T-01222-cli-surface-specification.md`
- Scaffold: `docs/tasks/evidence/T-01223-cli-surface-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01224-cli-surface-implementation.md`
- Unit Tests: `docs/tasks/evidence/T-01225-cli-surface-unit-test.md`
- Integration: `docs/tasks/evidence/T-01226-cli-surface-integration.md`
- Security Review: `docs/tasks/evidence/T-01227-cli-surface-security-review.md`
- Hardening: `docs/tasks/evidence/T-01228-cli-surface-hardening.md`
