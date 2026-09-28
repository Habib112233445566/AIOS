# T-02468: Sandbox Enforcement Security Policy Hardening

## 1. Hardening Overview
This task hardens `SandboxSecurityPolicy` file I/O operations against directory traversal, symlink attacks, and corrupted or oversized policy payloads.

---

## 2. Hardening Measures Implemented
1. **Directory Traversal Prevention**:
   - Both `load_from_path` and `save_to_path` reject paths containing `..` with `SANDBOXPOL_ERR_VALIDATION`.
2. **ASCII Control Character Rejection**:
   - Rejects policy paths containing ASCII control characters (`< 32` or null bytes).
3. **Symlink Rejection**:
   - `load_from_path` inspects `fs::symlink_metadata(p)`. If the target is a symlink, execution fails closed, mitigating symlink manipulation attacks (CWE-59).
4. **Size Cap Enforcement**:
   - Caps policy payload size at `MAX_SANDBOX_SECURITY_POLICY_BYTES` (64 KiB).
5. **Atomic Serialization**:
   - Parent directory creation (`fs::create_dir_all`) and atomic UTF-8 writes prevent partial or corrupted file state.

---

## 3. Test Verification
All 7 unit tests pass in 0.06s.
