# T-00317 — Dependency & Toolchain Pinning: Data Model Security Review

## 1. Scope
This review evaluates the security posture of the `ToolchainManifest` data model and its integration via the `aios.toolchain.config.get` MCP endpoint.

## 2. Threat Modeling & Abuse Scenarios

### 2.1 Scenario A: Arbitrary File Read via Config Path Injection
- **Attack Vector**: An attacker manipulating the host environment sets `AIOSH_TOOLCHAIN_CONFIG=/etc/shadow` prior to launching the MCP server.
- **Analysis**: The `from_env` loader uses `std::fs::File::open` on the provided path, then reads the first 64KB. If the file is successfully read, `serde_json` attempts to parse it. Since `/etc/shadow` is not valid JSON, the parser will fail with a generic `"Malformed toolchain config"` error, discarding the buffer. The contents of the file are never reflected back to the caller.
- **Conclusion**: Safe. The vulnerability is mitigated by strict JSON schema validation, preventing data exfiltration.

### 2.2 Scenario B: Resource Exhaustion (DoS) via Giant Config File
- **Attack Vector**: An attacker points `AIOSH_TOOLCHAIN_CONFIG` to `/dev/zero` or a massive 50GB file to trigger Out-Of-Memory (OOM) panic during JSON load.
- **Analysis**: The loader uses `.take(65_536)` during the file read operation. The buffer stops pulling bytes precisely at 64KB. 
- **Conclusion**: Safe. The limit is mechanically enforced, preventing memory exhaustion.

### 2.3 Scenario C: Privilege Escalation via Read-Only Bypass
- **Attack Vector**: An unprivileged AI agent calls `aios.toolchain.config.get` to learn about the host's version pinning in order to plan an exploit.
- **Analysis**: The tool is correctly classified as a read-only operation in the MCP dispatcher (grant requirement is false). Furthermore, it is routed through `dispatch::recorded_call`, meaning the query is durably logged to the Audit Ring. 
- **Conclusion**: Safe. Information disclosure is intentional and fully audited. No state mutation occurs.

## 3. Findings
No policy bypasses, input validation loopholes, or injection vulnerabilities were identified. The data model strictly implements safe bounded reads and fails closed.

**Status**: PASS.
