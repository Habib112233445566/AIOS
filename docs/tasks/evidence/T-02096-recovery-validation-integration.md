# Integration Details: T-02096

- **Task**: T-02096 (recovery & validation: Integration)
- **Subsystem**: Capability Model Recovery & Validation
- **Integration Layer**: `aiosh-mcp` JSON-RPC Protocol
- **Tools Added**:
  1. `aios.capability.recover`:
     - Parameters: `store_path` (optional string), `grant_id` (optional string)
     - Implements fail-safe self-healing via `recover_capability_store`
  2. `aios.capability.validate`:
     - Parameters: `store_path` (optional string), `grant_id` (optional string)
     - Implements read-only deep validation via `validate_capability_store`
- **Smoke Test Results**:
```
=== Capability Recovery & Validation MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: capability recovery and validation ... OK
=== All Capability Recovery smoke tests passed ===
```
- **Invariants Enforced**: `CAPREC1..CAPREC6`.
