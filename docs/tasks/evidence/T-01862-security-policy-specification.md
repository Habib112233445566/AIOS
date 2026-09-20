# Task Evidence: T-01862 - Network Bootstrap / security policy: Specification

## 1. Overview
- **Task ID**: `T-01862`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Formally specify `NetworkSecurityPolicy`, evaluation semantics, violation rules, persistence contracts, and invariants `NPOL1..NPOL6`.

---

## 2. Specification: Data Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSecurityPolicy {
    pub mode: NetworkPolicyMode,
    pub disallowed_interface_types: Vec<InterfaceType>,
    pub prohibited_interface_names: Vec<String>,
    pub allowed_interface_names: Option<Vec<String>>,
    pub allow_promiscuous: bool,
    pub require_mac_for_ethernet: bool,
    pub disallowed_dns_servers: Vec<String>,
    pub allowed_dns_servers: Option<Vec<String>>,
    pub max_interfaces_allowed: usize,
    pub max_routes_allowed: usize,
    pub max_dns_servers_allowed: usize,
    pub redact_sensitive_addresses: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkPolicyViolation {
    pub rule_id: String,
    pub target: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPolicyReport {
    pub verdict: String, // "allow", "deny", "audit"
    pub mode: NetworkPolicyMode,
    pub violations: Vec<NetworkPolicyViolation>,
    pub interfaces_evaluated: usize,
    pub routes_evaluated: usize,
    pub dns_servers_evaluated: usize,
}
```

---

## 3. Evaluation Rules & Decision Logic

### Rules (`NPOL1..NPOL5`)
| Rule ID | Invariant | Target | Condition | Fatal |
|---|---|---|---|---|
| `RULE_IFACE_MAX_CAP` | `NPOL4` | Global | `interfaces.len() > max_interfaces_allowed` | Yes |
| `RULE_IFACE_DISALLOWED_TYPE` | `NPOL1` | Interface | `iface.interface_type` in `disallowed_interface_types` | Yes |
| `RULE_IFACE_PROHIBITED_NAME` | `NPOL1` | Interface | `iface.name` in `prohibited_interface_names` | Yes |
| `RULE_IFACE_NOT_WHITELISTED` | `NPOL1` | Interface | `allowed_interface_names` present and `iface.name` not in list | Yes |
| `RULE_IFACE_PROMISCUOUS` | `NPOL1` | Interface | Flag `PROMISC` set and `!allow_promiscuous` | Yes |
| `RULE_IFACE_MISSING_MAC` | `NPOL1` | Interface | Type is Ethernet, MAC is None, and `require_mac_for_ethernet` is true | Yes |
| `RULE_ROUTE_MAX_CAP` | `NPOL4` | Global | `routes.len() > max_routes_allowed` | Yes |
| `RULE_ROUTE_ORPHAN_IFACE` | `NPOL2` | Route | `route.interface` not found in `state.interfaces` | Yes |
| `RULE_DNS_MAX_CAP` | `NPOL4` | Global | `dns.nameservers.len() > max_dns_servers_allowed` | Yes |
| `RULE_DNS_DISALLOWED_SERVER` | `NPOL3` | DNS | Server IP in `disallowed_dns_servers` | Yes |
| `RULE_DNS_NOT_WHITELISTED` | `NPOL3` | DNS | `allowed_dns_servers` present and IP not in list | Yes |

### Verdict Determination
- **`Enforcing` Mode**:
  - Any fatal violation $\implies$ `verdict = "deny"`.
  - Zero fatal violations $\implies$ `verdict = "allow"`.
- **`Audit` Mode**:
  - Any fatal violation $\implies$ `verdict = "audit"`.
  - Zero fatal violations $\implies$ `verdict = "allow"`.
- **`Permissive` Mode**:
  - Always `verdict = "allow"` (violations collected for telemetry).

---

## 4. Policy File Persistence & Path Hygiene (`NPOL6`)
- `validate_policy_path(path)`: Rejects empty strings, strings $> 1024$ chars, control characters, and parent directory traversal (`..`).
- `MAX_POLICY_FILE_BYTES = 1_048_576` (1 MB limit enforced via metadata before reading).
- `save_to_path(path)`: Uses atomic sibling temporary file `.{name}.tmp.{pid}` and rename.
