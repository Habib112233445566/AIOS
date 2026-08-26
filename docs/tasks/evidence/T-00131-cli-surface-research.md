# T-00131 — CI Smoke Orchestration / CLI surface: Research

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration CLI surface

## 1. Scope & Objective
Establish facts, constraints, and prior art for the CLI surface of CI Smoke Orchestration. The goal is to determine how the CI core service (built in Epic T-00121..T-00130) should be exposed to operators and agents via the command line, noting that the `aiosh` binary is the primary v2.1 shipping surface.

## 2. Facts vs. Assumptions

### F1: The CLI surface is already natively integrated in Rust
- **FACT:** During T-00128 (core service: Hardening), the strict requirement to "write an honest audit row (per ADR-0035 §F-2)" necessitated natively integrating the CI service into the Rust CLI.
- **FACT:** The Python `tools/ci_service.py` is not part of the S-rank pipeline natively and cannot easily emit audit rows without hacky subprocesses. Therefore, the implementation was ported to `aiosh-core/src/ci.rs` and exposed directly as the `aiosh ci` subcommand in `aiosh-cli/src/main.rs`.
- **FACT:** The existing CLI surface supports the required actions: `aiosh ci check`, `aiosh ci show`, and `aiosh ci failures`, along with the `--file PATH` flag.

### F2: Legacy Substrate Parity
- **FACT:** The v2.1 shipping stack mandate designates Rust as the single source of truth (`code/aiosh-rust`). The TS and Python implementations are considered legacy reference substrates.
- **ASSUMPTION:** The TS CLI (`code/aiosh-cli/src/cli.ts`) does not have a `ci` command and is not required to have one under the v2.1 mandate, since we are moving all new functionality to Rust.

### F3: MCP Exposure
- **FACT:** Task T-00112 specifically excluded MCP exposure of orchestration as out-of-scope.
- **ASSUMPTION:** We do not need to add an MCP tool for the CI surface in this epic, unless specifically requested. The CLI is the primary target.

## 3. Authoritative Sources & Prior Art
- **v2.1 Mandate:** `docs/README.md` explicitly states that `code/aiosh-rust/` is the shipping stack.
- **ADR-0035:** Defines S-rank architecture and the absolute requirement for audit row emission (F-2) on all consequential actions.
- **Prior Art (Task Ledger Control):** `aiosh task` is implemented purely in Rust (`code/aiosh-rust/aiosh-cli/src/main.rs`) and not in the TS CLI, demonstrating that new CLI features only need to be added to the Rust stack.

## 4. Decisions Needed Before Implementation
- **D1 (Actionability):** Since the `aiosh ci` surface is already fully implemented, tested, and integrated in Rust, the remaining tasks in this epic (T-00132 through T-00140) will primarily serve to specify, document, and formally verify the *existing* implementation, rather than writing net-new code.
- **D2 (Legacy):** Do we need to port `aiosh ci` to the TypeScript `code/aiosh-cli`? Decision: No, per the v2.1 Rust-only mandate.

## 5. Conclusion
No new code needs to be scaffolded for the CLI surface; it was preemptively built to satisfy the hardening requirements of the previous epic. The upcoming tasks will verify and document this surface.
