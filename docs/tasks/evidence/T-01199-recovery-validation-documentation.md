# T-01199: Base Image Build - Recovery & Validation: Documentation

## Metadata
- **Task ID:** `T-01199`
- **Subsystem:** `code/aiosh-rust/aiosh-core::base_image_recovery`
- **Component:** Recovery & Validation Documentation
- **Status:** Complete

## 1. Summary of Documentation Deliverables
The Base Image Build Recovery & Validation subsystem has been documented across both the architectural/operational guide and the root README:

1. **Operational Guide (`docs/base_image_build.md`):**
   - **Section 7 (CLI Reference):** Documented `aiosh image check [--fix] [--json] [--store <path>]` with syntax and flags.
   - **Section 8 (MCP Reference):** Documented `aios.image.check` tool call interface, parameters (`store_path?: string`, `auto_recover?: bool`), and response envelopes.
   - **Section 9 (Recovery Protocol):** Detailed `load_or_recover` mechanics, non-destructive `.bak.<timestamp>` backup preservation, structured `BaseImageValidationReport`, and mathematical validation invariants `RV1..RV4`.

2. **Root Specification (`docs/README.md`):**
   - **Section 8.11:** Added subsection for Recovery & Validation subsystem covering manifest validation, store recovery, invariants RV1..RV4, CLI surface, MCP tool surface, and evidence range extension.

## 2. Recovery & Validation Invariants (RV1..RV4)
- **`RV1`:** Total manifest invariant: `valid_manifests + invalid_manifests == total_manifests`.
- **`RV2`:** Healthy invariant: `healthy == (errors.is_empty() && invalid_manifests == 0)`.
- **`RV3`:** Error count invariant: `invalid_manifests > 0 ==> errors.len() >= invalid_manifests`.
- **`RV4`:** Forensic preservation invariant: `load_or_recover` creates non-destructive `<path>.bak.<timestamp>` before re-seeding with clean canonical defaults.

## 3. Operator CLI Example
```bash
# Check integrity of default image store
aiosh image check

# Run integrity check and auto-recover if corrupted, producing forensic backup
aiosh image check --fix --json
```

Output:
```json
{
  "code": 0,
  "data": {
    "total_manifests": 4,
    "valid_manifests": 4,
    "invalid_manifests": 0,
    "errors": [],
    "recovered": false,
    "backup_path": null,
    "healthy": true
  },
  "error": null
}
```

## 4. MCP Tool Invocation Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.image.check",
    "arguments": {
      "auto_recover": true
    }
  }
}
```

## 5. Constraints & Known Limitations
- Automatic recovery resets the store file to canonical reference templates; custom non-persisted user templates that were corrupted are retained in the `.bak.<timestamp>` archive for forensic/manual extraction.
- In-memory validation scans up to 10 MiB store sizes and 1,024 packages per manifest.
- Store re-seeding uses atomic rename and `0o644` file permissions.
