# T-01399: Init & Service Supervision Recovery & Validation Documentation

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01399  

---

## 1. Executive Summary
Task `T-01399` documented the **Init & Service Supervision Recovery & Validation** subsystem for system operators and autonomous AI agents. The documentation was integrated into:
1. `code/aiosh-mcp/README.md`: Added `aios.service.check` to the tool catalog table and detailed copy-pasteable JSON-RPC 2.0 invocation examples.
2. `docs/service_supervision.md`: Updated Section 7 (Operator CLI Reference) with `aiosh service check` examples and Section 8 (MCP Reference) with `aios.service.check` specification.

---

## 2. Copy-Pasteable Usage Examples

### 1. Operator CLI Surface (`aiosh service check`)
```bash
# Audit on-disk service store integrity (read-only audit mode)
aiosh service check --store /var/lib/aios/services.json --json

# Automatically repair corrupted or unreadable service store with timestamped backup
aiosh service check --store /var/lib/aios/services.json --fix --json
```

### 2. Autonomous Agent MCP Surface (`aios.service.check`)
```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "aios.service.check",
    "arguments": {
      "store_path": "/var/lib/aios/services.json",
      "auto_recover": true
    }
  }
}
```

---

## 3. Operational Constraints & Known Limitations
- **10 MiB File Ceiling:** Stores exceeding 10 MiB are rejected during load and audit evaluation.
- **10,000 Unit Entity Bound:** The registry is capped at 10,000 service specifications; larger counts trigger invariant `SR1..SR3` non-compliance errors.
- **Path Length Boundary:** Custom store paths are strictly bounded to 1,024 characters and must contain zero ASCII control characters.
- **Canonical Seed Invariance:** Auto-recovery always reconstitutes the standard reference set (`aios-securityd.service`, `auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `ssh.service`). Any custom user services defined before corruption are preserved inside the quarantined `.corrupt.<ts>.bak` file for manual operator recovery.
- **Audit Non-Repudiation:** Every audit and repair operation emits an immutable, SHA-256 hash-chained audit record to `audit.db` / `audit.log`.

---

## 4. Evidence Trace
- `T-01391`: Research — Prior art, systemd/OpenRC comparison (`docs/tasks/evidence/T-01391-recovery-validation-research.md`)
- `T-01392`: Specification — Formal invariants SR1..SR5 and data schemas (`docs/tasks/evidence/T-01392-recovery-validation-specification.md`)
- `T-01393`: Scaffold — Module skeleton and type exports (`docs/tasks/evidence/T-01393-recovery-validation-scaffold.md`)
- `T-01394`: Implementation — Core operational behavior (`docs/tasks/evidence/T-01394-recovery-validation-implementation.md`)
- `T-01395`: Unit Test — Focused automated test suite (`docs/tasks/evidence/T-01395-recovery-validation-unit-test.md`)
- `T-01396`: Integration — Production CLI and MCP surfaces (`docs/tasks/evidence/T-01396-recovery-validation-integration.md`)
- `T-01397`: Security Review — Threat modeling & abuse scenarios (`docs/tasks/evidence/T-01397-recovery-validation-security-review.md`)
- `T-01398`: Hardening — Resource limits and bounded loops (`docs/tasks/evidence/T-01398-recovery-validation-hardening.md`)
- `T-01399`: Documentation — Operator and agent usage guides (`docs/tasks/evidence/T-01399-recovery-validation-documentation.md`)
