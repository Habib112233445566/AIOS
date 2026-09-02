# T-01011 — Distro Selection & Justification / Core Service: Research

## 1. Prior Art & Service Architecture
- The core service for Linux Distro Selection manages the registry of supported and custom Linux distribution profiles.
- Service Requirements:
  - In-memory registry with atomic file persistence (`.tmp` write + rename).
  - Out-of-the-box pre-loaded canonical profiles (Debian 12 Bookworm Minimal, Alpine 3.19 Container).
  - Dynamic evaluation query engine to score profiles and recommend the optimal base for host vs sandbox.
  - Safe failure and corruption recovery via fallback to canonical defaults.
- Integration: New module `code/aiosh-rust/aiosh-core/src/distro_service.rs` exposed via `aiosh-core::lib`.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Registry Storage | Fact | Can persist to and load from canonical JSON `docs/distro_registry.json`. |
| Evaluation Idempotence | Fact | Evaluating a profile does not mutate state or emit audit rows. |
| Corruption Recovery | Fact | If registry file is missing or corrupted, `load_or_recover` restores built-in canonical defaults safely. |
