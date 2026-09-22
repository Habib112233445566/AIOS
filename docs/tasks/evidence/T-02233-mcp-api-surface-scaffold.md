# Task Evidence: T-02233 (Grant Lifecycle / MCP/API surface: Scaffold)

## 1. Metadata
- **Task ID:** `T-02233`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Scaffold (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Scaffold

---

## 2. Scaffold Implementation Details

1. **Manifest Registration (`tool_manifest`)**:
   - Registered `aios.pep.grant.issue` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Defined JSON Schema inputs:
     - Required: `id`, `subject`, `scope_type`, `rights`
     - Optional: `issuer`, `scope_path`, `delegation_depth`, `expires_at`, `not_before`, `max_invocations`, `max_bytes`, `store_path`, `grant_id`
2. **Call Dispatch Skeleton (`call_tool`)**:
   - Added match branch for `"aios.pep.grant.issue"` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Stubbed fail-loud closure returning `Err("not implemented: aios.pep.grant.issue")`.
   - Enclosed in `dispatch::recorded_call` to guarantee audit trail emission even during early scaffold testing.
3. **Compilation Verification**:
   - `cargo check -p aiosh-mcp` completed cleanly with zero warnings or errors.

---

## 3. Build Verification Output
```text
> cargo check -p aiosh-mcp
    Checking aiosh-mcp v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-mcp)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.54s
```

---

## 4. Acceptance Confirmation
- [x] Project builds and checks with zero warnings/errors.
- [x] New tool interface registered in MCP `tools/list` manifest.
- [x] Scaffold stub wired with fail-loud error in `call_tool`.
