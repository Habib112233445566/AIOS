# T-02031: Research — Capability Model MCP/API Surface

See full research documentation at [T-02031-mcp-api-surface-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02031-mcp-api-surface-research.md).

- **Facts**: MCP is the sole external tool invocation protocol; `CapabilityService` provides complete backend capabilities; all tools route via `dispatch::recorded_call`.
- **Assumptions**: Canonical store path is `.aios/capability_store.json`; 7 tools provide full lifecycle management.
- **Decisions**: Use `load_or_create` on `CapabilityService`; apply strict input hygiene on all arguments; partition tools into read-only vs consequential for PEP/Audit.
