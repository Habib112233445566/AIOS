# T-00632 — Repository Health / CLI surface: Specification

## 1. Specification Overview
The `aiosh repo` command group exposes diagnostic health verification routines from `aiosh-core::repo_health_service` over the command-line interface.

## 2. Command Grammar & Options

### 2.1 Synopsis
```bash
aiosh repo <health|check> [--repo <path>] [--json]
```

### 2.2 Subcommands
- `health`: Executes repository health checks across all diagnostic categories and outputs an aggregate assessment.
- `check`: Canonical alias for `health`.

### 2.3 Options & Flags
- `--repo <path>`: Explicit target path for the repository root (default: current working directory `.`).
- `--json`: Formats output as indented JSON (`RepoHealthReport`).

## 3. Output Formats

### 3.1 Human-Readable Format (Default)
```text
=== Repository Health Assessment: /workspace ===
Timestamp: 2026-08-29T12:00:00Z
Overall Status: Pass (3 checks: 3 pass, 0 warn, 0 fail, 0 skip)

[+] Git Working Tree Cleanliness (git_working_tree, GitHygiene) - 14ms
    Working tree is clean
[+] File Size Bounds Integrity (file_bounds, FileIntegrity) - 8ms
    All monitored files are within size limit (16777216 bytes)
[+] Security Governance Policy Verification (security_governance, SecurityGovernance) - 1ms
    SECURITY.md exists and meets governance policy requirements
```

### 3.2 Machine-Readable JSON Format (`--json`)
```json
{
  "ok": true,
  "subcommand": "repo health",
  "data": {
    "repo_path": "/workspace",
    "timestamp_utc": "2026-08-29T12:00:00Z",
    "overall_status": "Pass",
    "total_checks": 3,
    "passed_checks": 3,
    "warn_checks": 0,
    "failed_checks": 0,
    "skipped_checks": 0,
    "checks": [ ... ]
  }
}
```

## 4. Exit Code Contract
- `0`: Overall status is `Pass` or `Warn`.
- `1`: Overall status is `Fail` or internal error encountered.
- `2`: Invalid command syntax or unknown option tokens.

## 5. Audit Row Contract
- `tool`: `"repo.health"`
- `command`: `"aiosh repo health"`
- `args`: `{"repo": "<path>", "json": <bool>}`
- `outcome`: `"ok"` (if checks executed) or `"error"` (if execution failed).
