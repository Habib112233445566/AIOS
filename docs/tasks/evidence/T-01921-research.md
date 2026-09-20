# Task Evidence: T-01921 - System Update Mechanism / CLI surface: Research

## 1. Overview
- **Task ID**: `T-01921`
- **Sub-Epic**: 3 (System Update Mechanism CLI Surface)
- **Goal**: Research CLI command structures, operator ergonomics, argument parsing patterns, audit logging integrations, and CLI invariants `UCLI1..UCLI6` for the System Update Mechanism.

---

## 2. Research Summary
- Designed `aiosh update` subcommands: `status`, `slots`, `check`, `apply`, `confirm`, `rollback`.
- Defined invariants `UCLI1..UCLI6` (help ergonomics, path hygiene, JSON envelope, audit ring emission, hermetic isolation, deterministic exit codes).
- Formulated CLI architecture for `aiosh-cli::cmd_update`.

---

## 3. Status
Research completed; ready for specification in `T-01922`.
