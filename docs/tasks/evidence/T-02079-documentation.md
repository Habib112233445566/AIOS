# Documentation Summary: T-02079 (observability: Documentation)

- **Target File**: `docs/capability_model.md` (Section 13)
- **Topics Documented**:
  - `CAPOBS1..CAPOBS6` observability invariants.
  - Complete capability state breakdown (active, revoked, expired, root, attenuated).
  - Algorithmic guarantees ($O(N)$ memoization, cycle handling).
  - Quota aggregation using saturating arithmetic.
  - Distribution metrics (scopes, rights, unique subjects/issuers).
  - Health evaluation criteria.
  - MCP tool `aios.capability.observability` request and response contract.
