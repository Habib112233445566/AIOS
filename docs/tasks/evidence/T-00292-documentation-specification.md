# T-00292 — Release Packaging & Backup: Documentation Specification

## Documentation Updates Contract

### 1. Unified Sub-Section
The existing `docs/README.md` must be reorganized to provide a complete "operator manual" for `Release Packaging & Backup`. It will consolidate the disparate notes (configuration, PEP gating, observability) into structured, readable headers.

### 2. MCP JSON Payload Examples (New)
The documentation must explicitly show an AIOS agent how to trigger a backup. 
- **Input Example**: 
  ```json
  {
    "tool": "aios.backup.create",
    "args": {
      "target_path": "/var/aios",
      "include_audit": true,
      "include_memory": false
    }
  }
  ```
- **Constraint**: Must explicitly state that agents require an active PEP grant for `aios.backup.create` or `aios.backup.*` to successfully submit this payload.

### 3. Edge Cases & Error Behaviors
The documentation must formally explain that failures do not crash the MCP server. Instead, they write an `outcome = "error"` to the `AuditRing`, and the caller should parse the `outcome_detail` for the OS-level subprocess string.

## Target File
`docs/README.md` under the `# AIOS Release Packaging & Backup` header. No new files will be created to maintain a central entry point.
