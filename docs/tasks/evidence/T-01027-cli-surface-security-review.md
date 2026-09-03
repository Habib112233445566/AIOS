# T-01027 — Distro Selection & Justification / CLI Surface: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-CLI-1: Flag/Argument Confusion & Injection
- **Threat**: Attackers supply flag-prefixed strings (e.g., `--inject`, `--json`) where a profile ID is expected, attempting to trigger unintended behavior or bypass validation.
- **Mitigation & Verification**: The argument parser in `cmd_distro` checks `!id_str.starts_with("--")`. Any flag-prefixed token passed in place of a required ID is rejected immediately with exit code `2`.

### AS-CLI-2: Arbitrary Store Path Traversal
- **Threat**: Passing relative traversal paths (`--store ../../etc/passwd`) to read or overwrite critical files.
- **Mitigation & Verification**: `DistroStore::load_from_path` strictly requires valid JSON adhering to the `DistroStore` schema. Arbitrary text/system files fail JSON deserialization cleanly with exit code `1` and emit an audit event.

### AS-CLI-3: Non-UTF-8 Argv Crash Attack
- **Threat**: Adversary passes malformed non-UTF-8 byte sequences in command-line arguments to trigger unhandled panics in Rust strings.
- **Mitigation & Verification**: `std::env::args_os()` is sanitized with `.to_string_lossy().into_owned()`, replacing invalid sequences with the Unicode replacement character (`U+FFFD`) and preventing aborts.

### AS-CLI-4: Audit Evasion Check
- **Threat**: Invoking CLI subcommands without creating an immutable entry in the audit database.
- **Mitigation & Verification**: In `cmd_distro`, each branch (`list`, `show`, `evaluate`, `recommend`) invokes `classify_and_emit`. Verified in live audit tail inspections.

## 2. Invariant Checklist
- [x] Input sanitization and bounds enforcement verified.
- [x] Audit row emission verified for every subcommand.
- [x] No open policy bypass remains.
