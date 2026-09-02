# T-00347 — Dependency & Toolchain Pinning / MCP/API surface: Security Review
# T-00348 — Dependency & Toolchain Pinning / MCP/API surface: Hardening

## Security Review

### 1. Threat Vectors Addressed
- **Arbitrary Config Loading**: The MCP tool `aios.toolchain.check` does not accept a `--config` argument. It hardcodes the use of `ToolchainManifest::from_env()`. This prevents a hostile agent from specifying a malicious config path (e.g., `/dev/null` or a fake permissive JSON) to bypass the host validation.
- **Side Effects**: The enforcement process is strictly read-only. It shells out to `rustc`, `node`, `python`, etc., with `--version`, which are safe, read-only commands without side effects.
- **Audit Logging Evasion**: The tool is wrapped in `dispatch::recorded_call()`. Every time an agent calls `aios.toolchain.check`, an immutable row is appended to the audit ring with `tool = "aios.toolchain.check"`.
- **PEP Gate Bypassing**: Because the tool has no side effects and does not alter the ledger or system state, it safely sets `requires_grant = false`.

### 2. Hardening Measures Implemented
The surface was already hardened during the specification phase by deliberately omitting the `config_path` argument from the MCP input schema, keeping the surface area perfectly minimal. The JSON-RPC parsing handles arbitrary inputs gracefully (returning `-32602` or equivalent if unrecognized arguments are passed, though `recorded_call` handles the invocation).

## Conclusion
The MCP API surface for toolchain pinning is secure, fully audited, and properly isolated from agent manipulation. No further hardening is required.
