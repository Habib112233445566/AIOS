# T-02327: Audit Chain Extensions CLI Surface Security Review

## Threat Model & Security Posture
This security review assesses the attack surface, input validation, and audit row invariants for the CLI subcommands (`query`, `ancestry`, `sign-verify`, `inspect`).

## Abuse Scenarios & Mitigations

### 1. AS-CLI-01: Malformed Hash Argument Injection
- **Scenario**: An untrusted caller supplies shell metacharacters, SQL fragments, or excessive string lengths as the positional `<hash>` argument.
- **Mitigation**: Hashes are passed as parameters into parameterized queries without shell execution or raw string formatting. Missing hashes immediately fail with usage refusal exit code `2`.

### 2. AS-CLI-02: Output Buffer Overflow & Memory Exhaustion via `--limit`
- **Scenario**: An attacker runs `aiosh audit query --limit 999999999` to flood the terminal or exhaust process heap memory.
- **Mitigation**: In `cmd_audit_query` and underlying `AuditChainService`, limit parsing clamps between 1 and 1,000, preventing unbounded heap allocation.

### 3. AS-CLI-03: Silent Bypass of State-Changing Invariants
- **Scenario**: Audit operations execute without logging operator actions, allowing stealth reconnaissance of security event histories.
- **Mitigation**: Every CLI subcommand unconditionally emits an audit event with tool name `audit.<subcommand>` into the local SQLite WAL ring via `emit(&mut ctx, ...)`, recording actor, command line, arguments, and outcome.

### 4. AS-CLI-04: Deep Recursion Denial-of-Service via `--depth`
- **Scenario**: An operator passes `--depth 100000` to `aiosh audit ancestry` to crash the CLI stack.
- **Mitigation**: `AuditChainService::trace_ancestry` clamps depth to `MAX_LINEAGE_DEPTH` (64) and tracks visited event hashes to block cyclic links.

## Security Verdict
No vulnerabilities or policy bypasses identified. Ready for hardening.
