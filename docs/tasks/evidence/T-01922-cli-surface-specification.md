# Task Evidence: T-01922 - System Update Mechanism / CLI surface: Specification

## 1. Overview
- **Task ID**: `T-01922`
- **Sub-Epic**: 3 (System Update Mechanism CLI Surface)
- **Goal**: Formally specify CLI syntax, subcommands, flags, exit codes, and JSON schemas for `aiosh update`.

---

## 2. Formal Specification

### 2.1 Command Grammar
```
aiosh (update|upd) <subcommand> [flags] [args]
```

### 2.2 Subcommands
| Subcommand | Arguments | Description |
|---|---|---|
| `status` | None | Returns active update status (state, current_version, target_version, active_slot, progress). |
| `slots` | None | Returns dual partition slot status (`current_slot`, `target_slot`, `rollback_slot`, versions). |
| `check` | `<manifest_path>` | Validates manifest file and initiates download/staging. |
| `apply` | None | Validates staged update and toggles boot slot to inactive partition. |
| `confirm` | `[version]` | Confirms running slot after reboot and marks update successful. |
| `rollback` | None | Reverts boot slot to `rollback_slot`. |

### 2.3 Global & Subcommand Flags
- `--json`: Format output as standard envelope `{"code": i32, "data": Any, "error": Any}`.
- `--state-dir <PATH>`: Directory containing `slot_status.json` and `update_status.json`.
- `--staging-dir <PATH>`: Directory for staging artifact payloads.
- `--version <VER>`: Override running version for testing / initialization (default: `1.0.0`).
- `--slot <SLOT>`: Override active slot for testing (`slot_a` or `slot_b`).
- `--help`, `-h`: Display help message.

### 2.4 Exit Code Contracts
- `0`: Success.
- `1`: Domain validation or operational error (manifest invalid, hash mismatch, state transition blocked).
- `2`: Command line usage error, missing arguments, or path hygiene failure (> 1024 chars or control chars).

### 2.5 JSON Envelope Schema
- **Success (`code: 0`)**:
  ```json
  {
    "code": 0,
    "data": { ... },
    "error": null
  }
  ```
- **Error (`code: 1` or `2`)**:
  ```json
  {
    "code": 1,
    "data": null,
    "error": {
      "code": "UPD_VALIDATION_ERROR",
      "message": "manifest file does not exist"
    }
  }
  ```
