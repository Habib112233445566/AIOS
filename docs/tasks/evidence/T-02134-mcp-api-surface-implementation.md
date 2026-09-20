# Task Evidence: T-02134 (MCP/API Surface: Implementation)

## Overview
- **Task ID**: `T-02134`
- **Task Name**: MCP/API surface: Implementation
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:48:00+05:00
- **Status**: COMPLETED

## Objective
Implement MCP tool handlers and schemas in `code/aiosh-rust/aiosh-mcp/src/main.rs` for the PEP Decision Engine:
1. `aios.pep.status`: Query service status, rule count, and policy store path.
2. `aios.pep.rule_add`: Register a new policy rule with validated ID, subject, resource, action, effect, and store persistence.
3. `aios.pep.rule_list`: List registered policy rules with optional subject and action filtering.
4. `aios.pep.rule_remove`: Remove a policy rule by ID with atomic store update.
5. `aios.pep.evaluate`: Enhanced to evaluate against the persistent policy store when inline `rules` are omitted, while retaining backward compatibility for inline rules.

## Implementation Details
1. **Schema Definitions in `tools/list`**:
   - `aios.pep.status`: Parameters: `store_path` (optional string).
   - `aios.pep.rule_add`: Parameters: `id` (required string), `effect` (required string: "permit"|"deny"), `subject` (optional string), `resource` (optional string), `action` (optional string), `description` (optional string), `store_path` (optional string).
   - `aios.pep.rule_list`: Parameters: `subject` (optional string), `action` (optional string), `store_path` (optional string).
   - `aios.pep.rule_remove`: Parameters: `id` (required string), `store_path` (optional string).
   - `aios.pep.evaluate`: Added `store_path` (optional string) and made `rules` optional array.

2. **Tool Dispatch Arms in `call_tool`**:
   - Each handler validates inputs using `aiosh_core::pep_decision_service::validate_pep_service_path` to prevent path traversal.
   - Input validation enforces rule ID length (<= 128 chars) and no control characters.
   - Operations are dispatched through `dispatch::recorded_call`, ensuring every invocation is recorded in the SQLite audit ring with structured parameters.
   - Non-destructive store recovery is handled via `PepDecisionService::load_or_recover`.

## Verification
- Clean compilation verified via `cargo check -p aiosh-mcp` and `cargo build -p aiosh-mcp`.
