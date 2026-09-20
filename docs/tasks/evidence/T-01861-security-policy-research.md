# Task Evidence: T-01861 - Network Bootstrap / security policy: Research

## 1. Overview
- **Task ID**: `T-01861`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Research security policy requirements, policy enforcement modes, threat vectors, evaluation contracts, and formulate invariants `NPOL1..NPOL6`.

---

## 2. Facts vs. Assumptions

### Facts
1. Subsystems across `aiosh-core` (`hardware_policy`, `service_policy`, `kernel_module_policy`, `package_policy`, `session_policy`) consistently implement:
   - Three enforcement modes: `Enforcing`, `Audit`, and `Permissive`.
   - Structured violation objects with rule IDs, target identifiers, descriptions, and fatal flags.
   - Comprehensive policy report objects summarizing total elements evaluated, violations, and final verdict (`allow`, `deny`, `audit`).
   - File hygiene checks (`validate_policy_path`) and a 1 MB file size cap (`MAX_POLICY_FILE_BYTES`).
2. Network interfaces, routing tables, and DNS settings are high-value targets for host persistence, exfiltration, traffic hijacking, and sniffing.
3. Promiscuous mode (`IFF_PROMISC` / flag `0x100`) on an interface indicates active packet sniffing and must be strictly governed by policy.

### Assumptions
1. `NetworkSecurityPolicy` will be implemented in `code/aiosh-rust/aiosh-core/src/network_policy.rs` and re-exported in `code/aiosh-rust/aiosh-core/src/lib.rs`.
2. Policy evaluation takes a reference to `NetworkState` and produces a `NetworkPolicyReport`.
3. Policy can optionally redact IP/MAC addresses from reports for untrusted consumer viewing.

---

## 3. Decisions Needed & Invariants Formulation

### Invariants (`NPOL1..NPOL6`)
1. **`NPOL1` (Interface Authorization & Promiscuous Mode Control)**:
   - Evaluates interface names against `prohibited_interface_names` and `allowed_interface_names` (if specified).
   - Rejects interfaces with types in `disallowed_interface_types`.
   - Detects `PROMISC` flag; rejects promiscuous interfaces unless `allow_promiscuous` is explicitly `true`.
   - Verifies MAC address presence on Ethernet interfaces if `require_mac_for_ethernet` is `true`.
2. **`NPOL2` (Route Governance)**:
   - Flags suspicious route metrics or invalid default gateways.
   - Rejects routes referencing unauthorized or down interfaces.
3. **`NPOL3` (DNS Resolver Whitelist/Denylist)**:
   - Evaluates configured DNS nameservers against `disallowed_dns_servers` and `allowed_dns_servers` (if specified).
   - Flags non-standard or unapproved DNS resolvers.
4. **`NPOL4` (Resource & Quota Limits)**:
   - Enforces `max_interfaces_allowed` (default: 1024, cap: 10,000).
   - Enforces `max_routes_allowed` (default: 4096, cap: 50,000).
   - Enforces `max_dns_servers_allowed` (default: 32, cap: 64).
5. **`NPOL5` (Sanitization & Sensitive Attribute Redaction)**:
   - When `redact_sensitive_addresses` is enabled, sanitizes IP host portions or MAC bytes in user-facing reports.
6. **`NPOL6` (Policy Path Hygiene & Atomic Serialization)**:
   - Validates policy paths ($\le 1024$ chars, no `..`, no control characters).
   - Enforces `MAX_POLICY_FILE_BYTES = 1,048,576` (1 MB) to prevent OOM DoS during policy parsing.

---

## 4. Next Steps
- Formally specify `NetworkSecurityPolicy`, `NetworkPolicyMode`, `NetworkPolicyViolation`, and `NetworkPolicyReport` in `T-01862`.
- No code modified during this research phase.
