# T-02408: Sandbox Enforcement Data Model Hardening

## 1. Hardening Measures Implemented
Hardened the data model against adversarial boundary values, memory exhaustion, and unbounded resource requests:
1. **Capacity and Bounds Caps**:
   - `MAX_PATHS_PER_POLICY`: Capped at 256 paths per policy to prevent linear lookup degradation in path resolution loops.
   - `MAX_PATH_LEN`: Capped at 4096 bytes (POSIX `PATH_MAX`).
   - `MAX_PROFILE_NAME_LEN`: Capped at 128 characters.
   - `MAX_ARGS_COUNT`: Capped at 1024 arguments.
   - `MAX_ARG_LEN`: Capped at 65536 bytes per argument.
   - `MAX_ENV_VARS_COUNT`: Capped at 256 environment variables.
   - `MAX_ENV_KEY_LEN`: Capped at 256 bytes per key.
   - `MAX_ENV_VAL_LEN`: Capped at 32768 bytes per value.
2. **Explicit Error Code Taxonomy**:
   - Standardized `ERR_SANDBOX_BOUNDS_EXCEEDED` on any cap breach.
   - Preserved `ERR_SANDBOX_INVALID_PATH`, `ERR_SANDBOX_INVALID_LIMIT`, and `ERR_SANDBOX_POLICY_CONFLICT`.
3. **Environment Policy Validation**:
   - Added validation enforcing size limits and string bounds on environment variable injection and allowlists.

## 2. Invariant Verification
- Verified bounded validation logic in standalone unit test `test_hardening_bounds_exceeded`.
- Verified 0 warnings and zero memory leaks.
