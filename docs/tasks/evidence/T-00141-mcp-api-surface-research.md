# T-00141 — CI Smoke Orchestration / MCP/API surface: Research

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration MCP/API surface

## 1. Scope & Objective
Establish facts, constraints, and prior art for exposing the CI Smoke Orchestration core service to S-rank agents via the Model Context Protocol (MCP). This enables agents to query CI test results organically during their workflow without shelling out to raw terminal commands.

## 2. Facts vs. Assumptions

### F1: Current Integration State
- **FACT:** The core service logic (`load_summary_with_retry`, `human_report`) and CLI integration (`aiosh ci`) are fully implemented natively in Rust (`aiosh-core` and `aiosh-cli`). 
- **FACT:** There is currently no `aios.ci` MCP tool registered in `code/aiosh-rust/aiosh-mcp/src/main.rs`. Agents currently must use the CLI indirectly via `run_terminal_command` if they wish to consume the CI verdict.

### F2: Audit & Gating Constraints (ADR-0035)
- **FACT:** `aiosh ci check` writes a consequential audit row natively to explicitly seal the CI run with a `success` or `failure` outcome (as implemented in T-00128).
- **CONSTRAINT:** Any MCP wrapper (`aios.ci`) must honor this and correctly delegate to the underlying `aiosh_core::ci` logic, ensuring the classifier+PEP+audit gate applies. If the MCP tool replicates the `check` action, it must write the identical audit row. 

### F3: S-Rank Agent Use Cases
- **FACT:** An agent modifying a test suite or application logic needs to verify if its changes broke anything by running CI and consuming the result. 
- **ASSUMPTION:** A read-only query like `show` or `failures` over MCP provides higher-fidelity and safer context to the agent than forcing it to parse raw stdout from a terminal command, aligning with the project's S-rank MCP abstraction layer.

## 3. Prior Art & Upstream Context
- **MCP Standard:** Anthropic's Model Context Protocol specifies structured JSON inputs/outputs for tool calls.
- **`aios.task` Prior Art:** The task ledger was exposed to MCP as `aios.task` with sub-actions like `status`, `check`, `validate`. The `aios.ci` tool should adopt a similar JSON RPC structure (`{"action": "check|show|failures"}`).

## 4. Decisions Needed Before Implementation
- **D1 (Action Mapping):** The MCP tool `aios.ci` should accept a single string `action` parameter (`check`, `show`, `failures`) matching the CLI exactly. 
- **D2 (Audit Granularity):** Do `show` and `failures` need to emit audit rows when called over MCP? The CLI currently only emits an audit row on `check` (and on load error). We must preserve parity: `check` and errors write audit rows; `show` and `failures` do not (they are pure reads), unless ADR-0035 specifies otherwise for MCP.
- **D3 (Payload Structure):** The MCP tool should return the structured data (the JSON `RunSummary` object) when `action` is `check` or `failures`, rather than just the human-readable string, so the LLM agent can parse it natively. (Or it can just return the human string, which LLMs are good at reading, to reuse the exact `human_report` logic).
