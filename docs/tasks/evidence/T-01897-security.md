# Task Evidence: T-01897 - Network Bootstrap / recovery & validation: Security Review

## 1. Overview
- **Task ID**: `T-01897`
- **Sub-Epic**: 10 (Network Bootstrap Recovery & Validation)
- **Goal**: Perform comprehensive security review and threat modeling of `network_recovery.rs` in `aiosh-core`.

---

## 2. Threat Modeling & Risk Assessment

| Threat ID | Category | Description | Severity | Mitigation Status |
|---|---|---|---|---|
| `THREAT-NVAL-01` | Backup Collision / Overwrite | Rapid recovery triggers generating identical timestamps within the same second colliding and overwriting prior forensic `.bak` files. | Medium | Needs Hardening: Add process ID or nanosecond / counter entropy to backup path. |
| `THREAT-NVAL-02` | DNS Resolver Poisoning | Injecting invalid, malformed, or untrusted resolver IP strings during automatic DNS fallback recovery. | High | Defended: Hardcoded RFC 1918/RFC 5737-safe canonical resolvers (`1.1.1.1`, `8.8.8.8`), validated via IP parser. |
| `THREAT-NVAL-03` | Loopback Spoofing & Address Conflicts | Recovering `lo` interface with non-canonical IP addresses or conflicting netmasks interfering with local host IPC. | High | Defended: Strictly hardcoded canonical loopback addresses (`127.0.0.1/8`, `::1/128`) and `InterfaceType::Loopback`. |
| `THREAT-NVAL-04` | Partial Write / State Corruption | Process interruption during recovery state write leaving partially written, corrupted files on disk. | High | Defended: Sibling temporary file atomic swap via `.{name}.tmp.{pid}` with RAII `TempFileGuard` and Unix `0600` permissions. |
| `THREAT-NVAL-05` | File Size Exhaustion / Memory DoS | Reading or writing multi-megabyte malformed network state stores consuming excessive system memory. | Medium | Defended: `MAX_NETWORK_STORE_SIZE = 1,048,576` (1 MB) checked via metadata prior to buffer allocation. |
| `THREAT-NVAL-06` | Path Traversal & Injection | Untrusted store paths containing `..` or control characters escaping system network directories. | High | Defended: `validate_network_store_path` strictly rejects `ParentDir`, control characters, non-`.json` paths, and paths $> 1024$ characters. |

---

## 3. Actionable Hardening Plan for T-01898
1. **Collision-Resistant Quarantine Naming**: Enhance `.bak` file generation to include process ID and sub-second timestamp (`%Y%m%d_%H%M%S_%f`), ensuring forensic integrity even during rapid automated recovery cycles.
2. **DNS Resolver Validation**: Add explicit IP address format validation check on injected fallback resolvers before modifying in-memory state.
3. **Hardening Verification Suite**: Add test cases in `test_network_recovery.rs` verifying sub-second quarantine collision resistance.

Status: Security review complete. Ready for hardening in `T-01898`.
