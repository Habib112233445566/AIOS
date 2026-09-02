# T-00732 — Secrets & Access Hygiene / CLI surface: Specification

## 1. Command Syntax & Interface
```bash
aiosh secrets <scan|check> [--repo <path>] [--file <path>] [--json] [--max-bytes <n>]
```

### Subcommands
- **`scan`**: Performs a comprehensive secret scan across the workspace or specified target file, printing detailed finding cards (with redacted snippets and fingerprints) and returning exit code `0` on clean scans, `1` if secrets are found.
- **`check`**: Fast verification mode designed for CI/pre-commit gates. Prints summary pass/fail lines and returns exit code `0` if clean, `1` if findings exist.

### Flags & Options
- `--repo <path>`: Root workspace directory to scan. Defaults to `.`.
- `--file <path>`: Specific file path to scan in isolation.
- `--max-bytes <n>`: Maximum file size to scan in bytes (default: `16777216` = 16 MiB).
- `--json`: Format output as JSON envelope `{ "ok": bool, "subcommand": "secrets <action>", "data": SecretScanReport }`.

## 2. Output Specifications

### Human-Readable Prose (scan)
```text
=== Secrets & Access Hygiene Scan: . ===
Timestamp: 2026-08-31T04:00:00Z
Status: FINDINGS DETECTED
Files Scanned: 42 | Total Findings: 2
  - Critical: 1 | High: 1 | Medium: 0 | Low: 0 | Info: 0

Findings:
  - [!] SEC-001 (Critical) src/id_rsa:1 - Private Key block detected
      Snippet: -----BEGIN RSA****KEY----- [fp: 9f86d081]
  - [!] SEC-003 (High) .env:12 - GitHub Personal Access Token detected
      Snippet: ghp_1234****5678 [fp: 5e884898]
```

### Machine-Readable JSON (`--json`)
```json
{
  "ok": true,
  "subcommand": "secrets scan",
  "data": {
    "repo_path": ".",
    "timestamp_utc": "2026-08-31T04:00:00Z",
    "scanned_files_count": 42,
    "is_clean": false,
    "total_findings": 2,
    "critical_findings": 1,
    "high_findings": 1,
    "medium_findings": 0,
    "low_findings": 0,
    "info_findings": 0,
    "findings": [
      {
        "rule_id": "SEC-001",
        "path": "src/id_rsa",
        "line_number": 1,
        "severity": "Critical",
        "pattern_kind": "PrivateKey",
        "description": "Private Key block detected",
        "redacted_snippet": "-----BEGIN RSA****KEY-----",
        "fingerprint": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
      }
    ]
  }
}
```

## 3. Exit Codes
- `0`: Scan completed cleanly with zero findings, or successful check.
- `1`: Findings detected during `scan`/`check` or runtime scanning error.
- `2`: Invalid CLI arguments or syntax error.

## 4. Audit Trail Integration
Every execution calls `AuditRing::write` emitting:
- `tool`: `"secrets.scan"`
- `command`: `"aiosh secrets <args>"`
- `outcome`: `"ok"` (clean) / `"failure"` (findings detected) / `"error"` (runtime error)
