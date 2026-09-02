# T-00951 — Agent Handoff Protocol / Configuration: Research

## 1. Prior Art & Architecture
- Configuration pattern follows `TriageConfig` in `aiosh-core::triage_config`.
- `HandoffConfig` structure:
  - `max_store_bytes`: Capacity bounds for `handoff_store.json` (16 KiB to 64 MiB, default 1 MiB).
  - `default_priority`: Default `HandoffPriority` when unspecified.
  - `default_ttl_seconds`: Default expiration duration in seconds (default: 86400 = 24h).
  - `allow_auto_accept`: Flag indicating whether receiving agents can automatically accept handoffs.
  - `store_path`: Optional override for the store location.
- Environment variables: `AIOSH_HANDOFF_CONFIG`, `AIOSH_HANDOFF_STORE`.
- Default config persisted at `docs/handoff_config.json`.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Struct Location | Fact | Module `code/aiosh-rust/aiosh-core/src/handoff_config.rs`. |
| Max File Size | Fact | Capped at 64 KiB (`MAX_CONFIG_FILE_BYTES`). |
| Validation | Fact | `validate()` rejects out-of-bound `max_store_bytes` or zero TTL. |
