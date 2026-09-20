# Task Evidence: T-01822 - Network Bootstrap / CLI Surface: Specification

## Metadata
- **Task ID:** `T-01822`
- **Sub-Epic:** Sub-Epic 3: Network Bootstrap / CLI Surface
- **Component:** `code/aiosh-rust/aiosh-cli/src/main.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. CLI Surface Specification: `aiosh net`

### Subcommands & Arguments

| Command | Arguments | Flags | Exit Codes | Description |
|:---|:---|:---|:---|:---|
| `aiosh net list` | None | `--json`, `--sysfs <path>` | 0 = ok, 2 = usage error | List all discovered interfaces. |
| `aiosh net show <iface>` | `<iface>` (required) | `--json`, `--sysfs <path>` | 0 = ok, 1 = not found, 2 = usage error | Show details of a specific interface. |
| `aiosh net routes` | None | `--json`, `--procfs <path>` | 0 = ok, 2 = usage error | Display IPv4 routing table. |
| `aiosh net dns` | None | `--json`, `--resolv <path>` | 0 = ok, 2 = usage error | Display DNS nameservers and search domains. |
| `aiosh net state` | None | `--json`, `--sysfs <path>`, `--procfs <path>`, `--resolv <path>` | 0 = ok, 2 = usage error | Output complete host network state snapshot. |
| `aiosh net up <iface>` | `<iface>` (required) | `--json`, `--sysfs <path>` | 0 = ok, 1 = failure, 2 = usage error | Bring interface up. |
| `aiosh net down <iface>` | `<iface>` (required) | `--json`, `--sysfs <path>` | 0 = ok, 1 = failure, 2 = usage error | Bring interface down. |

### JSON Envelope Formats

Success (`code: 0`):
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```

Error (`code: 1` or `2`):
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "MISSING_INTERFACE_NAME",
    "message": "interface name is required for 'show'"
  }
}
```

### Standard Error Codes
- `UNKNOWN_SUBCOMMAND` (code 2)
- `PATH_TOO_LONG` (code 2)
- `PATH_CONTAINS_CONTROL_CHAR` (code 2)
- `MISSING_INTERFACE_NAME` (code 2)
- `INVALID_INTERFACE_NAME` (code 2)
- `INTERFACE_NOT_FOUND` (code 1)
- `OPERATION_FAILED` (code 1)
