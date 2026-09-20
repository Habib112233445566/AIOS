# T-02034: Implementation — Capability Model MCP/API Surface

See complete implementation documentation at [T-02034-mcp-api-surface-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02034-mcp-api-surface-implementation.md).

- **Tools Implemented**:
  1. `aios.capability.list`: Lists capabilities filtered by subject or active status.
  2. `aios.capability.get`: Retrieves capability by ID with structured error if missing.
  3. `aios.capability.issue`: Issues root capability with parsed scope and rights.
  4. `aios.capability.attenuate`: Derives attenuated child capability with monotonic restrictions.
  5. `aios.capability.revoke`: Cascade revokes capability and all derived descendants.
  6. `aios.capability.check`: Fast capability access check with optional invocation consumption.
  7. `aios.capability.prune`: Prunes expired leaf capabilities.
- **Dispatch**: All tools use `dispatch::recorded_call` for PEP gating and Audit Ring recording.
