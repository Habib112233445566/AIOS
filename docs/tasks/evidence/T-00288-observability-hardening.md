# T-00288 — Release Packaging & Backup: Observability Hardening

## Hardening Details

- **Size Caps (Audit Log Inflation Protection)**: The native OS subprocess stderr capture implemented in `T-00284` poses a risk if a rogue packager dumps millions of bytes of error logs to `stderr`, which would serialize into the JSON ledger and artificially bloat/exhaust storage. We hardened `run_external_packager` to strictly clamp the string at **4KB (4096 bytes)**. If the output exceeds this, it is safely truncated with an explicit `[TRUNCATED]` suffix before being stored.
- **Fail-Open Auditing**: Because the `run_external_packager` returns a native Rust `Result::Err`, the core logic inside `generate_release` and `create_backup` immediately maps this `Err` into the `outcome_detail` parameter, ensuring the failure writes exactly one honest row to the ledger.
- **No Resource Leaks**: `Command::output()` synchronously awaits the child process. It is impossible to leak child processes here. File descriptors are handled natively by the Rust standard library.

## Acceptance Validation
- **Explicit Errors**: The clamped size cap effectively balances diagnostic visibility with DOS protection.
- The task invariants are successfully upheld.
