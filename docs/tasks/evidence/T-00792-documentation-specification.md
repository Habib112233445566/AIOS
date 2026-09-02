# T-00792 — Secrets & Access Hygiene / documentation: Specification

## 1. Documentation Contract & Architecture
The Secrets & Access Hygiene documentation in `docs/README.md` must provide a comprehensive, rot-proof guide with the following sections:

1. **Subsystem Architecture**:
   - `aiosh-core::secrets`: Data structures, severity models, and redaction logic.
   - `aiosh-core::secrets_service`: File-level scanners (`SEC-001`..`SEC-005`), workspace traversal, and binary file sniffing.
   - `aiosh-core::secrets_config`: `SecretsConfig` struct, schema bounds, and multi-tier precedence loading.
2. **CLI Surface**:
   - `aiosh secrets scan` and `aiosh secrets check` with options `--repo`, `--file`, `--config`, `--max-bytes`, and `--json`.
3. **MCP Tool Protocols**:
   - Tool definitions `aios.secrets.scan` and `aios.secrets.check` with JSON-RPC 2.0 schemas.
4. **Automated Testing & Observability**:
   - Test runner `tools/test_secrets_suites.py` validating criteria K1..K8.
   - `SecretScanReport::severity_counts()` and `SecretScanReport::summary_line()`.
5. **Security Policy**:
   - `SECURITY.md` rules and disclosure timeline.

## 2. Invariant Constraints
- Must satisfy all doc invariant criteria C1..C6 (`tools/check_task_docs.py`).
- Must not embed volatile or unpinned counter values.
