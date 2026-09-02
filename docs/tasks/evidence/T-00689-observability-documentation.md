# T-00689 — Repository Health / observability: Documentation

## Operator & Agent Observability Guide
The Repository Health subsystem provides structured health diagnostics with microsecond timing and categorized pass/warn/fail/skip aggregations.

### CLI Invocation
```bash
aiosh repo health
```
To receive structured JSON output suitable for automated processing and telemetry forwarding:
```bash
aiosh repo health --json
```

### Output Schema Example
```json
{
  "repo_path": "/path/to/repo",
  "timestamp_utc": "2026-08-29T14:15:00Z",
  "overall_status": "Pass",
  "total_checks": 3,
  "passed_checks": 3,
  "warn_checks": 0,
  "failed_checks": 0,
  "skipped_checks": 0,
  "checks": [
    {
      "check_id": "git_working_tree",
      "name": "Git Working Tree Cleanliness",
      "category": "GitHygiene",
      "status": "Pass",
      "message": "Working tree is clean",
      "details": null,
      "duration_ms": 12
    }
  ]
}
```

### Constraints & Limitations
- Timing reflects execution duration of health evaluation routines on the host.
- Detailed git change diffs in output details are clamped to 50 entries to bound payload size.
