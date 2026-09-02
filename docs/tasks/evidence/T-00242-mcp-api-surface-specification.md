# T-00242 — Phase 0 — Release Packaging & Backup / MCP/API surface: Specification

## Goal
Specify the exact contract for the MCP/API surface of Release Packaging & Backup.

## Inputs & Outputs

### 1. `aios.backup.create` (MCP Tool)
**Inputs (JSON Schema):**
- `target_path` (string, required): The physical directory to snapshot.
- `include_audit` (boolean, optional, default: true): Include `/audit` directory if present.
- `include_memory` (boolean, optional, default: false): Include `/memory` directory if present.
- `grant_id` (string, optional): PEP authorization token.

**Outputs (JSON):**
```json
{
  "ok": true,
  "action": "aios.backup.create",
  "data": {
    "backup_path": "aios_backup_2026-08-26T20-30-00Z.zip"
  },
  "audit_id": 42,
  "classifier_policy_revision": "rev-xyz"
}
```

### 2. `aios.release.generate` (MCP Tool)
**Inputs (JSON Schema):**
- `target_os` (string, required): The target OS configuration.
- `version` (string, required): Version tag.
- `components` (array of strings, optional, default: ["core"]): Components to embed.
- `grant_id` (string, optional): PEP authorization token.

**Outputs (JSON):**
```json
{
  "ok": true,
  "action": "aios.release.generate",
  "data": {
    "artifact_path": "output/release/aios_ubuntu_1.0.0.iso",
    "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  },
  "audit_id": 43,
  "classifier_policy_revision": "rev-xyz"
}
```

## Error Cases
If the `dispatch_mod.dispatch()` PEP gate rejects the execution (e.g., bad grant token or missing permissions), the tool will return a fast-fail refusal envelope *without* executing physical core logic. 

**Refusal Envelope:**
```json
{
  "ok": false,
  "action": "aios.release.generate",
  "gate": "refused",
  "reason": "Missing required capability 'system.admin'",
  "audit_id": 44
}
```

If the core function crashes during execution (e.g., permission denied writing the file), it raises an exception which is caught by the wrapper. An `error` row is written to the audit ledger via `dispatch_mod.commit()`, and the tool returns:
**Error Envelope:**
```json
{
  "ok": false,
  "action": "aios.release.generate",
  "error": "genisoimage failed: missing dependency",
  "audit_id": 45
}
```

## Persistence Effects
The tool wrappers rely explicitly on `dispatch_mod.dispatch()` to enforce policy and `dispatch_mod.commit()` (for errors). However, the internal functions `create_backup` and `generate_release` *also* invoke `audit_client.write_audit_row` natively.
To prevent duplicate rows (ADR-0035 invariant violation), the core Python internal functions must be adjusted *or* the MCP wrapper must rely on the core functions to do the primary commit, feeding the classifier metadata directly into the core function. The architecture explicitly injects `**classifier_kwargs` into `create_backup` for this exact purpose (as implemented in T-0218). Therefore, the MCP wrapper will *only* invoke `dispatch_mod.dispatch` to get the verdict, and then pass that verdict data to the core function. It will NOT call `dispatch_mod.commit()` on the success path, as `create_backup` does that internally.

## Acceptance Criteria Verified
- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
