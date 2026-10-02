# Task Evidence: T-02627 — Secrets Handling CLI Surface Security Review

## 1. Overview
Conducted comprehensive security review of the Secrets Handling CLI interface (`aiosh secret` / `aiosh sec` implemented in `code/aiosh-rust/aiosh-cli/src/main.rs`).

## 2. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: Terminal Log Scraping & Screen Capture Leakage
- **Threat Vector**: Terminal screen recording, shell history, or process monitoring tools capture secret payloads output by CLI invocations.
- **Mitigation Implemented**:
  - `aiosh secret get` defaults to masked representation (`val.masked_display()`), printing only the first 4 and last 4 characters separated by ellipses (`****...****`).
  - Unmasked plaintext requires explicit `--expose` flag.
  - `aiosh secret list` outputs `SecretMetadata` only, which contains zero payload bytes or plaintexts.
- **Status**: Verified mitigated.

### Abuse Scenario 2: Arbitrary File Overwrite & Path Traversal via `--store`
- **Threat Vector**: Malicious input targeting `--store` with `../../etc/passwd` or symbolic links pointing outside allowed directory boundaries.
- **Mitigation Implemented**:
  - CLI pre-validates `store_path` rejecting any path containing `..`.
  - `SecretService::load_from_path` checks `symlink_metadata` to reject symlinks and enforce regular file types.
  - `save_to_path` executes atomic write-and-replace (`.tmp.{pid}.{nanos}`) with auto-cleanup on failure to prevent corrupted or intercepted file states.
- **Status**: Verified mitigated.

### Abuse Scenario 3: Privilege Escalation & Cross-Scope Token Access
- **Threat Vector**: A lower-privileged caller (e.g., agent or untrusted session) requests a secret belonging to a higher or disjoint scope (e.g., `system` or another agent).
- **Mitigation Implemented**:
  - `cmd_secret` parses caller scope flags (`--scope`, `--target`) and queries `SecretService::get_secret(&id, &caller_scope)`.
  - Core service enforces `caller_scope.allows(&secret.scope)`. Unauthorized requests are rejected with `SECSVC_ERR_ACCESS_DENIED` and exit code 1.
- **Status**: Verified mitigated.

### Abuse Scenario 4: Terminal Injection / ANSI Control Sequenced Poisoning
- **Threat Vector**: Secret descriptions or error strings containing terminal escape sequences (`\x1b]0;...`) targeting operator console compromise.
- **Mitigation Implemented**:
  - All standard error outputs pass through `sanitize_terminal()`, stripping non-printable ASCII and escape sequences before emission.
- **Status**: Verified mitigated.

## 3. Audit Logging & Non-Repudiation
Every state mutation (`store`, `rotate`, `revoke`) and data extraction (`get`) emits a structured audit event through `classify_and_emit` to the security audit ring, capturing caller, action, resource ID, and exposure flag.

## 4. Conclusion
Security review completed with 0 policy bypasses and 0 open vulnerabilities. Ready for hardening in T-02628.
