# T-02431: Sandbox Enforcement MCP/API Surface Research

## 1. Research Objectives
Establish authoritative facts, constraints, and architecture for exposing Sandbox Enforcement capabilities via Model Context Protocol (MCP) JSON-RPC 2.0 tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Existing Substrate & Prior Art Analysis
1. **Existing Tools**:
   - `aios.sandbox.profiles`: Returns pre-configured and dynamically loaded profiles.
   - `aios.sandbox.probe`: Queries host OS kernel features (`landlock_supported`, `landlock_abi_version`, `seccomp_bpf_supported`, `no_new_privs_supported`, `platform`).
2. **Missing Surface Component**:
   - `aios.sandbox.exec`: Tool enabling LLM agents to execute supervised commands within bounded containment profiles with structured results.
3. **MCP Architectural Patterns**:
   - Tool registration in `tools/list` schema handler.
   - Dispatch through `dispatch::recorded_call` to guarantee audit trail row emission per execution.
   - Input schema validation conforming to JSON Schema Draft-07.
   - Standardized JSON envelope: `{"code": i32, "data": Value, "error": Value}`.

## 3. Facts vs Assumptions
- **Fact**: MCP tools must not block indefinitely; process-level execution requires watchdog monitoring and bounded memory output capture (`DEFAULT_MAX_OUTPUT_CAPTURE_BYTES = 10 MiB`).
- **Fact**: Consequential tool calls must be gated by PEP rules when `enforce_pep_grants` is set.
- **Fact**: `command` and `cwd` parameters must be validated to prevent directory traversal (`..`) and argument injection.
- **Assumption**: Autonomous AI agents prefer structured JSON return schemas containing exit codes and captured stdout/stderr instead of raw stream handles.

## 4. Key Design Decisions for Specification (T-02432)
1. Register `aios.sandbox.exec` with typed input schema:
   - `command` (string, required)
   - `args` (array of strings, optional)
   - `profile` (string, optional, default "standard")
   - `cwd` (string, optional)
   - `grant_token` (string, optional)
2. Enforce strict error codes:
   - `ERR_SANDBOX_INVALID_PATH` for directory traversal in `cwd` or `command`.
   - `ERR_SANDBOX_PROFILE_NOT_FOUND` for unknown profile names.
   - `ERR_SANDBOX_PEP_UNAUTHORIZED` when authorization token is missing or invalid.
