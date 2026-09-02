# T-00762 — Secrets & Access Hygiene / automated tests: Specification

## 1. Automated Test Suite Specification (K1..K7)

| Criteria | Target Component | Verifications |
|---|---|---|
| **K1** | `aiosh-core::secrets` | Struct integrity, serde serialization, discrete severity order, UTF-8 safe redaction. |
| **K2** | `aiosh-core::secrets_service` | Private key detection (`SEC-001`), null-byte binary file skipping. |
| **K3** | `aiosh-core::secrets_service` | AWS Access Key ID (`SEC-002`) and GitHub PAT (`SEC-003`) detection and redaction. |
| **K4** | `aiosh-core::secrets_service` | Configuration password assignment (`SEC-005`) and generic API token detection (`SEC-004`). |
| **K5** | `aiosh-cli::cmd_secrets` | `aiosh secrets scan` and `aiosh secrets check` with `--file`, `--repo`, `--config`, and `--json`. |
| **K6** | `aiosh-mcp::main` | `aios.secrets.scan` and `aios.secrets.check` JSON-RPC 2.0 schemas and tool execution. |
| **K7** | `aiosh-core::secrets_config` | Schema validation, default bounds, and JSON serialization roundtrips. |

## 2. Invariant & Exit Code Contracts
- **Clean Execution**: Exit 0 when all criteria K1..K7 pass.
- **Fail-Fast Detection**: Exit 1 and print failing criteria to stderr upon any test regression.
- **Independence**: Test suite runner requires zero third-party Python libraries (Python 3 stdlib only).
