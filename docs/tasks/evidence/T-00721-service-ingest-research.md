# T-00721 — Secrets & Access Hygiene / service & ingest: Research

## 1. Goal
Establish facts, detection algorithms, pattern definitions, and performance constraints for the Secrets & Access Hygiene service and ingestion layer (`code/aiosh-rust/aiosh-core/src/secrets_service.rs`).

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Invariants):
1. **Core Scanning Signatures**:
   - **Private Keys**: `-----BEGIN [A-Z ]*PRIVATE KEY-----` (RSA, EC, OpenSSH, DSA, PGP, Encrypted).
   - **AWS Access Key ID**: `AKIA[0-9A-Z]{16}`.
   - **GitHub Personal Access Token**: `ghp_[0-9a-zA-Z]{36}` or `github_pat_[0-9a-zA-Z_]{82}`.
   - **Generic API Token / Bearer Key**: `(?i)(?:api[_-]?key|bearer[_-]?token|auth[_-]?token)\s*[:=]\s*["']?([a-zA-Z0-9_\-\.]{20,})["']?`.
   - **Config Password Assignment**: `(?i)(?:password|db_pass|secret_key)\s*[:=]\s*["']?([^\s"']{8,})["']?`.
2. **Performance & Memory Bounds**:
   - File Size Limit: Maximum 16 MiB per file; larger files are skipped with a warning.
   - Line Length Limit: Clamp scan lines to 4096 bytes to avoid ReDoS on minified bundles.
   - Directory Pruning: Unconditionally ignore `.git`, `target`, `node_modules`, `.venv`, and `dist`.
   - Binary File Detection: Check the first 512 bytes for null bytes (`\0`) to skip compiled binaries and media assets.
3. **Deterministic Fingerprinting**:
   - Every finding computes a SHA-256 fingerprint: `sha256(rule_id || ":" || path || ":" || line_number || ":" || matched_secret)`.

### Assumptions:
1. Streaming file inspection using standard Rust `BufRead` delivers sub-second workspace scanning times.
2. The scanner should return a strongly typed `SecretScanReport`.

## 3. Prior Art & Authoritative Sources
- **Gitleaks Rule Definitions v8.18+**: Industry standard regular expressions for token and secret identification.
- **OWASP Automated Threat Handbook (OAT-018 Snooping)**: Credential discovery and protection.
- **TruffleHog v3 Architecture**: Filesystem crawling, binary detection, and entropy thresholding.

## 4. Decisions Needed
1. Structure `secrets_service.rs` with `scan_file_for_secrets` and `scan_workspace_for_secrets`.
2. Standardize rule identifiers (`SEC-001` Private Key, `SEC-002` AWS Credentials, `SEC-003` GitHub PAT, `SEC-004` Generic Token, `SEC-005` Password in Config).
3. Ensure unredacted secrets are never surfaced in the returned report.

## 5. Next Steps
Advance to Specification (**T-00722**) to formalize service function prototypes and scanning contracts.
