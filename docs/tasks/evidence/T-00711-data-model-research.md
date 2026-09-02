# T-00711 — Secrets & Access Hygiene / data model: Research

## 1. Goal
Establish facts, constraints, data model structures, and prior art for Secrets & Access Hygiene in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Invariants):
1. **Threat Vectors & Secret Leakage Scenarios**:
   - Accidental hardcoding of private keys (SSH `id_rsa`, PEM certificates, RSA/ECDSA/Ed25519 private keys).
   - Leakage of API tokens and cloud credentials (AWS Access Key IDs `AKIA...`, GitHub PATs `ghp_...`, Bearer tokens, OpenAI/Anthropic API keys `sk-...`).
   - Uncommitted or committed `.env` files and configuration dumps containing plaintext database passwords or PEP root tokens.
   - Exposure of sensitive tokens in task evidence logs or CLI/MCP output envelopes.
2. **Data Model Requirements**:
   - `SecretFinding`: Granular discovery record detailing `rule_id`, `path`, `line_number`, `severity` (`Critical`, `High`, `Medium`, `Low`), `pattern_kind`, `fingerprint` (SHA-256 of finding), and a strictly `redacted_snippet` (preserving first 4 and last 4 characters, masking all intermediate characters with `****`).
   - `SecretScanReport`: Aggregated report capturing `repo_path`, `timestamp_utc`, `is_clean`, `total_findings`, `critical_findings`, `high_findings`, `medium_findings`, `low_findings`, `scanned_files_count`, and `findings: Vec<SecretFinding>`.
3. **Architectural & Security Invariants**:
   - Zero Plaintext Leakage: Unredacted raw secret strings must NEVER be stored in `SecretFinding` structures or emitted to SQLite WAL audit logs.
   - Bounded Scanning: File reads during secret scans must be capped at 16 MiB per file, skipping heavy build folders (`.git`, `target`, `node_modules`, `.venv`).

### Assumptions:
1. High-precision regex pattern matching combined with Shannon entropy evaluation provides reliable secret detection with minimal false positives.
2. The data model should be implemented in pure Rust (`code/aiosh-rust/aiosh-core/src/secrets.rs`) with full `serde` serialization support.

## 3. Prior Art & Authoritative Standards
- **NIST SP 800-53 Rev. 5 (IA-5, SC-28)**: Authenticator Management & Protection of Information at Rest.
- **OWASP Top 10 (A07:2021)**: Identification and Authentication Failures / Hardcoded Secrets.
- **Gitleaks / TruffleHog / Git-Secrets**: Industry standard regex signatures and entropy heuristics for credential discovery.

## 4. Decisions Needed
1. Define `SecretSeverity` enum (`Critical`, `High`, `Medium`, `Low`, `Info`).
2. Standardize `SecretFinding` and `SecretScanReport` in `aiosh-core::secrets`.
3. Provide unit test suite validating JSON serialization roundtrips and redaction invariants.

## 5. Next Steps
Advance to Specification (**T-00712**) to formally specify the data model structs and validation methods.
