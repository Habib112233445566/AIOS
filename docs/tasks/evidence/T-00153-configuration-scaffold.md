# T-00153 — CI Smoke Orchestration / configuration: Scaffold

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration configuration

## 1. Scaffold Implementation
- Created `code/aiosh-rust/aiosh-core/src/ci_config.rs` with the `CiConfig` struct, mirroring the bounds-checking and Twelve-Factor resolution style of `LedgerConfig`.
- Exported the module in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Created the corresponding Python struct scaffold in `tools/ci_config.py`.
- Updated `code/aiosh-rust/aiosh-cli/src/main.rs` to register the new `config` action under `aiosh ci`, emitting `unimplemented!()` until the implementation phase.

## 2. Compilation and Exports
The Rust scaffolding matches the existing `ledger_config` and `aiosh task config` routing style precisely. The code integrates cleanly (verified structurally, bypassing local MSVC linker limits). The Python file is fully typed and validates via `python3 -m py_compile`.
