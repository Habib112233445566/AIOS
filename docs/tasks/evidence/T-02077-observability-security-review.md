# Security Review: Capability Model Observability (T-02077)

## Executive Summary
This security review evaluates the **Capability Observability** subsystem (`CAPOBS1..CAPOBS6`) implemented in `code/aiosh-rust/aiosh-core/src/capability_observability.rs` and exposed via MCP tool `aios.capability.observability` in `aiosh-mcp`.

---

## 1. Threat Model & Abuse Scenarios

### Scenario 1: Redundant Depth Calculation / Denial of Service (`THREAT-CAPOBS-01`)
- **Attack Vector**: In a large registry with many capabilities in deep derivation chains, invoking `generate()` computes `get_derivation_depth()` for every capability. Without caching, this could result in $O(N \times D)$ node lookups.
- **Analysis**:
  - `get_derivation_depth()` is cycle-safe and bounded to 256 iterations.
  - However, in `generate()`, caching/memoizing computed depths in a local `HashMap<String, usize>` guarantees each node's depth is calculated at most once, reducing worst-case complexity to $O(N)$.
- **Hardening Requirement (for T-02078)**:
  - Implement depth memoization during `generate()` in `capability_observability.rs`.

### Scenario 2: Information Disclosure via Telemetry (`THREAT-CAPOBS-02`)
- **Attack Vector**: An unauthorized or low-privileged agent inspects `aios.capability.observability` to steal cryptographic tokens, unforgeable capability IDs, or sensitive parameter values.
- **Analysis**:
  - `CapabilityObservabilityReport` contains strictly aggregate metrics: total counts, active/revoked/expired counts, depth statistics, scope type distributions, and rights distributions.
  - Raw capability IDs, tokens, paths, and subject tokens are never leaked in the report.
- **Verdict**: Satisfied. No secret material is present in telemetry structures.

### Scenario 3: Store Path Traversal & Unauthorized Store Ingestion (`THREAT-CAPOBS-03`)
- **Attack Vector**: An attacker supplies a malicious `store_path` (e.g., `../../../../etc/shadow` or `/dev/urandom`) to `aios.capability.observability`.
- **Analysis**:
  - `CapabilityService::load_or_create()` invokes `validate_service_path(path)`.
  - `validate_service_path` strictly enforces:
    - Path length $\le 1024$.
    - Prohibition of control characters.
    - Prohibition of parent directory traversal (`..`).
    - Mandatory `.json` file extension.
    - Symlink rejection via `symlink_metadata`.
    - Maximum file size cap (`MAX_CAPABILITY_STORE_SIZE = 10 MB`).
- **Verdict**: Fail-closed protection is enforced prior to file read.

### Scenario 4: Log Injection & Telemetry Corruption (`THREAT-CAPOBS-04`)
- **Attack Vector**: Malicious subjects or custom timestamp strings contain ANSI terminal escape codes, carriage returns (`\r`), or control characters to overwrite terminal logs or forge audit entries.
- **Analysis**:
  - `sanitize_telemetry_text()` filters out all control characters (`!c.is_control()`) and caps length to 256 characters.
- **Hardening Requirement (for T-02078)**:
  - Ensure string boundaries and whitespace trimming are preserved across all report string fields.

### Scenario 5: PEP Gating & Audit Logging (`THREAT-CAPOBS-05`)
- **Attack Vector**: Invoking observability tools could bypass audit trails.
- **Analysis**:
  - `aios.capability.observability` executes via `dispatch::recorded_call`, writing an immutable event to the SHA-256 hash-chained audit ring for every query.

---

## 2. Review Conclusion
- **Status**: Review completed. Abuse scenarios analyzed.
- **Action Items**: Hardening tasks defined for `T-02078` (depth memoization during report generation, verification of path hygiene).
