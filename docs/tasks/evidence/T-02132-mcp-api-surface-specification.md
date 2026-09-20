# T-02132: MCP/API Surface Specification — PEP Decision Engine

## Overview
- **Task ID**: `T-02132`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Status**: Completed

## 1. Tool Schemas & Contracts

### 1.1 `aios.pep.evaluate`
Evaluates an authorization request against inline or stored policy rules.
```json
{
  "name": "aios.pep.evaluate",
  "description": "Evaluate an authorization request against policy rules using the PEP Decision Engine",
  "inputSchema": {
    "type": "object",
    "properties": {
      "subject": { "type": "string", "description": "Subject identifier (e.g. agent:researcher)" },
      "resource": { "type": "string", "description": "Target resource URI (e.g. fs:/data/reports)" },
      "action": { "type": "string", "description": "Action requested (e.g. read, write, execute)" },
      "algorithm": {
        "type": "string",
        "enum": ["deny_overrides", "permit_overrides", "first_applicable"],
        "description": "Rule combining algorithm (default: deny_overrides)"
      },
      "rules": {
        "type": "array",
        "description": "Optional inline policy rules to evaluate against. If omitted, evaluates against persistent store.",
        "items": { "type": "object" }
      },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["subject", "resource", "action"],
    "additionalProperties": false
  }
}
```

### 1.2 `aios.pep.rule_add`
Adds a policy rule to the persistent store.
```json
{
  "name": "aios.pep.rule_add",
  "description": "Add a policy rule to the persistent PEP Decision Engine store",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Unique rule identifier" },
      "subject": { "type": "string", "description": "Target subject or wildcard" },
      "resource": { "type": "string", "description": "Target resource URI or wildcard" },
      "action": { "type": "string", "description": "Target action or wildcard" },
      "effect": { "type": "string", "enum": ["permit", "deny"], "description": "Rule effect" },
      "description": { "type": "string", "description": "Optional human-readable description" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["id", "effect"],
    "additionalProperties": false
  }
}
```

### 1.3 `aios.pep.rule_list`
Lists policy rules from the persistent store.
```json
{
  "name": "aios.pep.rule_list",
  "description": "List policy rules registered in the PEP Decision Engine",
  "inputSchema": {
    "type": "object",
    "properties": {
      "subject": { "type": "string", "description": "Optional subject filter" },
      "action": { "type": "string", "description": "Optional action filter" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "additionalProperties": false
  }
}
```

### 1.4 `aios.pep.rule_remove`
Removes a policy rule by ID.
```json
{
  "name": "aios.pep.rule_remove",
  "description": "Remove a policy rule from the PEP Decision Engine by ID",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Identifier of the rule to remove" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["id"],
    "additionalProperties": false
  }
}
```

### 1.5 `aios.pep.status`
Returns metrics and status of the PEP Decision Engine.
```json
{
  "name": "aios.pep.status",
  "description": "Get PEP Decision Engine status, rule count, and capacity metrics",
  "inputSchema": {
    "type": "object",
    "properties": {
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "additionalProperties": false
  }
}
```

## 2. Reused vs. New Interfaces
- **Reused**:
  - `aiosh_core::pep_decision::{PepRequest, PepDecision, PepPolicyRule, PepCombiningAlgorithm}`
  - `aiosh_core::pep_decision_service::PepDecisionService`
  - `dispatch::recorded_call` in `aiosh-mcp`
- **New**:
  - JSON-RPC tool declarations and dispatch arms in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 3. Audit Logging
Every MCP tool execution is routed through `dispatch::recorded_call`, which automatically records an immutable audit entry in the SQLite `AuditRing`.
