# T-02597: Privilege Escalation Prevention Recovery & Validation Security Review

- **Task**: `T-02597`
- **Sub-Epic**: Privilege Escalation Prevention / recovery & validation
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Threat Modeling & Abuse Scenarios
1. **Path Traversal via Store Path Argument**:
   - *Threat*: Attacker supplies paths like `../../etc/shadow` or `C:\Windows\System32\config\SAM` to overwrite or quarantine sensitive system files.
   - *Mitigation*: `PrivilegeRecoveryManager::validate_path` enforces strict path hygiene: rejects any path containing `..`, control characters, or exceeding 1024 characters.
2. **Denial of Service via File Bomb**:
   - *Threat*: Malicious actor points recovery manager to a multi-gigabyte file to cause out-of-memory crashes.
   - *Mitigation*: Hard file size ceiling enforced via `fs::metadata()`: `MAX_PRIVILEGE_STORE_SIZE = 1,048,576` bytes (1 MiB). Oversized files are immediately rejected fail-closed.
3. **Privilege Escalation via Corrupted State Injection**:
   - *Threat*: Malicious local process writes crafted JSON setting `active_level: "system_kernel"`, hoping recovery will accept it or preserve it.
   - *Mitigation*: Diagnostic validation flags `SystemKernel` as a FATAL `IllegalKernelTier` issue. Self-healing repair strictly demotes any kernel-tier context to `User` and clears active elevation grants.
4. **Data Loss During In-Place Repair**:
   - *Threat*: Machine power loss during store rewriting causes total loss of valid actor contexts.
   - *Mitigation*: Timestamped backup (`.bak.<timestamp>`) is created before reading and modifying file contents. Rewriting uses atomic temporary file replacement (`save_to_path()`).
