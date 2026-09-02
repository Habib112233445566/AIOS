# T-00542 — Evidence & Audit Trail / MCP/API surface: Specification

## 1. Specification Overview
This specification formalizes the Model Context Protocol (MCP) tool registrations, JSON-RPC 2.0 schemas, parameters, responses, and audit logging contracts for Evidence & Audit Trail.

## 2. MCP Tool Specifications

### A. Evidence Verification (`aios.evidence.verify`)
- **Description**: Verifies evidence files on disk against an evidence manifest.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "manifest_path": { "type": "string" },
      "repo_path": { "type": "string" }
    }
  }
  ```
- **Response Schema**:
  ```json
  {
    "ok": true,
    "tool": "aios.evidence.verify",
    "report": {
      "total_records": 10,
      "valid_records": 10,
      "missing_files": [],
      "hash_mismatches": [],
      "is_valid": true
    }
  }
  ```

### B. SHA-256 Checksum Computation (`aios.evidence.hash`)
- **Description**: Computes the SHA-256 checksum for a specific file within bounded size limits.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "file_path": { "type": "string" }
    },
    "required": ["file_path"]
  }
  ```
- **Response Schema**:
  ```json
  {
    "ok": true,
    "tool": "aios.evidence.hash",
    "file_path": "docs/README.md",
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
  ```

### C. Evidence Directory Scan (`aios.evidence.scan`)
- **Description**: Discovers and indexes evidence files in `docs/tasks/evidence/` with optional task ID filtering.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "repo_path": { "type": "string" },
      "task_id": { "type": "integer" }
    }
  }
  ```
- **Response Schema**:
  ```json
  {
    "ok": true,
    "tool": "aios.evidence.scan",
    "records": [
      {
        "task_id": 501,
        "file_path": "docs/tasks/evidence/T-00501-recovery-validation-research.md",
        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
      }
    ]
  }
  ```

## 3. Protocol & Security Guarantees
- **Read-Only Invariance**: All three tools are unauthenticated query actions and do not mutate repository state.
- **Size Bounds**: Files evaluated during hash computation are hard-capped at 16 MiB (`MAX_DOC_BYTES`).
- **Audit Persistence**: Every execution emits a structured JSON-RPC outcome row into SQLite WAL.
