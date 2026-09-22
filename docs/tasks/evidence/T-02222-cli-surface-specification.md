# Task Evidence: T-02222 (Grant Lifecycle / CLI surface: Specification)

## 1. Scope & Objective
Defines the functional and technical specification for all `aiosh pep grant` subcommands, parameter contracts, JSON envelope structures, and exit code mappings.

---

## 2. Command Specifications

### 2.1 `aiosh pep grant issue`
Issues and registers a new authorization grant.
- **Flags & Options**:
  - `--id <STRING>` (required): Unique alphanumeric grant ID ($\le 128$ chars).
  - `--subject <STRING>` (required): Target subject entity identifier.
  - `--scope-type <filesystem|network|ipc|system>` (required): Capability scope type.
  - `--rights <LIST>` (required): Comma-separated rights (`read`, `write`, `execute`, `delete`, `admin`, `delegate`).
  - `--issuer <STRING>` (optional, default: `operator`): Issuing authority identifier.
  - `--scope-path <STRING>` (optional): Resource target path or subsystem descriptor.
  - `--expires-at <RFC3339>` (optional): Grant expiration timestamp.
  - `--not-before <RFC3339>` (optional): Not-before validity timestamp.
  - `--max-invocations <INT>` (optional): Maximum allowed invocations.
  - `--max-bytes <INT>` (optional): Maximum allowed bytes processed.
  - `--delegation-depth <INT>` (optional, default: `0`): Maximum delegation depth allowed.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Grant issued and persisted successfully.
  - `1`: Capacity limit exceeded or store write error.
  - `2`: Validation error (duplicate ID, invalid right, malformed timestamp, path traversal).

### 2.2 `aiosh pep grant list`
Lists authorization grants in the store with optional filtering.
- **Flags & Options**:
  - `--subject <STRING>` (optional): Filter grants by subject.
  - `--state <requested|active|suspended|revoked|expired>` (optional): Filter by grant state.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Grants listed successfully.
  - `1`: Store read or deserialization error.
  - `2`: Validation error (invalid state filter, path traversal).

### 2.3 `aiosh pep grant inspect <grant_id>`
Displays full details, constraints, usage metrics, and revocation audit records for a grant.
- **Arguments**:
  - `<grant_id>` (required positional): Target grant identifier.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Grant found and printed.
  - `1`: Grant not found.
  - `2`: Missing positional `<grant_id>` or path traversal.

### 2.4 `aiosh pep grant validate <grant_id>`
Tests whether a grant authorizes an action at a specified timestamp.
- **Arguments**:
  - `<grant_id>` (required positional): Target grant identifier.
  - `--subject <STRING>` (optional): Verifies grant matches subject.
  - `--right <read|write|execute|delete|admin|delegate>` (optional): Verifies right confers access.
  - `--now <RFC3339>` (optional, default: current UTC time): Evaluates temporal validity against timestamp.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Grant is valid, active, and confers requested permissions.
  - `1`: Grant is invalid, expired, revoked, quota-exhausted, or does not match right/subject.
  - `2`: Missing `<grant_id>`, unrecognized right, or malformed timestamp.

### 2.5 `aiosh pep grant attenuate`
Derives a delegated child grant with restricted rights from an active parent grant.
- **Arguments**:
  - `<parent_id>` or `--parent <STRING>` (required): Active parent grant ID.
  - `--child <STRING>` (required): New unique child grant ID.
  - `--subject <STRING>` (required): Child subject entity identifier.
  - `--rights <LIST>` (required): Comma-separated child rights subset.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Attenuated child grant successfully derived and issued.
  - `1`: Attenuation failure (parent missing delegate right, right expansion, depth exhausted).
  - `2`: Missing required arguments, duplicate child ID, or validation error.

### 2.6 `aiosh pep grant revoke <grant_id>`
Revokes a grant, recording operator attribution, and optionally cascades to all child grants.
- **Arguments**:
  - `<grant_id>` (required positional): Grant ID to revoke.
  - `--reason <STRING>` (optional, default: `Revoked by operator`): Reason for audit trail.
  - `--cascade` (optional flag): Traverses child hierarchy to revoke all descendants.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Grant (and any descendants if cascaded) successfully revoked.
  - `1`: Grant not found or invalid transition from terminal state.
  - `2`: Missing `<grant_id>` or path validation error.

### 2.7 `aiosh pep grant sweep`
Evaluates all active/suspended grants and transitions expired or quota-exhausted grants to `Expired`.
- **Arguments**:
  - `--now <RFC3339>` (optional, default: current UTC time): Evaluation reference timestamp.
  - `--store <PATH>` (optional): Path to grants JSON store.
  - `--json` (optional): Formatted JSON output envelope.
- **Exit Codes**:
  - `0`: Sweep completed successfully; outputs count of swept grants.
  - `1`: Sweep operation failed.
  - `2`: Malformed timestamp or path validation error.

---

## 3. JSON Output Envelopes
Consistent standard envelope across all subcommands:
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```
Error envelope:
```json
{
  "code": 1,
  "data": null,
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "child rights must be a subset of parent rights"
  }
}
```

---

## 4. Acceptance Confirmation
- [x] All 7 subcommands (`issue`, `list`, `inspect`, `validate`, `attenuate`, `revoke`, `sweep`) specified in full.
- [x] Flags, argument types, defaults, and exit codes documented.
- [x] JSON envelope schemas defined for success and error conditions.
