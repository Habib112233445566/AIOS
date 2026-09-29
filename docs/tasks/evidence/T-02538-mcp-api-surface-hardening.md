# T-02538: Privilege Escalation Prevention MCP/API Surface Hardening

- **Task**: `T-02538`
- **Sub-Epic**: Privilege Escalation Prevention / MCP/API surface
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Hardening Interventions
1. **Path Traversal Shielding (`get_privilege_store_path`)**:
   - Environment variables `AIOS_PRIVILEGE_STORE` and `AIOS_STATE_DIR` are inspected for `..` directory traversal patterns. If detected, paths are discarded and fall back safely to `target/privilege_state.json`.
2. **Payload Size Hardening (`load_safe_privilege_service`)**:
   - Hard upper limit of 1 MiB ($1024 \times 1024$ bytes) enforced before attempting to read and deserialize state store files from disk. Oversized payloads are quarantined/ignored, preventing memory-exhaustion DoS attacks.
3. **Identifier & Token Sanitization**:
   - Actor IDs restricted to $\le 128$ bytes with zero control characters.
   - Grant tokens restricted to $\le 256$ bytes with zero control characters.
   - Requested capability lists capped at $\le 32$ items.
4. **Kernel Tier Immutability**:
   - Direct transitions to `SystemKernel` tier strictly rejected across all MCP endpoints.
