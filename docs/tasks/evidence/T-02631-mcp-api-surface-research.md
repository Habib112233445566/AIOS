# Task Evidence: T-02631 — Secrets Handling MCP/API Surface Research

## 1. Objective
Establish facts, constraints, and architecture for exposing the Secrets Handling subsystem over the Model Context Protocol (MCP) tool surface in `code/aiosh-rust/aiosh-mcp`.

## 2. Findings: Facts vs. Assumptions

### Authoritative Facts
1. **MCP Architectural Contract (ADR-0035 §D-2)**:
   - MCP is the exclusive tool-call protocol AIOS exposes to external LLMs and autonomous agents over stdio JSON-RPC.
   - Every tool call executes via `dispatch::recorded_call`, passing through the classifier, PEP policy evaluation, and the tamper-evident audit ring (`AuditRing`).
2. **Namespace Segregation**:
   - The plural namespace `aios.secrets.*` (`aios.secrets.scan`, `aios.secrets.check`) is reserved for static repository hygiene scanning (`T-00711..T-00760`).
   - The singular namespace `aios.secret.*` (`aios.secret.store`, `aios.secret.get`, `aios.secret.list`, `aios.secret.rotate`, `aios.secret.revoke`) is designated for the Phase 2 Security Kernel runtime secret vault.
3. **Core Subsystem Reusability**:
   - All core operations, data models, and validations exist in `aiosh_core::secret_service::SecretService` and `aiosh_core::secret_data_model::{SecretEntry, SecretMetadata, SecretKind, SecretScope, SecretValue}`.
4. **Information Disclosure Defense**:
   - Plaintext disclosure must be guarded: `aios.secret.get` must return masked payloads (`masked_display()`) by default unless an explicit `expose: true` parameter is supplied.
   - `aios.secret.list` must return only metadata descriptors and never disclose raw payloads.

### Assumptions & Decisions
- **Store Location**: Resolves via `AIOS_SECRETS_STORE` environment variable, falling back to `AIOS_STATE_DIR/secrets_vault.json` or `target/secrets_vault.json`, with optional override via `store_path` (guarded against `..` traversal).
- **Scope Context**: The caller's scope defaults to `actor:mcp-agent` unless overridden by explicit caller scope parameters, and must satisfy scope containment hierarchy against the secret's defined scope.

## 3. Decisions for Specification (T-02632)
1. Register tools in `Server::tool_manifest`:
   - `aios.secret.store`: Store or register a secret.
   - `aios.secret.get`: Fetch secret metadata and payload (masked by default).
   - `aios.secret.list`: List vaulted secrets metadata without payloads.
   - `aios.secret.rotate`: Rotate secret payload and increment version.
   - `aios.secret.revoke`: Revoke secret, locking future access.
2. Enforce strict JSON Schema argument validation on every MCP tool call.
