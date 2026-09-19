# T-01664: Security Policy Implementation

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Implement the complete working behavior for the Kernel Module Management Security Policy in `code/aiosh-rust/aiosh-core/src/kernel_module_policy.rs`.

## Implementation Details
1. **Policy Configuration Validation (`validate`)**:
   - Limit checks: `prohibited_modules <= 1024`, `protected_modules <= 256`, `allowed_install_commands <= 32`, `disallowed_parameter_keys <= 128`.
   - Bounds: `max_parameter_value_len` in `[1, 65536]`, `max_rules_per_module` in `[1, 1024]`.
   - Disjointness invariant: ensures that no module is simultaneously configured as both prohibited and protected.
   - Command hygiene: ensures install commands are absolute paths with no traversal (`..`) or whitespace.
2. **Rule Evaluation (`evaluate_rule`)**:
   - **SP-KM1**: Module name validation against `^[a-zA-Z0-9_]+$`.
   - **SP-KM2**: Prohibited modules enforcement — blocks options and aliases for prohibited modules (while allowing explicit blacklist).
   - **SP-KM3**: Protected modules guard — blocks blacklisting or disabling via install `/bin/false` or `/bin/true` for essential kernel drivers (`ext4`, `xfs`, `overlay`, `crypto`, `dm_mod`, `vfat`).
   - **SP-KM4**: Install command sanitization — requires binary to be present in `allowed_install_commands` and strictly rejects shell metacharacters (`;`, `&`, `|`, `` ` ``, `$`) and path traversal.
   - **SP-KM5**: Parameter inspection — validates parameter keys against `disallowed_parameter_keys`, checks length limits against `max_parameter_value_len`, and inspects values for dangerous patterns.
   - **SP-KM6**: Tri-state policy evaluation:
     - `Enforcing`: blocks on any fatal violation.
     - `Audit`: marks `allowed = true` while preserving all violations in the verdict for telemetry.
     - `Permissive`: blocks only critical protected module destruction and command injection.
3. **Autoload Evaluation (`evaluate_autoload`)**:
   - Enforces SP-KM1 (name syntax) and SP-KM2 (blocks autoloading prohibited modules).
4. **Store Evaluation (`evaluate_store`)**:
   - Evaluates all rules and all autoload entries in a `KernelModuleStore`.
5. **Config File & Environment Loading**:
   - `from_file`: bounded by `MAX_POLICY_FILE_BYTES` (64 KiB).
   - `from_source`: parses `AIOS_KERNEL_MODULE_POLICY_MODE`.

## Verification
- Code builds cleanly with zero warnings or errors (`cargo check -p aiosh-core`).
- Artifacts: `docs/tasks/evidence/T-01664-security-policy-implementation.md` and `T-01664-implementation.md`.
