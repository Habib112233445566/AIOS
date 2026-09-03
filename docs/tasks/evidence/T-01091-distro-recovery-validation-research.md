# T-01091 — Distro Selection & Justification / Recovery & Validation: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Recovery & Validation

## 1. Prior Art & Subsystem Recovery Patterns
- **Store Resilience & Corruption Recovery**:
  - `DistroStore::load_or_recover(path)` provides fault-tolerant recovery when a custom `distro_store.json` on disk is truncated, malformed, or unreadable.
  - Fail-safe fallback creates a sanitized default store containing canonical Debian and Alpine profiles.
- **Deep Store & Registry Validation**:
  - Validates all contained `DistroProfile` structures (ID format, semantic versions, kernel baselines, architectures).
  - Validates recommended profile resolution (target ID must exist in registry).
  - Validates score matrix bounds ($[0.0, 1.0]$) and evaluation integrity.
- **Surface Exposure**:
  - CLI: `aiosh distro check [--json] [--store <path>]`.
  - MCP: `aios.distro.check`.
  - Test Suite: Criterion `D6` in `tools/test_distro_suites.py`.

## 2. Invariant Rules
- Non-destructive recovery: Corrupted files are never silently overwritten without a timestamped backup (`.bak.<timestamp>`).
- Health report invariants: `healthy == (errors.is_empty())`.
