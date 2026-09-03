# T-01071 — Distro Selection & Justification / Observability: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Observability

## 1. Observability Architecture & Metrics
Research observability and telemetry aggregation for the Linux Distro Selection & Justification subsystem:
- **Registry Telemetry**:
  - `total_profiles`: Total profiles registered in the store.
  - `recommended_profile_id`: Currently designated reference profile ID.
  - `production_ready_count`: Number of profiles meeting production readiness threshold.
  - `policy_compliant_count`: Number of profiles meeting current security policy rules.
- **Score Distributions & Averages**:
  - `average_overall_score`, `average_security_score`, `average_footprint_score`, `average_binary_compatibility_score`.
- **Taxonomy Breakdowns**:
  - `family_breakdown`: Count by distribution family (e.g. Debian, Alpine, Fedora, Arch).
  - `architecture_breakdown`: Count by CPU architecture (e.g. x86_64, aarch64, riscv64).
- **Surfaces**:
  - CLI: `aiosh distro stats [--json]`.
  - MCP: `aios.distro.stats` JSON-RPC tool.

## 2. Invariant Rules
- Aggregate invariant: Sum of family counts == `total_profiles`.
- Aggregate invariant: Sum of architecture counts == `total_profiles`.
- Read-only invariant: Observability queries emit zero state-changing audit events.
