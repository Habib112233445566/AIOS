# Specification: Sandbox Enforcement Recovery & Validation Subsystem

## 1. Overview
The Sandbox Enforcement Recovery & Validation Subsystem (`SandboxRecoveryManager`) diagnoses profile inconsistencies, detects corrupted JSON manifests on disk, isolates damaged configuration files into non-destructive quarantine directories, and restores clean factory profiles (`standard`, `strict`, `permissive`).

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `SANDBOXRECV1` | **Factory Baseline Mandate** | The system requires all 3 immutable factory profiles (`standard`, `strict`, `permissive`) to be present and structurally valid. |
| `SANDBOXRECV2` | **Pre-flight Integrity Sweep** | Validation inspects in-memory registries and custom profile directories without mutating disk or system state. |
| `SANDBOXRECV3` | **Bounded Traversal & Memory** | Scanned files are capped at `MAX_SCANNED_PROFILES` (256 files) and file sizes at `MAX_PROFILE_FILE_BYTES` (64 KiB). Directory traversal (`..`) is rejected. |
| `SANDBOXRECV4` | **Non-Destructive Quarantine** | When executing `QuarantineAndReset`, corrupt files are moved into a timestamped directory `.quarantine_<timestamp>` preserving forensic evidence. |
| `SANDBOXRECV5` | **Diagnostic Granularity** | Each detected issue carries a profile name, structured severity (`Error` vs `Warning`), error code, and human-readable message. |
| `SANDBOXRECV6` | **Audit Provenance** | Recovery executions emit classified audit entries via `dispatch::recorded_call` and `classify_and_emit`. |

---

## 3. Interfaces & Usage

### CLI Commands
```bash
# Validate sandbox profile state
aiosh sandbox validate

# Validate sandbox profile state with custom profile directory
aiosh sandbox validate --dir /var/lib/aiosh/custom_profiles --json

# Restore factory defaults
aiosh sandbox recover --strategy defaults

# Quarantine corrupt files and restore defaults
aiosh sandbox recover --strategy quarantine --dir /var/lib/aiosh/custom_profiles --json

# Run dry-run recovery check
aiosh sandbox recover --strategy dry_run
```

### MCP Tool Interface
- Tool: `aios.sandbox.validate`
  - Input: `{"custom_dir": "<optional-path>"}`
  - Output: `{"ok": true, "report": {...}}`
- Tool: `aios.sandbox.recover`
  - Input: `{"strategy": "defaults"|"quarantine"|"dry_run", "custom_dir": "<optional-path>"}`
  - Output: `{"ok": true, "result": {...}}`

---

## 4. Error Codes
- `SANDBOXRECV_ERR_IO`: Filesystem IO error during file read or directory traversal.
- `SANDBOXRECV_ERR_TRAVERSAL`: Relative path components (`..`) detected in profile path.
- `SANDBOXRECV_ERR_CORRUPT`: Corrupted JSON or oversized file (> 64 KiB).
- `SANDBOXRECV_ERR_MISSING_FACTORY`: Critical baseline profile missing.
- `SANDBOXRECV_ERR_LIMIT_BOUNDS`: Negative or conflicting boundary parameters in profile.

---

## 5. Linked Task Evidence
- Unit Testing: `docs/tasks/evidence/T-02495-recovery-validation-unit-test.md`
- Integration: `docs/tasks/evidence/T-02496-recovery-validation-integration.md`
- Security Review: `docs/tasks/evidence/T-02497-recovery-validation-security-review.md`
- Hardening: `docs/tasks/evidence/T-02498-recovery-validation-hardening.md`
