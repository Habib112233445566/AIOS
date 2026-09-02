# T-00289 — Release Packaging & Backup: Observability Documentation

## Documentation Updates

**File Modified**: `docs/README.md`

**Updates Made**:
1. Added an **Observability & Troubleshooting** subsection under **Release Packaging & Backup**.
2. **Honest Limitations**: Explicitly noted that progress is not streamed. Large backups or ISOs will block synchronously, meaning the tool might appear to hang to the caller until completion.
3. **Diagnostic Guidance**: Instructed operators and agents on how to leverage the newly implemented error capturing. The docs state that failures are serialized into the `outcome_detail` parameter of the ledger row, meaning `aiosh ledger tail` is the authoritative way to debug OS-level tool crashes (like `genisoimage`).

## Acceptance Validation
- The documentation accurately reflects the observability enhancements implemented in `T-00284` and `T-00288`.
- The limitations are stated prominently without obfuscation.
