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

## 7. Model Context Protocol (MCP) & API Surface

The PEP Decision Engine exposes 5 MCP tools under the `aios.pep.*` namespace:

### 7.1 Available Tools

#### 1. `aios.pep.status`
Retrieves policy store status, total rule count, capacity, and active combining algorithm.
- **Parameters**: `store_path` (string, optional).
- **Response**:
```json
{
  "ok": true,
  "tool": "aios.pep.status",
  "store_path": ".aios/pep_policies.json",
  "store_exists": true,
  "rules_count": 12,
  "max_capacity": 5000,
  "default_algorithm": "deny_overrides",
  "unique_subjects": 4,
  "unique_actions": 3
}
```

#### 2. `aios.pep.rule_add`
Registers a new policy rule with atomic store persistence.
- **Parameters**:
  - `id` (string, required): Unique identifier (1..128 chars, no control chars).
  - `effect` (string, required): `"permit"` or `"deny"`.
  - `subject` (string, optional): Target subject identity.
  - `resource` (string, optional): Target resource URI.
  - `action` (string, optional): Target action name.
  - `description` (string, optional): Human-readable rule description.
  - `store_path` (string, optional): Destination store path.
- **Example Call**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.rule_add",
    "arguments": {
      "id": "rule_worker_read",
      "subject": "agent:worker",
      "resource": "fs:/tmp/*",
      "action": "read",
      "effect": "permit",
      "description": "Allow worker reading tmp"
    }
  }
}
```

#### 3. `aios.pep.rule_list`
Lists registered policy rules with optional subject and action filtering.
- **Parameters**:
  - `subject` (string, optional): Filter by target subject.
  - `action` (string, optional): Filter by target action.
  - `store_path` (string, optional): Policy store path.

#### 4. `aios.pep.rule_remove`
Removes a policy rule by ID with atomic store update.
- **Parameters**:
  - `id` (string, required): Rule ID to remove.
  - `store_path` (string, optional): Policy store path.

#### 5. `aios.pep.evaluate`
Evaluates an access request against the persistent policy store or inline rules.
- **Parameters**:
  - `subject` (string, required): Subject identifier.
  - `resource` (string, required): Resource URI.
  - `action` (string, required): Action name.
  - `algorithm` (string, optional): Combining algorithm (`"deny_overrides"`, `"permit_overrides"`, `"first_applicable"`). Default: `"deny_overrides"`.
  - `store_path` (string, optional): Policy store path.
  - `rules` (array of objects, optional): Inline rules to evaluate against (if omitted, evaluates against persistent store).

### 7.2 Constraints & Security Guarantees
- **Audit Logging**: All invocations route through `dispatch::recorded_call`, writing an immutable row to SQLite.
- **Path Hygiene**: `store_path` validated via `validate_pep_service_path` (rejecting `..`, length $\le 1024$, `.json` required).
- **Fail-Closed Default**: Requests evaluated without matching permit rules return `deny` (`allowed: false`).

---

## 8. Configuration Subsystem (`pep_config`)

The PEP Decision Engine is configured via `PepConfig` in `aiosh-core`, supporting JSON configuration files, environment variables, and compiled fail-safe defaults.

### 8.1 Configuration Invariants (`PEPCONF1..PEPCONF6`)
- **`PEPCONF1` (Path Hygiene)**: `store_path` must be $\le 1024$ chars, end in `.json`, contain zero control characters, and contain no `..` parent traversal components.
- **`PEPCONF2` (Resource & Registry Bounds)**: `max_rules` bounded within $[1, 50\,000]$ (default: 5,000); `max_store_bytes` bounded within $[1\,024, 104\,857\,600]$ (default: 10 MiB).
- **`PEPCONF3` (Algorithm Governance)**: `default_algorithm` restricted strictly to known combining algorithms: `deny_overrides`, `permit_overrides`, `first_applicable`.
- **`PEPCONF4` (Audit & Quarantine Settings)**: `audit_all_evaluations` (default: true) and `auto_quarantine_corrupt` (default: true).
- **`PEPCONF5` (Atomic Persistence & Symlink Rejection)**: Atomic file persistence via `.tmp.<pid>` rename pattern; symlinks strictly rejected before loading.
- **`PEPCONF6` (Environment Precedence)**: CLI flag `--store` > `AIOSH_PEP_STORE_PATH` > `AIOSH_PEP_CONFIG` file > Defaults.

### 8.2 JSON Configuration Schema
```json
{
  "version": "1.0.0",
  "store_path": ".aios/pep_policies.json",
  "max_store_bytes": 10485760,
  "max_rules": 5000,
  "default_algorithm": "deny_overrides",
  "audit_all_evaluations": true,
  "auto_quarantine_corrupt": true
}
```

### 8.3 Environment Variables
- `AIOSH_PEP_CONFIG`: Path to JSON configuration file.
- `AIOSH_PEP_STORE_PATH`: Overrides the policy store JSON file path.
- `AIOSH_PEP_MAX_RULES`: Overrides the maximum in-memory rule registry capacity (1..50,000).
- `AIOSH_PEP_DEFAULT_ALGORITHM`: Overrides the default combining algorithm (`deny_overrides`, `permit_overrides`, `first_applicable`).

---

## 9. Evidence & Traceability

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
- `T-02131`: [MCP/API Surface Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02131-mcp-api-surface-research.md)
- `T-02132`: [MCP/API Surface Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02132-mcp-api-surface-specification.md)
- `T-02133`: [MCP/API Surface Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02133-mcp-api-surface-scaffold.md)
- `T-02134`: [MCP/API Surface Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02134-mcp-api-surface-implementation.md)
- `T-02135`: [MCP/API Surface Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02135-mcp-api-surface-unit-test.md)
- `T-02136`: [MCP/API Surface Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02136-mcp-api-surface-integration.md)
- `T-02137`: [MCP/API Surface Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02137-mcp-api-surface-security-review.md)
- `T-02138`: [MCP/API Surface Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02138-mcp-api-surface-hardening.md)
- `T-02139`: [MCP/API Surface Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02139-mcp-api-surface-documentation.md)
- `T-02140`: [MCP/API Surface Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02140-mcp-api-surface-verification-evidenc.md)
- `T-02141`: [Configuration Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02141-configuration-research.md)
- `T-02142`: [Configuration Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02142-configuration-specification.md)
- `T-02143`: [Configuration Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02143-configuration-scaffold.md)
- `T-02144`: [Configuration Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02144-configuration-implementation.md)
- `T-02145`: [Configuration Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02145-configuration-unit-test.md)
- `T-02146`: [Configuration Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02146-configuration-integration.md)
- `T-02147`: [Configuration Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02147-configuration-security-review.md)
- `T-02148`: [Configuration Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02148-configuration-hardening.md)
- `T-02149`: [Configuration Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02149-configuration-documentation.md)
- `T-02150`: [Configuration Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02150-configuration-verification-evidenc.md)
- `T-02151`: [Automated Tests Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02151-automated-tests-research.md)
- `T-02152`: [Automated Tests Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02152-automated-tests-specification.md)
- `T-02153`: [Automated Tests Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02153-automated-tests-scaffold.md)
- `T-02154`: [Automated Tests Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02154-automated-tests-implementation.md)
- `T-02155`: [Automated Tests Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02155-automated-tests-unit-test.md)
- `T-02156`: [Automated Tests Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02156-automated-tests-integration.md)
- `T-02157`: [Automated Tests Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02157-automated-tests-security-review.md)
- `T-02158`: [Automated Tests Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02158-automated-tests-hardening.md)
- `T-02159`: [Automated Tests Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02159-automated-tests-documentation.md)
- `T-02160`: [Automated Tests Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02160-automated-tests-verification-evidenc.md)
- `T-02161`: [Security Policy Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02161-security-policy-research.md)
- `T-02162`: [Security Policy Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02162-security-policy-specification.md)
- `T-02163`: [Security Policy Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02163-security-policy-scaffold.md)
- `T-02164`: [Security Policy Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02164-security-policy-implementation.md)
- `T-02165`: [Security Policy Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02165-security-policy-unit-test.md)
- `T-02166`: [Security Policy Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02166-security-policy-integration.md)
- `T-02167`: [Security Policy Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02167-security-policy-security-review.md)
- `T-02168`: [Security Policy Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02168-security-policy-hardening.md)
- `T-02169`: [Security Policy Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02169-security-policy-documentation.md)
- `T-02170`: [Security Policy Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02170-security-policy-verification-evidenc.md)
- `T-02171`: [Observability Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02171-observability-research.md)
- `T-02172`: [Observability Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02172-observability-specification.md)
- `T-02173`: [Observability Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02173-observability-scaffold.md)
- `T-02174`: [Observability Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02174-observability-implementation.md)
- `T-02175`: [Observability Unit Tests](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02175-observability-unit-test.md)
- `T-02176`: [Observability Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02176-observability-integration.md)
- `T-02177`: [Observability Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02177-observability-security-review.md)
- `T-02178`: [Observability Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02178-observability-hardening.md)
- `T-02179`: [Observability Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02179-observability-documentation.md)
- `T-02180`: [Observability Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02180-observability-verification-evidenc.md)

---

## 10. Automated Test Suite Reference (Sub-Epic 6)

The PEP Decision Engine automated test suite (`code/aiosh-rust/aiosh-core/tests/test_pep_decision_e2e.rs`) verifies end-to-end integration and security invariants across the decision engine lifecycle.

### 10.1 Invariants Matrix
- **`PEPE2E1` (Algorithm Matrix)**: Verifies `DenyOverrides`, `PermitOverrides`, `FirstApplicable`, and default-deny against conflicting and unmatched requests.
- **`PEPE2E2` (Obligation Delivery)**: Asserts accurate delivery of structured obligations (`AuditLog`, `RateLimit`).
- **`PEPE2E3` (Capacity Stress & Boundary Limits)**: Tests 5,000 rules registered, fast indexed evaluation, and rejection of rule 5,001 with `PEPSERV_ERR_CAPACITY`.
- **`PEPE2E4` (Corrupt Store Quarantine)**: Simulates corrupted JSON files and verifies non-destructive quarantine to `.bak.<timestamp>`.
- **`PEPE2E5` (Adversarial Fuzzing & Traversal)**: Asserts rejection of path traversals (`..`), null bytes, control characters, and non-`.json` extensions.
- **`PEPE2E6` (Cross-Surface Persistence Parity)**: Verifies lossless JSON roundtrip serialization between memory and disk stores.

### 10.2 Invocation Examples
```bash
# Run the Rust end-to-end integration test suite
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_pep_decision_e2e

