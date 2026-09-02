# T-00742 — Secrets & Access Hygiene / MCP/API surface: Specification

## 1. Tool Schemas (JSON-RPC 2.0)

### Tool 1: `aios.secrets.scan`
```json
{
  "name": "aios.secrets.scan",
  "description": "Scan workspace or specific file for exposed API keys, private keys, and credentials without revealing raw secrets.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "repo_path": { "type": "string", "description": "Workspace root directory (defaults to .)" },
      "file_path": { "type": "string", "description": "Specific file path to scan in isolation" },
      "max_bytes": { "type": "integer", "description": "Maximum file size in bytes to scan (default: 16777216)" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "additionalProperties": false
  }
}
```

### Tool 2: `aios.secrets.check`
```json
{
  "name": "aios.secrets.check",
  "description": "Fast boolean cleanliness check verifying that no exposed credentials exist in the target workspace.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "repo_path": { "type": "string", "description": "Workspace root directory (defaults to .)" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "additionalProperties": false
  }
}
```

## 2. Response Envelopes

### `aios.secrets.scan` Success
```json
{
  "ok": true,
  "tool": "aios.secrets.scan",
  "report": {
    "repo_path": ".",
    "timestamp_utc": "2026-08-31T04:10:00Z",
    "is_clean": true,
    "total_findings": 0,
    "critical_findings": 0,
    "high_findings": 0,
    "medium_findings": 0,
    "low_findings": 0,
    "scanned_files_count": 128,
    "findings": []
  }
}
```

### Error Path
```json
{
  "ok": false,
  "tool": "aios.secrets.scan",
  "error": "Failed to read file metadata /invalid/path: No such file or directory"
}
```

## 3. Dispatch & Audit Gate
Routed via `dispatch::recorded_call()` with:
- `tool_name`: `"aios.secrets.scan"` / `"aios.secrets.check"`
- `read_only`: `true`
- `c_flags`: Default (no C-1..C-4 flags fired)
