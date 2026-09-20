# Hardening Summary: T-02078 (observability: Hardening)

- **Target**: `code/aiosh-rust/aiosh-core/src/capability_observability.rs`
- **Vulnerabilities Mitigated**:
  - Algorithmic complexity exhaustion ($O(N \times D)$ repeated derivation traversal): mitigated via memoized depth resolution (`get_memoized_depth`) caching DAG nodes in $O(N)$.
  - Recursion cycle vulnerability: mitigated via `visiting: HashSet<String>` cycle detection and depth clamp at 256.
  - Telemetry timestamp corruption via non-printable or control-character sequences: mitigated via sanitized string validation and fallback to RFC 3339 UTC timestamps.
- **Verification**: `test_capability_observability` test suite passes.
