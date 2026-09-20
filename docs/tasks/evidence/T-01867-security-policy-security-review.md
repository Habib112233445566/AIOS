# Task Evidence: T-01867 - Network Bootstrap / security policy: Security Review

## 1. Overview
- **Task ID**: `T-01867`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Perform comprehensive security review of `NetworkSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/network_policy.rs`.

---

## 2. Threat Modeling & Abuse Scenarios

| Threat ID | Threat Vector | Mechanism / Attack Surface | Mitigation / Invariant | Status |
|---|---|---|---|---|
| `THREAT-NPOL-01` | Path Traversal & Injection | Malicious `policy_path` supplied with `..`, control characters, or excessive length | `validate_policy_path` rejects paths with `ParentDir`, length > 1024, or control characters (`NPOL6`) | Mitigated |
| `THREAT-NPOL-02` | Name Normalization & Bypass | Interface names with trailing whitespace, leading spaces, or unusual casing in prohibition/whitelists | Interface names validated with `trim().is_empty()` check and max length 15 (`NPOL1`) | Mitigated |
| `THREAT-NPOL-03` | Memory Exhaustion / Resource DoS | Giant policy documents or unbounded rule lists causing OOM in `aiosh-core` | `MAX_POLICY_FILE_BYTES` (1 MB) file cap, plus limits on lists (prohibited <= 1,000, allowed <= 1,000) (`NPOL4`, `NPOL6`) | Mitigated |
| `THREAT-NPOL-04` | Atomic File Tampering & Leaks | Unlinked temporary sibling files or race conditions during policy persistence | Temp sibling `.{name}.tmp.{pid}` written with Unix `0600` permissions; error branches unlink temporary files (`NPOL6`) | Mitigated |
| `THREAT-NPOL-05` | Topology & Hardware Address Leakage | Exposing exact MAC and IP addresses in telemetry or non-isolated logs | `apply_and_sanitize()` provides deterministic masking of MAC (`prefix:xx:xx:xx`) and IPv4 (`prefix.xxx`) (`NPOL5`) | Mitigated |

---

## 3. Findings & Recommendations for Hardening (T-01868)
1. In `NetworkSecurityPolicy::validate()`, ensure `prohibited_interface_names` and `allowed_interface_names` entries are trimmed during matching in `evaluate()` to prevent whitespace padding evasion.
2. In `save_to_path()`, ensure temp file cleanup is robust across all early-return paths.
3. Ensure IPv6 addresses are safely redacted in addition to IPv4 in `apply_and_sanitize()` if present.
