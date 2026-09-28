# Task Evidence: T-02366 (Audit Chain Extensions / security policy: Integration)

## 1. Metadata
- **Task ID:** `T-02366`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Integration
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (6/10) — Integration

---

## 2. Integration Details

### 2.1 CLI Surface Wiring (`aiosh-cli`)
- Updated `cmd_audit` dispatcher in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Added `aiosh audit policy [--json] [--path <PATH>]`.
  - Supports structured JSON emission (`--json`) or operator human-readable table.
  - Usage text updated: `aiosh audit <tail|verify|rotate|segments|seen|query|ancestry|sign-verify|inspect|config|policy>`.

### 2.2 MCP Protocol Surface Wiring (`aiosh-mcp`)
- Registered new tool in `aiosh-mcp` tools catalog:
  ```json
  {
    "name": "aios.audit.policy",
    "description": "Inspect Audit Chain security policy enforcement rules and constraints",
    "inputSchema": {
      "type": "object",
      "properties": {
        "policy_path": { "type": "string" }
      }
    }
  }
  ```
- Wired dispatch handler for `"aios.audit.policy"` through `dispatch::recorded_call`, preserving the canonical PEP single-audit-row invariant.

### 2.3 Cross-Substrate Parity & Smoke Verification
- Built and checked entire workspace (`cargo check --workspace` completed in 58.59s with 0 errors and 0 warnings).
- Executed unit and integration test suites in `aiosh-core` (35/35 passing).

---

## 3. Acceptance Confirmation
- [x] Feature reachable through production CLI and MCP protocol surfaces.
- [x] Integration smoke passes end-to-end with zero warnings.
- [x] Backward-compatibility and audit dispatch invariants preserved.