# Run all PEP test suites in aiosh-core
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_pep_decision --test test_pep_decision_service --test test_pep_config --test test_pep_decision_e2e

# Run CLI and MCP smoke suites
python code/aiosh-cli/tests/test_pep_cli_smoke.py
python code/aiosh-cli/tests/test_pep_config_smoke.py
python code/aiosh-mcp/tests/test_pep_decision_smoke.py
```

### 10.3 Constraints and Known Limitations
1. `FirstApplicable` combines candidate rules sorted deterministically by rule ID. Rules with lower lexical IDs evaluate earlier.
2. Single-query evaluation evaluates up to `MAX_PEP_RULES_PER_EVALUATION = 1000` matching candidate rules. Total registry capacity supports up to `MAX_RULES_IN_SERVICE = 5000` rules.

---

## 11. Security Policy Subsystem Reference (Sub-Epic 7)

The PEP Security Policy subsystem (`aiosh_core::pep_security_policy`) provides governance over authorization enforcement modes, administrative authoring boundaries, obligation criticality, and temporal validity.

### 11.1 Key Invariants (`PEPPOL1..PEPPOL6`)
- **`PEPPOL1` (Enforcement Modes)**: Supports `Enforcing` (default, fail-closed), `Permissive` (dry-run mode allowing denied actions while preserving `effect: Deny` and injecting an audit warning), and `Disabled`.
- **`PEPPOL2` (Administrative Privilege Governance)**: Rejects unprivileged additions of `Permit` rules targeting restricted resources (`sys:*`, `sec:*`, `kernel:*`) with `PEPPOL_ERR_PRIVILEGE`.
- **`PEPPOL3` (Obligation Criticality)**: `Strict` criticality revokes permit decisions to `Deny` on obligation failure; `BestEffort` logs a warning audit obligation.
- **`PEPPOL4` (Temporal Validity)**: Active window verification via `valid_from_epoch_secs` and `valid_until_epoch_secs`. Expired policies return default deny with `PEPPOL_ERR_TEMPORAL`.
- **`PEPPOL5` (Atomic Persistence & Path Hygiene)**: Safe atomic file writes (`.tmp.<pid>`), symlink rejection, and 64 KiB read bounds.
- **`PEPPOL6` (Audit Integration)**: Consequential policy mutations write immutable audit rows to the SQLite audit ring.

### 11.2 Invocation Examples
```bash
# Add a restricted rule as privileged administrator
aiosh pep rule-add --id r_admin --subject agent:admin --resource sys:kernel:module --action load --effect permit --privileged

