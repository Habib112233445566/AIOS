# Security Review: Capability Security Policy (T-02067)

## Executive Summary
This security review evaluates the **Capability Security Policy** subsystem (`CAPSEC1..CAPSEC6`) implemented in `code/aiosh-rust/aiosh-core/src/capability_policy.rs` and integrated into `CapabilityService` (`capability_service.rs`) and `aiosh-mcp`.

---

## 1. Threat Model & Abuse Scenarios

### Scenario 1: Path Traversal & Normalization Evasion (`THREAT-CAPSEC-01`)
- **Attack Vector**: An attacker requests a capability with path variants such as `//etc/shadow`, `/etc/../etc/shadow`, `/etc/.`, or `C:\Windows\..\Windows\System32` to evade prefix-based prohibition checks.
- **Analysis**:
  - `validate()` in `CapabilitySecurityPolicy` rejects `..` in policy prefix configuration.
  - However, in `evaluate_issuance`, `scope.path` is evaluated. If `scope.path` contains `..` or unnormalized `/` sequences, prefix matching could potentially be circumvented unless paths are normalized prior to evaluation.
  - Furthermore, `validate_service_path` in `CapabilityService` already checks for `..` in store paths, but `CapabilityScope::Filesystem.path` requires canonicalization/normalization (e.g. collapsing redundant slashes, dot-dot resolution).
- **Hardening Requirement (for T-02068)**:
  - Add explicit path normalization in `evaluate_issuance` (e.g., stripping duplicate slashes, checking for `..` in the requested scope path, and rejecting path traversal characters).

### Scenario 2: SSRF & Host Obfuscation (`THREAT-CAPSEC-02`)
- **Attack Vector**: An attacker crafts network scope requests to `http://169.254.169.254:80/` using bracketed IPv4 `[169.254.169.254]`, IPv6 equivalents `::ffff:169.254.169.254`, alternative radix, or port appending (`169.254.169.254:80` in `host` field).
- **Analysis**:
  - `CapabilityScope::Network` separates `host` and `port`.
  - Case-insensitive comparison `h.eq_ignore_ascii_case(host)` handles ASCII case variations (`Metadata.Google.Internal`).
- **Hardening Requirement (for T-02068)**:
  - Add host stripping (strip trailing dots, strip square brackets, strip any accidental `:port` suffix if provided in host field).

### Scenario 3: Derivation Tree Cycles & Infinite Loops (`THREAT-CAPSEC-03`)
- **Attack Vector**: A corrupted or manipulated capability store introduces a cycle in parent pointers (e.g., $A \rightarrow B \rightarrow A$).
- **Analysis**:
  - `CapabilityService::get_derivation_depth` traverses parent links:
    ```rust
    while let Some(cap) = self.capabilities.get(&current_id) {
        if let Some(ref parent_id) = cap.parent_id {
            depth += 1;
            current_id = parent_id.clone();
        } else { break; }
    }
    ```
    If a cycle exists, `depth` increments indefinitely until memory/time exhaustion.
- **Hardening Requirement (for T-02068)**:
  - Add a visited set or cycle detection (`HashSet<String>`) in `get_derivation_depth` and cap maximum traversal iterations to prevent infinite loops.

### Scenario 4: Subject Prefix Spoofing (`THREAT-CAPSEC-04`)
- **Attack Vector**: An untrusted actor crafts subject names like `untrusted_admin` or `guest_root` or attempts to bypass disallowed rights by prefix manipulation.
- **Analysis**:
  - `subject.starts_with(prefix)` is used. If prefix is `"untrusted"`, both `"untrusted:agent"` and `"untrusted_agent"` match and are denied dangerous rights (`Admin`, `Delegate`, `Delete`). This is fail-closed.
  - Adding explicit colon delimiter matching (e.g. checking `prefix` or `format!("{}:", prefix)`) ensures clean classification.

### Scenario 5: Temporal Boundary & Clock Skew Abuse (`THREAT-CAPSEC-05`)
- **Attack Vector**: Capabilities specify `not_before` in the far future or `expires_at` with negative durations or parsing anomalies.
- **Analysis**:
  - `DateTime::parse_from_rfc3339` validates RFC3339 formatting.
  - If `expires_at < now`, duration is negative, which is correctly detected as expired during validity checks.
  - In `evaluate_issuance`, `dur.num_seconds() > max_dur` detects excessive duration.

---

## 2. PEP Gating & Audit Logging Compliance
- Every capability tool in `aiosh-mcp` (`aios.capability.issue`, `aios.capability.attenuate`, etc.) executes via `dispatch::recorded_call`, ensuring:
  1. PEP grant verification prior to execution.
  2. SHA-256 hash-chained audit ring emission into SQLite WAL.
  3. Structured error return in standard JSON-RPC envelope on policy violation.

---

## 3. Review Verdict
- **Status**: Review completed. Abuse scenarios identified.
- **Action Items**: Hardening tasks defined for `T-02068` (path normalization, cycle detection in depth calculation, host sanitization).
