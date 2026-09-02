# T-00279 — Security Policy: Documentation

## Documentation Updates

**File Modified**: `docs/README.md`

**Updates Made**:
1. Added a **Security Policy (PEP Gating)** subsection under **Release Packaging & Backup** explaining the exact gating requirements (possession of an active cryptographic grant token) for autonomous agents to use these tools.
2. Clearly stated the constraints: `aios.release.generate` and `aios.backup.create` will synchronously reject the MCP invocation with a 403-equivalent refusal if the grant is missing.

## Acceptance Validation
- The documentation accurately reflects the security policy implemented in `T-00274`.
- The limitations and enforcement requirements are stated clearly and prominently in the README, ensuring operators and agents understand the boundaries.
