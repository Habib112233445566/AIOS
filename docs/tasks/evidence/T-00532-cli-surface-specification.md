# T-00532 — Evidence & Audit Trail / CLI surface: Specification

## 1. Specification Overview
This specification formalizes the command-line interface syntax, argument parsing rules, exit codes, output envelopes, and audit trail effects for `aiosh evidence`.

## 2. CLI Command Specifications

### A. Manifest Verification (`aiosh evidence verify`)
```text
aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]
```
- **Arguments / Flags**:
  - `--repo <path>`: Base repository directory path (default: `"."`).
  - `--manifest <path>`: Path to a JSON `TaskEvidenceManifest` file.
  - `--json`: Machine-readable output format.
- **Exit Codes**:
  - `0`: All evidence files exist and match their expected SHA-256 hashes (`report.is_valid == true`).
  - `1`: One or more files are missing or have checksum mismatches (`report.is_valid == false`).
  - `2`: Invalid arguments or unreadable manifest file.
- **Output Envelopes**:
  - **Text Mode (Exit 0)**:
    ```text
    [+] All 10 evidence records verified successfully (SHA-256 match).
    ```
  - **Text Mode (Exit 1)**:
    ```text
    [-] Evidence verification failed: 8/10 valid.
        - Missing: docs/tasks/evidence/T-00530-verify.md
        - Mismatch: docs/tasks/evidence/T-00524-impl.md (expected ..., found ...)
    ```
  - **JSON Mode (Exit 0)**:
    ```json
    {
      "ok": true,
      "subcommand": "evidence verify",
      "data": {
        "total_records": 10,
        "valid_records": 10,
        "missing_files": [],
        "hash_mismatches": [],
        "is_valid": true
      }
    }
    ```
  - **JSON Mode (Exit 1)**:
    ```json
    {
      "ok": false,
      "subcommand": "evidence verify",
      "error": "Evidence verification failed",
      "report": {
        "total_records": 10,
        "valid_records": 8,
        "missing_files": ["docs/tasks/evidence/T-00530-verify.md"],
        "hash_mismatches": ["docs/tasks/evidence/T-00524-impl.md: expected ..., found ..."],
        "is_valid": false
      }
    }
    ```

### B. SHA-256 Checksum (`aiosh evidence hash`)
```text
aiosh evidence hash <path> [--json]
```
- **Arguments / Flags**:
  - `<path>`: Positional argument specifying file path to hash.
  - `--json`: Machine-readable output format.
- **Exit Codes**:
  - `0`: Checksum computed successfully.
  - `1`: File not found, unreadable, or exceeds 16 MiB size limit.
  - `2`: Missing `<path>` positional argument.
- **Output Envelopes**:
  - **Text Mode**: `[+] docs/README.md -> e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
  - **JSON Mode**: `{"ok": true, "subcommand": "evidence hash", "path": "docs/README.md", "sha256": "e3b0c442..."}`

### C. Evidence Directory Scan (`aiosh evidence scan`)
```text
aiosh evidence scan [--repo <path>] [--task <id>] [--json]
```
- **Arguments / Flags**:
  - `--repo <path>`: Base repository path.
  - `--task <id>`: Optional filter for specific task ID.
  - `--json`: Machine-readable output format.
- **Exit Codes**:
  - `0`: Scan completed successfully.
  - `1`: Directory read error.
  - `2`: Invalid flag usage.

## 3. Audit Logging Effects
Every CLI invocation logs a row to SQLite WAL via `aiosh-core::audit::AuditRing`:
- `tool`: `"evidence.verify"`, `"evidence.hash"`, or `"evidence.scan"`.
- `command`: Full CLI invocation string.
- `args_json`: Parsed flags and target parameters.
- `outcome`: `"ok"`, `"failure"`, or `"error"`.