# Attempting to add a restricted permit rule without --privileged fails with exit code 2
aiosh pep rule-add --id r_hack --subject agent:guest --resource sys:kernel:module --action load --effect permit
# Output: security policy violation: PEPPOL_ERR_PRIVILEGE: unprivileged caller cannot add Permit rule for restricted resource 'sys:kernel:module'
```

### 11.3 Constraints and Known Limitations
1. Unprivileged callers can create `Deny` rules on restricted resources to restrict access, but cannot create `Permit` rules on restricted resources without the `--privileged` flag or administrative capability grant.
2. The default security policy enforces `Enforcing` mode and `Strict` obligation criticality.

---

## 12. Observability Subsystem Reference (Sub-Epic 8)

The PEP Observability subsystem (`aiosh_core::pep_observability`) aggregates point-in-time state, capacity utilization metrics, rule effect distributions, obligation counts, and composite health status.

### 12.1 Key Invariants (`PEPOBS1..PEPOBS6`)
- **`PEPOBS1` (Point-in-Time Metrics Aggregation)**: Accurately reports total rules, counts categorized by decision effect (`permit`, `deny`, `indeterminate`, `not_applicable`), count of rules with obligations, counts broken down by obligation type (`audit_log`, `rate_limit`, `redact_fields`, `custom`), and distinct counts of unique subjects, resources, and actions.
- **`PEPOBS2` (Capacity Utilization Bounds)**: Computes capacity utilization percentage `((total_rules * 100) / MAX_RULES_IN_SERVICE)` bounded strictly to `0..=100%`.
- **`PEPOBS3` (Composite Health Threshold)**: Evaluates `is_healthy` as boolean `capacity_utilization_percent < PEP_HEALTH_UTILIZATION_THRESHOLD` (90%). At or above 90% utilization, health degrades to `false`.
- **`PEPOBS4` (Fail-Closed Structural Invariant Validation)**: Calling `validate()` rejects corrupted or contradictory metrics (such as `sum(rules_by_effect) != total_rules` or `rules_with_obligations > total_rules`) with `PEPOBS_ERR_VALIDATION`.
- **`PEPOBS5` (Telemetry Input Sanitization)**: Telemetry strings (store paths, timestamps) are sanitized by removing ASCII control characters and truncating to 256 characters (`MAX_PEPOBS_TEXT_LEN`).
- **`PEPOBS6` (Deterministic Serialization & Audit Transparency)**: Lossless JSON roundtrip serialization (`to_json()` / `from_json()`) and immutable audit row emission on both CLI and MCP surfaces.

### 12.2 Invocation Examples

#### CLI Usage
```bash
# Generate human-readable PEP observability report
aiosh pep report

# Output structured JSON envelope
aiosh pep report --json

# Generate report against a custom policy store
aiosh pep report --store /custom/path/pep_policies.json --json
```

#### MCP Tool Call
```json
{
  "method": "tools/call",
  "params": {
    "name": "aios.pep.report",
    "arguments": {
      "store_path": ".aios/pep_policies.json"
    }
  }
}
```

### 12.3 Constraints and Known Limitations
1. Health status degrades when rule count exceeds 4,500 (90% of `MAX_RULES_IN_SERVICE = 5000`).
2. Telemetry timestamps default to current UTC ISO 8601 when omitted or invalid.
3. Observability queries are non-destructive and read-only, but still emit an audit event to maintain complete operational observability.






