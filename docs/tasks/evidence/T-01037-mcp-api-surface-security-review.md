# T-01037 — Distro Selection & Justification / MCP/API Surface: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-MCP-1: JSON-RPC Payload Injection & Malformed Arguments
- **Threat**: Malicious client sends malformed types (e.g., array instead of string, null bytes) to trigger unhandled deserialization panics.
- **Mitigation & Verification**: Handled by `serde_json`. Missing required fields return explicit error envelopes (`ok: false` with detailed message) or `-32602`.

### AS-MCP-2: Local File Exfiltration via `store_path`
- **Threat**: Supplying arbitrary local file paths (`store_path: "/etc/shadow"`) to disclose sensitive host contents.
- **Mitigation & Verification**: The loader validates that the content conforms strictly to `DistroStore` JSON schema; arbitrary system files fail parsing with `ok: false`, preventing any data leakage.

### AS-MCP-3: Audit Evasion & Non-Repudiation
- **Threat**: Agent calls distro tools to modify or inspect system state without an immutable log.
- **Mitigation & Verification**: `dispatch::recorded_call` wraps each handler, enforcing cryptographic audit logging with SHA-256 hash chaining prior to returning the result to the caller.

### AS-MCP-4: LLM Prompt Injection via Distribution Metadata
- **Threat**: Profile justification strings inject malicious system instructions into agent context.
- **Mitigation & Verification**: Ingestion validates alphanumeric IDs and strict semver constraints; AIOS agent prompts treat all profile payloads as untrusted data envelopes.

## 2. Invariant Checklist
- [x] Input schema bounds enforced.
- [x] Policy Enforcement Point gating verified.
- [x] Audit row emission verified.
- [x] No open policy bypass remains.
