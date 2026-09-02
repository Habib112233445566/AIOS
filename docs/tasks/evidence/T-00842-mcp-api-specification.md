# T-00842 — Regression Triage / MCP/API: Specification

## 1. Tool Schemas & Contracts

### `aios.triage.list`
- **Description**: List triage records with optional status/severity filtering.
- **Parameters**: `status` (optional string), `severity` (optional string), `store_path` (optional string).
- **Result**: `{"ok": true, "tool": "aios.triage.list", "count": usize, "records": [TriageRecord]}`.

### `aios.triage.show`
- **Description**: Show detailed information for a single regression record.
- **Parameters**: `id` (required string), `store_path` (optional string).
- **Result**: `{"ok": true, "tool": "aios.triage.show", "record": TriageRecord}`.

### `aios.triage.record`
- **Description**: Record a test regression finding into the store.
- **Parameters**: `test_target` (required string), `suite_name` (optional string), `error_message` (required string), `repro_command` (optional string), `severity` (optional string), `store_path` (optional string).
- **Result**: `{"ok": true, "tool": "aios.triage.record", "record": TriageRecord}`.

### `aios.triage.resolve`
- **Description**: Resolve a triage record with resolution notes.
- **Parameters**: `id` (required string), `notes` (required string), `store_path` (optional string).
- **Result**: `{"ok": true, "tool": "aios.triage.resolve", "record": TriageRecord}`.

### `aios.triage.check`
- **Description**: Check store health for unresolved blocker/critical regressions.
- **Parameters**: `store_path` (optional string).
- **Result**: `{"ok": true, "tool": "aios.triage.check", "clean": bool, "total_records": u32, "open_records": u32, "blocker_open": u32, "critical_open": u32}`.

## 2. Invariant & Audit Policy
- Every execution emits a structured AuditRow to SQLite WAL.
- Storage updates are committed synchronously to disk.
