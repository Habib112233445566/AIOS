# Specification: Secrets Handling MCP Tool Surface (SPEC-SECRETS-MCP)

- **Status**: APPROVED & IMPLEMENTED
- **Date**: 2026-10-02
- **Protocol**: Model Context Protocol (MCP) JSON-RPC 2.0 over stdio
- **Binding**: ADR-0035 §D-2
- **Reference Tasks**:
  - Research: [T-02631](file:///docs/tasks/evidence/T-02631-mcp-api-surface-research.md)
  - Specification: [T-02632](file:///docs/tasks/evidence/T-02632-mcp-api-surface-specification.md)
  - Scaffold: [T-02633](file:///docs/tasks/evidence/T-02633-mcp-api-surface-scaffold.md)
  - Implementation: [T-02634](file:///docs/tasks/evidence/T-02634-mcp-api-surface-implementation.md)
  - Unit Test: [T-02635](file:///docs/tasks/evidence/T-02635-mcp-api-surface-unit-test.md)
  - Integration: [T-02636](file:///docs/tasks/evidence/T-02636-mcp-api-surface-integration.md)
  - Security Review: [T-02637](file:///docs/tasks/evidence/T-02637-mcp-api-surface-security-review.md)
  - Hardening: [T-02638](file:///docs/tasks/evidence/T-02638-mcp-api-surface-hardening.md)
  - Documentation: [T-02639](file:///docs/tasks/evidence/T-02639-mcp-api-surface-documentation.md)

---

## 1. Tool Call Examples & Payloads

### 1.1 `aios.secret.store`
Registers or updates a credential in the runtime vault.

**JSON-RPC Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.store",
    "arguments": {
      "id": "github_runner_token",
      "name": "CI/CD Runner Registration Token",
      "kind": "api_key",
      "value": "ghp_secure_runner_token_48392",
      "scope": "actor",
      "target": "agent:mcp@aiosh-mcp"
    }
  }
}
```

**JSON-RPC Success Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "ok": true,
    "metadata": {
      "id": "github_runner_token",
      "name": "CI/CD Runner Registration Token",
      "kind": "api_key",
      "scope": "actor:agent:mcp@aiosh-mcp",
      "version": 1,
      "fingerprint": "a3f5b7...",
      "created_at": "2026-10-02T14:35:00Z",
      "state": "active"
    }
  }
}
```

### 1.2 `aios.secret.get`
Fetches a vaulted secret. By default returns masked output to prevent leakages into LLM contexts and logs.

**Masked Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.get",
    "arguments": {
      "id": "github_runner_token"
    }
  }
}
```
**Masked Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "ok": true,
    "exposed": false,
    "value": "ghp_...8392",
    "metadata": { "id": "github_runner_token", "state": "active", ... }
  }
}
```

**Plaintext Exposed Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.get",
    "arguments": {
      "id": "github_runner_token",
      "expose": true
    }
  }
}
```
**Plaintext Exposed Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "ok": true,
    "exposed": true,
    "value": "ghp_secure_runner_token_48392",
    "metadata": { "id": "github_runner_token", "state": "active", ... }
  }
}
```

### 1.3 `aios.secret.list`
Lists metadata for all stored secrets without exposing payloads.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.list",
    "arguments": {
      "kind": "api_key"
    }
  }
}
```

### 1.4 `aios.secret.rotate`
Rotates payload bytes and increments version.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.rotate",
    "arguments": {
      "id": "github_runner_token",
      "value": "ghp_rotated_runner_token_99999"
    }
  }
}
```

### 1.5 `aios.secret.revoke`
Revokes an active secret, locking future access.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.revoke",
    "arguments": {
      "id": "github_runner_token"
    }
  }
}
```

---

## 2. Invariants & Security Rules
- **SECMCP1**: Default retrieval (`expose: false` or omitted) returns masked representation only.
- **SECMCP2**: Listing (`aios.secret.list`) never outputs payload bytes under any circumstance.
- **SECMCP3**: All operations dispatch through `dispatch::recorded_call`, writing an immutable entry into `AuditRing`.
- **SECMCP4**: Path traversal attempts in `store_path` are rejected before disk access.
- **SECMCP5**: Caller scope checks are strictly evaluated via `SecretScope::allows`.

---

## 3. Constraints & Limitations
1. **Size Ceilings**: Secret payloads capped at 64 KiB (`MAX_SECRET_PAYLOAD_SIZE`).
2. **Store Capacity**: Vault capacity is 1,024 entries; store file size ceiling is 1 MiB (`1,048,576` bytes).
3. **Transport Protocol**: Local stdio JSON-RPC 2.0. TLS/HTTP transport is scheduled for Phase 3 network hardening.
4. **Hardware Cryptography**: Phase 2 uses software SHA-256 fingerprinting and local atomic storage. TPM 2.0 sealing is scheduled for Phase 3.
