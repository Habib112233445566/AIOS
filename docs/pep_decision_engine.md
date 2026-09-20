# AIOS PEP Decision Engine Architecture & Specification

## 1. Overview & Principles

The **PEP Decision Engine** is the authoritative Policy Decision Point (PDP) and Policy Enforcement Point (PEP) evaluation fabric in the AIOS Security Kernel (Phase 2). It mediates all actions executed by agents, CLI commands, and MCP tools against declared policy rules.

Key architectural principles:
- **Zero Ambient Authority**: No entity has authority simply by running in the userspace environment. Every operation requires explicit authorization.
- **Fail-Closed Default Deny**: If no rule permits an action, or if an evaluation fails or encounters ambiguity, the engine strictly denies access (`PEPDEC1`).
- **Complete Mediation**: Every request is intercepted and evaluated before execution.
- **Deterministic Precedence**: Rule evaluation outcomes are reproducible across all runs and platforms.

---

## 2. Invariants (`PEPDEC1..PEPDEC6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`PEPDEC1`** | **Complete Mediation & Fail-Closed** | Default deny: `allowed == (effect == PepDecisionEffect::Permit)`. Any evaluation without an explicit permit rule returns `Deny`. |
| **`PEPDEC2`** | **Canonical Request Context** | Every authorization request must be validated: subject ($\le 256$ chars), resource ($\le 1024$ chars), action ($\le 64$ chars). No control characters or `..` path traversal components. |
| **`PEPDEC3`** | **Deterministic Combining Algorithms** | Supported algorithms: `DenyOverrides`, `PermitOverrides`, and `FirstApplicable`. Evaluation order and precedence rules are deterministic. |
| **`PEPDEC4`** | **Atomic Decision Response** | An evaluation produces an atomic, immutable `PepDecision` containing effect, boolean verdict, matched rule ID, reason, obligations, and latency. |
| **`PEPDEC5`** | **Pure Evaluation** | Rule evaluation is pure and side-effect-free, guaranteeing zero state mutation. |
| **`PEPDEC6`** | **Audit Trail Traceability** | Every decision is structured for direct logging and verification in the AIOS Audit Ring. |

---

## 3. Data Model

### 3.1 Authorization Request (`PepRequest`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepRequest {
    pub id: String,
    pub subject: String,
    pub resource: String,
    pub action: String,
    pub environment: PepEnvironmentContext,
}
```

### 3.2 Decision Outcome (`PepDecision`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepDecision {
    pub request_id: String,
    pub effect: PepDecisionEffect,
    pub allowed: bool,
    pub matched_rule_id: Option<String>,
    pub reason: String,
    pub obligations: Vec<PepObligation>,
    pub evaluated_at: String,
    pub evaluation_duration_us: u64,
}
```

### 3.3 Combining Algorithms (`PepCombiningAlgorithm`)

- **`DenyOverrides`** (Default): If any matching rule specifies `Deny`, the overall decision is `Deny`. If at least one matches `Permit` and none match `Deny`, the decision is `Permit`. Otherwise `Deny`.
- **`PermitOverrides`**: If any matching rule specifies `Permit`, the overall decision is `Permit`. If at least one matches `Deny` and none match `Permit`, the decision is `Deny`. Otherwise `Deny`.
- **`FirstApplicable`**: The first matching rule in the evaluation list determines the outcome (`Permit` or `Deny`). If no rule matches, defaults to `Deny`.

---

## 4. MCP Integration (`aios.pep.evaluate`)

The decision engine is exposed to agents and tools via `aios.pep.evaluate`.

### Example: Request with Inline Rules

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.evaluate",
    "arguments": {
      "subject": "agent:researcher",
      "resource": "fs:/data/reports/q3.pdf",
      "action": "read",
      "algorithm": "deny_overrides",
      "rules": [
        {
          "id": "rule_allow_reports",
          "target_subject": "agent:researcher",
          "target_resource": "fs:/data/reports/*",
          "target_action": "read",
          "effect": "permit",
          "obligations": [
            {
              "type": "audit_log",
              "level": "info",
              "message": "researcher read report"
            }
          ],
          "description": "allow reading reports"
        }
      ]
    }
  }
}
```

### Example: Decision Response

```json
{
  "ok": true,
  "tool": "aios.pep.evaluate",
  "decision": {
    "request_id": "pep_req_1726858000000_123456",
    "effect": "permit",
    "allowed": true,
    "matched_rule_id": "rule_allow_reports",
    "reason": "permitted by rule 'rule_allow_reports': allow reading reports",
    "obligations": [
      {
        "type": "audit_log",
        "level": "info",
        "message": "researcher read report"
      }
    ],
    "evaluated_at": "2026-09-20T18:46:40Z",
    "evaluation_duration_us": 14
  }
}
```
