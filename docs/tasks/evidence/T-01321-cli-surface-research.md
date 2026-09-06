# T-01321: Init & Service Supervision - CLI Surface: Research

## Metadata
- **Task ID:** `T-01321`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Research
- **Status:** Complete

## 1. Objectives & Scope
Research the operator-facing command-line interface (`aiosh service`) for inspecting, querying, managing lifecycle transitions, and resolving dependency execution sequences for services on AIOS. The CLI surface must fulfill:
1. **Interactive Usability**: Clean, aligned terminal representation for interactive operator sessions.
2. **Machine Parsing**: Deterministic, structured JSON envelopes when `--json` is specified for scripts and downstream tools.
3. **Auditability (ADR-0035)**: Synchronous audit row emission via `classify_and_emit` on every CLI invocation branch (success and failure).
4. **Defensive Sizing & Validation**: Strict 1 MiB payload ceiling on specification inputs, string length limits, control character rejection, and standard POSIX exit code contracts.

## 2. Prior Art & Authoritative Sources
- **`systemctl(1)` Man Page**: Standard systemd control interface. Subcommands: `list-units`, `status`, `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`.
- **`rc-service(8)` & `rc-status(8)`**: OpenRC service supervision tools (Alpine, Gentoo) emphasizing simple verbs, clean return codes, and runlevel separation.
- **`s6-rc(8)` Architecture**: Modern supervision suite providing compiled dependency DAG calculation and deterministic state transitions.
- **POSIX.1-2017 Utility Conventions (IEEE Std 1003.1-2017)**: Guidelines 3 through 10 governing standard option syntax, argument validation, and return code semantics (0 for success, 1 for operational error, 2 for syntax/argument error).

## 3. Fact vs. Assumption Separation

### Established Facts
- **Fact 1 (Data Model & Core Service)**: `aiosh-core::service` and `aiosh-core::service_service` provide canonical data structures (`ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceQuery`, etc.) and `ServiceStore` enforcing invariants `SS1..SS5` and `CS1..CS5`.
- **Fact 2 (Existing Subcommands)**: `aiosh service` implements `validate` (`--name` or `--spec`), `list` (with `--pattern`, `--state`, `--mode`, `--limit`), `show` (by name), `action` (start, stop, restart, reload, enable, disable, mask, unmask), and `order` (topological dependency startup order via Kahn's algorithm).
- **Fact 3 (Audit Emission)**: Every execution path calls `classify_and_emit` to record event details to the SQLite WAL audit ring or `audit.log`.
- **Fact 4 (Memory & Input Hardening)**: Input reads are strictly bounded: 1 MiB limit on specification files/strings, 1024 characters on `--store`, 256 characters on `--pattern`, 128 characters on service names, control characters rejected.
- **Fact 5 (Exit Code Contract)**: Return codes follow standard semantics: 0 = success, 1 = operational/store failure, 2 = argument/syntax error.

### Engineering Assumptions
- **Assumption 1 (Convenience Shortcuts)**: Operators familiar with `systemctl` will benefit from convenience shortcut verbs (`status`, `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`) mapping directly to underlying `show` and `action` operations.
- **Assumption 2 (Dependency Sequence Usability)**: The `order` subcommand output should display both step numbers and service names in topological start order, facilitating verification and bootstrap automation.
- **Assumption 3 (Sandbox/Store Isolation)**: The `--store <path>` flag allows operators to inspect or manipulate offline/chroot service stores without altering active running system state.

## 4. CLI Grammar & Command Taxonomy
```text
aiosh service <subcommand> [options]

Subcommands:
  validate   --name <name> | --spec <file_or_json> [--json]
  list       [--pattern <pat>] [--state <state>] [--mode <mode>] [--limit <n>] [--store <path>] [--json]
  show       <name> [--store <path>] [--json]
  action     <name> <action> [--store <path>] [--json]
  order      <name> [--store <path>] [--json]
  status     <name> [--store <path>] [--json]          (alias for show)
  start      <name> [--store <path>] [--json]          (alias for action <name> start)
  stop       <name> [--store <path>] [--json]          (alias for action <name> stop)
  restart    <name> [--store <path>] [--json]          (alias for action <name> restart)

Flags:
  --json           Format response in canonical JSON envelope
  --store <path>   Explicit path to persistent service store file
  --help, -h       Display help and usage details
```

## 5. Exit Code Contract
- `0`: Success (action executed, order planned, service shown, list returned, validation passed).
- `1`: Operational error (service not found, store file unreadable, persistence failed, action rejected by FSM).
- `2`: Syntax or invocation error (missing required arguments, unrecognized subcommand or flag, payload > 1 MiB, invalid service name syntax).

## 6. Decisions & Unknowns for Specification Phase (T-01322)
- **Decision 1**: Standardize whether `status` should be a first-class alias for `show` to maximize developer velocity and alignment with `systemctl status`.
- **Decision 2**: Provide direct action shortcuts (`start`, `stop`, `restart`, `reload`) while preserving `action <name> <verb>` for programmatic dispatch.
- **Decision 3**: Ensure all error responses in `--json` mode conform strictly to `{"code": <n>, "data": null, "error": {"code": "...", "message": "..."}}`.
- **Decision 4**: Maintain 100% test coverage and zero regressions in `tools/test_service_suites.py`.
