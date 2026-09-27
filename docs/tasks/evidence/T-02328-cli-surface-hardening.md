# T-02328: Audit Chain Extensions CLI Surface Hardening

## Overview
This task documents defensive hardening, input validation guards, and error envelopes implemented for the CLI surface of Audit Chain Extensions.

## Hardening Controls

### 1. Standard Error Envelope & Exit Codes
- **Usage Refusals**: Missing positional `<hash>` arguments in `ancestry`, `sign-verify`, or `inspect` emit clean usage instructions to stderr and exit with POSIX code `2`.
- **Structured Error Envelope**: Any service failure produces a standard JSON result envelope via `err_out(json!({"ok": false, "subcommand": "...", "outcome": "error", "audit_id": -1, "error": e}))` and exits with code `1`.
- **Not Found Handling**: `inspect` handles nonexistent event hashes explicitly with `"outcome": "not_found"` instead of crashing or panicking.

### 2. Argument Clamping & Safe Defaults
- `query --limit`: Clamped between 1 and 1,000 (default 50).
- `ancestry --depth`: Default 16, clamped to maximum 64.
- Safe string parsing: Numeric parsing handles invalid, negative, or overflow values gracefully via `.and_then(|s| s.parse().ok())`.

### 3. Resource Management & Connection Hygiene
- Connection handles are transferred across service boundaries via `into_ring()` avoiding file lock contention or leakages on Windows and Linux platforms.
- Zero child process spawning or external shell execution; operations run purely in-process via compiled Rust services.
