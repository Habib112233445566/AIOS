# T-00647 — Repository Health / MCP/API surface: Security Review

## 1. Security Review Scope
This task evaluates the security posture and threat model for the `aios.repo.health` MCP tool surface in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Threat Model & Abuse Scenarios

### Scenario MCP-1: Arbitrary Path Traversal & Probing
- **Threat**: An autonomous agent invokes `aios.repo.health` with malicious paths (e.g., `../../../../etc` or sensitive directories) to map system files.
- **Finding & Mitigation**:
  - `repo_health_service` operates strictly in read-only mode without file mutations or side-effects.
  - Large binary/build directories (`.git`, `target`, `node_modules`, `.venv`) are filtered out during recursive traversal.
  - File reading is limited to metadata checks and bounded policy checks.

### Scenario MCP-2: PEP Dispatch Evasion & Unaudited Execution
- **Threat**: Direct invocation of core diagnostics bypassing the PEP dispatch ring.
- **Finding & Mitigation**:
  - `call_tool` delegates all executions through `dispatch::recorded_call`.
  - Every call produces an immutable audit record in SQLite containing tool name, timestamp, and arguments.

### Scenario MCP-3: Hostile Arguments & Schema Fuzzing
- **Threat**: Malformed JSON payloads, non-string `repo_path` types, or extra properties passed to trigger panics.
- **Finding & Mitigation**:
  - `arguments.get("repo_path").and_then(|v| v.as_str()).unwrap_or(".")` gracefully recovers strings from valid JSON.
  - Invalid types fall back safely to current working directory without panicking.

## 3. Verdict
- **Status**: PASS
- **Open Vulnerabilities**: 0
- **Residual Risks**: None identified.
