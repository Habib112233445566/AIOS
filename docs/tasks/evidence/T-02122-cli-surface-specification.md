# T-02122: CLI Surface Specification — PEP Decision Engine

## Overview
- **Task ID**: `T-02122`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## 1. CLI Commands & Arguments

### 1.1 `aiosh pep evaluate`
Evaluates an authorization request against the policy rules in the store.
- **Arguments**:
  - `--subject <STRING>` (required): Subject identifier (e.g. `agent:researcher`).
  - `--action <STRING>` (required): Action requested (e.g. `read`, `write`, `execute`).
  - `--resource <STRING>` (required): Resource URI (e.g. `fs:/data/stats.json`).
  - `--algorithm <ALG>` (optional): Combining algorithm: `deny_overrides` (default), `permit_overrides`, `first_applicable`.
  - `--store <PATH>` (optional): Path to policy JSON store.
  - `--json` (optional): Output JSON envelope.
- **Exit Codes**:
  - `0`: Decision is `Permit` (`allowed == true`).
  - `1`: Decision is `Deny` (`allowed == false`).
  - `2`: Missing required flags, invalid syntax, or path traversal detected.

### 1.2 `aiosh pep rule-add`
Adds a new policy rule to the store.
- **Arguments**:
  - `--id <STRING>` (required): Unique rule ID (alphanumeric, `_`, `-`, $\le 128$ chars).
  - `--subject <STRING>` (required): Target subject or glob pattern.
  - `--action <STRING>` (required): Target action or `*`.
  - `--resource <STRING>` (required): Target resource URI or glob pattern.
  - `--effect <permit|deny>` (required): Rule effect.
  - `--priority <INT>` (optional): Integer priority (default: `0`).
  - `--desc <STRING>` (optional): Human-readable description.
  - `--store <PATH>` (optional): Path to policy store.
  - `--json` (optional): Output JSON envelope.
- **Exit Codes**:
  - `0`: Rule successfully added and persisted.
  - `1`: Capacity exceeded or persistence failure.
  - `2`: Validation error (invalid ID, missing arguments, invalid effect, path traversal).

### 1.3 `aiosh pep rule-list`
Lists policy rules currently loaded in the store.
- **Arguments**:
  - `--subject <STRING>` (optional): Filter rules by target subject.
  - `--action <STRING>` (optional): Filter rules by target action.
  - `--store <PATH>` (optional): Path to policy store.
  - `--json` (optional): Output JSON envelope.
- **Exit Codes**:
  - `0`: Rules retrieved successfully.
  - `1`: Store load failure.
  - `2`: Path validation error.

### 1.4 `aiosh pep rule-remove`
Removes a rule by its ID.
- **Arguments**:
  - `<ID>` (required): Unique ID of the rule to remove.
  - `--store <PATH>` (optional): Path to policy store.
  - `--json` (optional): Output JSON envelope.
- **Exit Codes**:
  - `0`: Rule removed successfully.
  - `1`: Rule not found or persistence failure.
  - `2`: Missing ID or invalid path.

### 1.5 `aiosh pep status`
Displays decision engine statistics, rule counts, and store file location.
- **Arguments**:
  - `--store <PATH>` (optional): Path to policy store.
  - `--json` (optional): Output JSON envelope.
- **Exit Codes**:
  - `0`: Status displayed successfully.
  - `1`: Store load error.
  - `2`: Invalid arguments or path.

## 2. Standard Output Envelope (`--json`)
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```
In case of error:
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "INVALID_ARGUMENT",
    "message": "missing required flag: --resource"
  }
}
```

## 3. Audit Logging Contract
Every command invocation emits an audit row:
- `tool`: `"pep"`
- `command`: subcommand name (`"evaluate"`, `"rule-add"`, etc.)
- `outcome`: `"success"` or `"failure"`
- `target`: rule ID, resource URI, or store path
- `c_flags`: standard constitutional flags
