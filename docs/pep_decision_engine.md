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

## 4. Core Service: `PepDecisionService`

The `PepDecisionService` provides stateful rule management, multi-index lookups (`by_subject`, `by_action`), atomic persistence, and quarantine recovery.

### 4.1 Features & Limits
- **Capacity Cap**: `MAX_RULES_IN_SERVICE = 5000` rules per service instance.
- **Atomic Persistence**: `save_to_path` writes to a temporary file (`.tmp.<pid>.<timestamp>`) and atomically renames to the target path.
- **Non-Destructive Quarantine**: Corrupted policy files are safely renamed to `<path>.bak.<timestamp>` (mode `0600` on Unix) before a clean store is reinitialized.
- **Thread Safety**: Pure evaluation and cloneable rule models ensure safe concurrent reads.

### 4.2 Rust API Example

```rust
use aiosh_core::pep_decision::{PepPolicyRule, PepDecisionEffect, PepRequest, PepCombiningAlgorithm};
use aiosh_core::pep_decision_service::PepDecisionService;

// Initialize service
let mut service = PepDecisionService::new();

// Add a policy rule
service.add_rule(PepPolicyRule {
    id: "rule-101".into(),
    target_subject: "agent:analyst".into(),
    target_resource: "fs:/data/*".into(),
    target_action: "read".into(),
    effect: PepDecisionEffect::Permit,
    priority: 10,
    obligations: vec![],
    description: "Allow analyst to read data".into(),
}).expect("add rule");

// Evaluate an access request
let request = PepRequest::new("agent:analyst", "fs:/data/stats.json", "read", None).unwrap();
let decision = service.evaluate(&request, PepCombiningAlgorithm::DenyOverrides);

assert!(decision.allowed);
```

---

## 5. MCP Integration (`aios.pep.evaluate`)

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

---

---

## 6. CLI Surface Reference (`aiosh pep`)

The engine provides comprehensive operator CLI tooling via `aiosh pep`:

### 6.1 Commands & Syntax
```bash
# Evaluate authorization request (exit code 0 = Permit, 1 = Deny, 2 = Error)
aiosh pep evaluate --subject <SUBJECT> --action <ACTION> --resource <RESOURCE> [--algorithm <ALG>] [--store <PATH>] [--json]

# Add a policy rule
aiosh pep rule-add --id <ID> --subject <SUBJECT> --action <ACTION> --resource <RESOURCE> --effect <permit|deny> [--priority <INT>] [--desc <TEXT>] [--store <PATH>] [--json]

# List active policy rules
aiosh pep rule-list [--subject <SUBJECT>] [--action <ACTION>] [--store <PATH>] [--json]

# Remove a policy rule
aiosh pep rule-remove <ID> [--store <PATH>] [--json]

# Show engine status and capacity
aiosh pep status [--store <PATH>] [--json]
```

### 6.2 Exit Code Semantics
- `0`: Success (for `evaluate`: Access `PERMIT`).
- `1`: Failure / Denied (for `evaluate`: Access `DENY`; or rule not found).
- `2`: Validation / Syntax / Hygiene error (missing flags, invalid effect, path traversal).

### 6.3 Structured JSON Envelope
When `--json` is specified, outputs adhere to:
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```

### 6.4 Constraints & Known Limitations
- Resource paths must not contain `..` path traversal segments.
- Policy store files must have a `.json` extension.
- Maximum rule capacity is 5000 rules.

---

## 7. Evidence & Traceability

- `T-02106`: [Data Model Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02106-data-model-integration.md)
- `T-02107`: [Data Model Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02107-data-model-security-review.md)
- `T-02108`: [Data Model Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02108-data-model-hardening.md)
- `T-02109`: [Data Model Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02109-data-model-documentation.md)
- `T-02110`: [Data Model Verification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02110-data-model-verification-evidenc.md)
- `T-02111`: [Core Service Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02111-core-service-research.md)
- `T-02112`: [Core Service Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02112-core-service-specification.md)
- `T-02113`: [Core Service Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02113-core-service-scaffold.md)
- `T-02114`: [Core Service Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02114-core-service-implementation.md)
- `T-02115`: [Core Service Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02115-core-service-unit-test.md)
- `T-02116`: [Core Service Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02116-core-service-integration.md)
- `T-02117`: [Core Service Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02117-core-service-security-review.md)
- `T-02118`: [Core Service Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02118-core-service-hardening.md)
- `T-02119`: [Core Service Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02119-core-service-documentation.md)
- `T-02120`: [Core Service Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02120-core-service-verification-evidenc.md)
- `T-02121`: [CLI Surface Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02121-cli-surface-research.md)
- `T-02122`: [CLI Surface Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02122-cli-surface-specification.md)
- `T-02123`: [CLI Surface Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02123-cli-surface-scaffold.md)
- `T-02124`: [CLI Surface Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02124-cli-surface-implementation.md)
- `T-02125`: [CLI Surface Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02125-cli-surface-unit-test.md)
- `T-02126`: [CLI Surface Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02126-cli-surface-integration.md)
- `T-02127`: [CLI Surface Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02127-cli-surface-security-review.md)
- `T-02128`: [CLI Surface Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02128-cli-surface-hardening.md)
- `T-02129`: [CLI Surface Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02129-cli-surface-documentation.md)
- `T-02130`: [CLI Surface Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02130-cli-surface-verification-evidenc.md)

