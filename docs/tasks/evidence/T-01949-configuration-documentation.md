# Task Evidence: T-01949 (System Update / configuration: Documentation)

## Summary
Documented the System Update Configuration & Policy Subsystem in Section 8 of `docs/system_update.md`.
The documentation covers:
1. **Configuration Architecture & Data Model**: Detailed specification of `SystemUpdateConfig`, field types, default values, and operational semantics (`update_channel`, `poll_interval_secs`, `auto_apply`, `storage_path`, `max_payload_size_bytes`, `signature_verification_required`, `trusted_public_keys`, `require_rollback_protection`, `maintenance_window_*`, `rate_limit_bytes_per_sec`, `min_free_space_bytes`).
2. **Environment Variable Overrides (`AIOSH_UPDATE_*`)**: Mappings, parsing rules, boundary enforcement, and validation error propagation.
3. **Invariants Enforced (UCONF1 - UCONF6)**:
   - `UCONF1`: Channel Validity (`stable`, `beta`, `nightly`).
   - `UCONF2`: Poll Interval Boundaries ($60 \le \text{poll\_interval\_secs} \le 2,592,000$).
   - `UCONF3`: Path Length and Traversal Hygiene (max 1024 chars, no `..`, no control chars, canonicalized path check).
   - `UCONF4`: Payload Size Limits ($1\,\text{MB} \le \text{max\_payload\_size\_bytes} \le 10\,\text{GB}$).
   - `UCONF5`: Cryptographic Key Trust Set ($1 \le \text{trusted\_public\_keys} \le 32$ when verification required).
   - `UCONF6`: Maintenance Window Validity ($0 \le \text{start/end} \le 23$).
4. **Hardened Persistence Protocol**: Atomic write using `.tmp.<pid>` files, rename semantics, 1 MB configuration size bounds check, and symlink rejection (`symlink_metadata`).
5. **Runtime Integration**: How `aiosh-mcp` and `aiosh-cli` consume `SystemUpdateConfig` via `to_service_config()` into `SystemUpdateService`.

## Associated Evidence Files
- `docs/system_update.md`: Section 8 "System Update Configuration & Policy Subsystem"
- `docs/tasks/evidence/T-01949-documentation.md`
