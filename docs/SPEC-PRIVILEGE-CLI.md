# Specification: Privilege Escalation Prevention CLI Surface

## 1. Overview
The Privilege Escalation Prevention CLI Surface provides operators and automation tools with shell commands under `aiosh privilege` to inspect active execution contexts, evaluate and execute dynamic privilege elevations, safely de-escalate privilege tiers, and query assigned capabilities.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `PRIVESC_CLI1` | **Audit Provenance Mandate** | Every `aiosh privilege` subcommand invocation emits an audit record recording tool, action, actor, target tier, and outcome. |
| `PRIVESC_CLI2` | **Terminal Sanitization** | Non-JSON output streams pass through `sanitize_terminal` to prevent terminal injection (CWE-150). |
| `PRIVESC_CLI3` | **Strict Flag Validation** | Invalid privilege levels, malformed capability names, or missing mandatory parameters exit with code 1. |
| `PRIVESC_CLI4` | **Kernel Tier Rejection** | CLI invocations attempting to elevate to `SystemKernel` fail-fast with exit code 1. |
| `PRIVESC_CLI5` | **Dual Output Format** | Every subcommand supports standard human-readable output and machine-readable `--json` output envelopes (`{"code": 0, "data": ...}`). |

---

## 3. Command Syntax & Options

### `aiosh privilege status`
- **Syntax**: `aiosh privilege status [--actor <id>] [--json]`
- **Description**: Displays the active privilege level, capabilities, elevation status, and grant token for the actor.

### `aiosh privilege elevate`
- **Syntax**: `aiosh privilege elevate --to <tier> [--grant <token>] [--actor <id>] [--caps <c1,c2>] [--json]`
- **Description**: Requests dynamic privilege elevation to `<tier>`. Requires `--grant` if escalating above current tier.

### `aiosh privilege drop`
- **Syntax**: `aiosh privilege drop --to <tier> [--actor <id>] [--json]`
- **Description**: De-escalates privilege level to `<tier>`, stripping higher capabilities and clearing elevation grants.

### `aiosh privilege revoke`
- **Syntax**: `aiosh privilege revoke [--actor <id>] [--json]`
- **Description**: Restores baseline privilege level, clearing dynamic grants and revoking elevated capabilities.

### `aiosh privilege check`
- **Syntax**: `aiosh privilege check --cap <capability> [--actor <id>] [--json]`
- **Description**: Verifies whether an actor holds a specific capability in their active context.

### `aiosh privilege list`
- **Syntax**: `aiosh privilege list [--json]`
- **Description**: Lists all registered actors and their active tiers.

---

## 4. Input Constraints & Hardening Bounds

| Parameter / Boundary | Limit | Behavior on Exceeded |
|---|---|---|
| State store file size | $\le 1\text{ MiB}$ | Exit code 2 (payload too large) |
| Path Traversal (`..`) | Forbidden | Exit code 2 (security violation) |
| Actor ID Length | $\le 128\text{ bytes}$ | Exit code 2 (identifier too long) |
| Actor ID Characters | Printable ASCII / UTF-8, no control chars | Exit code 2 (malformed identifier) |
| Grant Token Length | $\le 256\text{ bytes}$ | Exit code 2 (token too long) |
| Grant Token Characters | Printable ASCII / UTF-8, no control chars | Exit code 2 (malformed token) |
| Max Capabilities per Request | $\le 32$ | Exit code 2 (capability list exceeded) |

---

## 5. Exit Codes
- `0`: Success / evaluation allowed.
- `1`: Validation error, grant missing/rejected, or actor not found.
- `2`: Hardening boundary violation (input too large, control characters, traversal attempt).

