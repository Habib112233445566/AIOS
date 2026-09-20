# Task Evidence: T-02023 - Capability Model / CLI surface: Scaffold (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02023`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Scaffold `aiosh capability` (`aiosh cap`) CLI subcommand routing, help text, and argument skeleton in `code/aiosh-rust/aiosh-cli/src/main.rs`.

---

## 2. Scaffold Implementation Details
- In `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Added routing for `Some("capability") | Some("cap") => cmd_capability(&args[1..])`.
  - Updated the CLI root help text with `aiosh capability <list|show|issue|attenuate|revoke|check|prune>`.
  - Scaffolded `cmd_capability` function with:
    - Path hygiene check using `validate_service_path(store_path)`.
    - Context opening for audit emission.
    - Subcommand routing skeleton for `list`, `show`, `issue`, `attenuate`, `revoke`, `check`, and `prune`.
    - Dedicated help text for `aiosh capability`.
    - Unknown subcommand rejection returning exit code 2 and structured error payload.
