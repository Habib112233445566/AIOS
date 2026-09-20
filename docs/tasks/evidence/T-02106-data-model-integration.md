# Integration Details: T-02106

- **Task**: T-02106 (PEP Decision Engine / data model: Integration)
- **Subsystem**: PEP Decision Engine Data Model
- **Integration Layer**: `aiosh-mcp` JSON-RPC Protocol
- **Endpoint**: `aios.pep.evaluate`
  - Parameters: `subject`, `resource`, `action`, `algorithm` (optional), `rules` (optional), `grant_id` (optional).
  - Invokes `aiosh_core::pep_decision::evaluate_rules`.
- **Smoke Test Results**:
```
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
=== All PEP Decision Engine smoke tests passed ===
```
