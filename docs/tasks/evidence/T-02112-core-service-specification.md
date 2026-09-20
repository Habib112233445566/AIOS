# Core Service Specification: T-02112

- **Task**: T-02112 (PEP Decision Engine / core service: Specification)
- **Subsystem**: PEP Decision Engine Core Service
- **Types Specified**:
  - `PepDecisionService`: Authoritative rule repository and evaluation engine.
  - Multi-index structures (`by_subject`, `by_action`).
  - Constants: `MAX_RULES_IN_SERVICE = 5000`, `MAX_PEP_SERVICE_STORE_SIZE = 10MB`.
  - Error codes: `PEPSERV_ERR_CAPACITY`, `PEPSERV_ERR_DUPLICATE_ID`, `PEPSERV_ERR_IO`, `PEPSERV_ERR_VALIDATION`.
- **Status**: Formally specified.
