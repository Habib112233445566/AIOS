# Specification: Secrets Handling CLI Surface (SPEC-SECRETS-CLI)

- **Status**: APPROVED & IMPLEMENTED
- **Date**: 2026-10-02
- **Scope**: Command-line interface subcommands for runtime secret vaulting, scoped retrieval, rotation, and revocation.
- **Reference Tasks**:
  - Research: [T-02621](file:///docs/tasks/evidence/T-02621-cli-surface-research.md)
  - Specification: [T-02622](file:///docs/tasks/evidence/T-02622-cli-surface-specification.md)
  - Scaffold: [T-02623](file:///docs/tasks/evidence/T-02623-cli-surface-scaffold.md)
  - Implementation: [T-02624](file:///docs/tasks/evidence/T-02624-cli-surface-implementation.md)
  - Unit Test: [T-02625](file:///docs/tasks/evidence/T-02625-cli-surface-unit-test.md)
  - Integration: [T-02626](file:///docs/tasks/evidence/T-02626-cli-surface-integration.md)
  - Security Review: [T-02627](file:///docs/tasks/evidence/T-02627-cli-surface-security-review.md)
  - Hardening: [T-02628](file:///docs/tasks/evidence/T-02628-cli-surface-hardening.md)
  - Documentation: [T-02629](file:///docs/tasks/evidence/T-02629-cli-surface-documentation.md)

---

## 1. Command Syntax & Subcommands

Aliases: `aiosh secret` or `aiosh sec`.

### 1.1 `aiosh secret store`
Registers or updates a secret in the vault.
```bash
aiosh secret store --id <ID> --name <NAME> --kind <KIND> [--scope <SCOPE>] [--target <TARGET>] [--value <VAL>] [--store <PATH>] [--json]
```
Example:
```bash
aiosh secret store --id db_prod_pass --name "Production DB Password" --kind database_credential --scope environment --target production --value "super_secure_pass_123" --store vault.json --json
```

### 1.2 `aiosh secret get`
Fetches a secret by ID with automatic terminal masking.
```bash
aiosh secret get --id <ID> [--scope <SCOPE>] [--target <TARGET>] [--expose] [--store <PATH>] [--json]
```
Example (Masked default):
```bash
aiosh secret get --id db_prod_pass --store vault.json
# Output:
# Secret 'db_prod_pass':
#   Name:    Production DB Password
#   Kind:    database_credential
#   Scope:   environment:production
#   State:   active
#   Value:   supe..._123
```
Example (Exposed plaintext):
```bash
aiosh secret get --id db_prod_pass --expose --store vault.json
# Output:
# Secret 'db_prod_pass':
#   Name:    Production DB Password
#   Kind:    database_credential
#   Scope:   environment:production
#   State:   active
#   Value:   super_secure_pass_123
```

### 1.3 `aiosh secret list`
Lists all stored secrets metadata (zero payload disclosure).
```bash
aiosh secret list [--kind <KIND>] [--scope <SCOPE>] [--target <TARGET>] [--store <PATH>] [--json]
```
Example:
```bash
aiosh secret list --kind database_credential --store vault.json --json
```

### 1.4 `aiosh secret rotate`
Rotates an existing secret payload and increments version.
```bash
aiosh secret rotate --id <ID> --value <NEW_VAL> [--store <PATH>] [--json]
```
Example:
```bash
aiosh secret rotate --id db_prod_pass --value "new_rotated_password_987" --store vault.json
# Output:
# Secret 'db_prod_pass' rotated to version 2.
```

### 1.5 `aiosh secret revoke`
Revokes an active secret, preventing future retrieval.
```bash
aiosh secret revoke --id <ID> [--store <PATH>] [--json]
```
Example:
```bash
aiosh secret revoke --id db_prod_pass --store vault.json --json
# Output:
# {"code":0,"data":{"id":"db_prod_pass","state":"revoked"},"error":null}
```

---

## 2. Argument Specifications & Exit Codes
- `--id`: 1..128 ASCII characters matching `^[a-zA-Z0-9_\-]+$`.
- `--name`: 1..256 non-empty characters without ASCII control codes.
- `--kind`: One of `api_key`, `oauth_token`, `database_credential`, `private_key`, `tls_certificate`, `symmetric_key`, `generic`.
- `--scope`: One of `global`, `environment`, `actor`, `session`.
- `--target`: Non-empty string required for non-global scopes.
- `--expose`: Explicit opt-in flag required to display plaintext secrets.
- `--store`: Optional custom file path to vault JSON.

### Exit Codes
- `0`: Success.
- `1`: Operation error (access denied, not found, revoked/expired, IO failure).
- `2`: Syntax or missing required argument error, path traversal rejection.

---

## 3. Constraints & Limitations
1. **Payload Size Limit**: Individual secret payloads are capped at 64 KiB (`MAX_SECRET_PAYLOAD_SIZE`).
2. **Vault Capacity Limit**: A single vault file can store up to 1,024 secret entries.
3. **Store File Size**: Vault JSON store files are limited to 1 MiB (`1,048,576` bytes).
4. **Scope Gating**: Plaintext retrieval requires caller scope to satisfy scope containment hierarchy (`global > environment > actor > session`).
5. **Phase 2 Scope**: Local file-backed atomic store with SHA-256 fingerprinting. Hardware TPM/HSM binding is scheduled for Phase 3.
