# T-02427: Sandbox Enforcement CLI Surface Security Review

## 1. Security Review Scope
This review evaluates the security posture and vulnerability resistance of the Sandbox Enforcement CLI surface (`aiosh sandbox` / `aiosh sb` in `code/aiosh-rust/aiosh-cli/src/main.rs`).

## 2. Threat Modeling & Abuse Scenarios

### Scenario A: Flag Confusion & CLI Option Injection (CWE-88)
- **Vector**: An operator or higher-level agent runs `aiosh sandbox exec --profile strict my_tool --json`. If the CLI does not enforce a delimiter, the child arguments could hijack parent CLI behavior or options.
- **Mitigation**: The CLI strictly requires a `--` delimiter. All arguments before `--` are parsed strictly as `aiosh` flags; all arguments after `--` are passed verbatim to the target process. Absence of `--` or missing executable after `--` rejects fail-closed with exit code `2`.

### Scenario B: Directory Traversal via Working Directory (CWE-22)
- **Vector**: Supplying `--cwd ../../etc` or `--cwd /var/log/../../root` to execute inside sensitive directories.
- **Mitigation**: The CLI inspects `--cwd` and explicitly rejects any path containing `..` with exit code `2` (`TRAVERSAL_DETECTED`), emitting a security failure audit event before process invocation.

### Scenario C: Terminal Escape Sequence Injection (CWE-150)
- **Vector**: A malicious sandboxed child process emits ANSI terminal control codes (e.g., OSC 8 hyperlinks, cursor manipulation, escape characters `\x1b`) to manipulate terminal state or spoof operator output.
- **Mitigation**: All raw child stdout and stderr bytes are passed through `sanitize_terminal_output`, which replaces non-whitespace control characters with Unicode replacement glyphs (`\u{FFFD}`).

### Scenario D: Missing PEP Grant in Enforcing Mode (CWE-862)
- **Vector**: Attempting to invoke sensitive sandboxed commands without a cryptographically issued PEP capability grant.
- **Mitigation**: When `enforce_pep_grants` is enabled, the CLI passes `--grant <token>` as `pep_grant_id` to `SandboxService::execute`. Requests without a valid grant fail with code `1` (`ERR_SANDBOX_PEP_UNAUTHORIZED`).

### Scenario E: Unbounded Buffer Overflow / Memory Exhaustion
- **Vector**: Executing commands that produce gigabytes of standard output.
- **Mitigation**: Child output is truncated at `DEFAULT_MAX_OUTPUT_CAPTURE_BYTES = 10 MiB` in `SandboxService`, preventing out-of-memory crashes in CLI processes.

## 3. Finding & Verdict
- **Critical / High Vulnerabilities**: 0
- **Medium / Low Vulnerabilities**: 0
- **Verdict**: **APPROVED / PASS**. Input validation, delimiter enforcement, output sanitization, and audit invariants are fully verified.
