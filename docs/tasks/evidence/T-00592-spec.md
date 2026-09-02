# T-00592 — Evidence & Audit Trail / documentation: Specification

## 1. Specification Overview
This specification formalizes the operator and agent documentation structure, CLI and MCP interfaces, configuration schemas, and invariant boundaries for Evidence & Audit Trail in AIOS.

## 2. Interface References & Specifications

### A. CLI Commands Reference (`aiosh-cli`)
- **`aiosh evidence hash <path> [--json]`**:
  - Computes and displays the SHA-256 hex checksum of a file up to 16 MiB.
- **`aiosh evidence verify [--json]`**:
  - Validates all records in the evidence manifest against disk state and computes `EvidenceTelemetry`.
  - Exit code: `0` if all artifacts exist with matching hashes; `1` if mismatches or missing files are detected.
- **`aiosh evidence scan [--task <id>] [--json]`**:
  - Discovers and parses evidence markdown files in `docs/tasks/evidence/`, with optional task filtering.

### B. MCP Tools Reference (`aiosh-mcp`)
- **`aios.evidence.hash`**:
  - Arguments: `file_path` (string, required).
  - Returns: `{ "file_path": string, "sha256": string }`.
- **`aios.evidence.verify`**:
  - Arguments: `repo_path` (string, default: `"."`).
  - Returns: `{ "report": EvidenceVerificationReport, "telemetry": EvidenceTelemetry, "status": string }`.
- **`aios.evidence.scan`**:
  - Arguments: `repo_path` (string, default: `"."`), `task_id` (integer, optional).
  - Returns: `{ "records": Vec<EvidenceRecord>, "count": usize }`.

### C. Configuration Schema (`EvidenceConfig`)
```json
{
  "evidence_dir": "docs/tasks/evidence",
  "max_file_bytes": 16777216,
  "allowed_extensions": [".md", ".json"],
  "enforce_checksum": true
}
```
- **Precedence**: `AIOS_EVIDENCE_CONFIG_PATH` > `AIOS_EVIDENCE_DIR` / `AIOS_EVIDENCE_MAX_FILE_BYTES` > `config/evidence.config.json` > in-memory defaults.

### D. Security & Hardening Invariants
- File ingestion ceiling: 16 MiB per artifact.
- Configuration file ceiling: 64 KiB.
- Repository root containment: Out-of-tree traversal sequences (`..`) fail validation and are rejected as security errors.
- PEP token gating: Mutating operations (`aios.evidence.record`, `evidence.record`, `aios.evidence.set`, `evidence.set`) require active verified PEP grant tokens.
