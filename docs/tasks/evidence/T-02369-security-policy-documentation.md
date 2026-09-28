# Task Evidence: T-02369 (Audit Chain Extensions / security policy: Documentation)

## 1. Metadata
- **Task ID:** `T-02369`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Documentation
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (9/10) — Documentation

---

## 2. Documentation Updates

### 2.1 Specification Updates
- Updated `docs/SPEC-AUDIT-EXTENSIONS.md` with Section 6 ("Audit Chain Security Policy Subsystem").
- Documented policy modes (`enforcing`, `permissive`, `disabled`), error codes (`AUDITPOL_ERR_*`), and enforcement invariants.
- Included copy-pasteable operator CLI and MCP JSON-RPC invocation commands.

### 2.2 Operational Examples
```bash
# Operator CLI inspection
aiosh audit policy

# Machine-readable JSON output
aiosh audit policy --json

# MCP Tool execution
aios.audit.policy {"policy_path": "config/audit_policy.json"}
```

### 2.3 Known Constraints & Limitations
- Policy applies only to extended rows ingested via `AuditChainService::record_event`.
- Future clock skew tolerance is capped at 300 seconds; nodes with excessive clock drift will fail event ingestion.
- Ed25519 signatures must match the hex-encoded 32-byte public key and 64-byte signature formatting.

---

## 3. Acceptance Confirmation
- [x] Documentation updated with runnable CLI and MCP examples.
- [x] Constraints and limitations documented explicitly.
- [x] Evidence files cross-referenced.
