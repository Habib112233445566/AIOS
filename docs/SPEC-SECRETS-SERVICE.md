# Specification: Secrets Handling Core Service (SPEC-SECRETS-SERVICE)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Scope**: Core runtime vault service, scoped authorization gate, lifecycle operations, and atomic persistence.

## 1. Domain Entities & Vault Storage Model

### 1.1 In-Memory Service
```rust
pub struct SecretService {
    entries: HashMap<String, SecretEntry>,
}
```
- Implements `Debug` exposing only `entries_count` to prevent accidental heap dumping.
- Manages collection of `SecretEntry` instances.

### 1.2 On-Disk Vault Persistence Format
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSecretRecord {
    pub metadata: SecretMetadata,
    pub payload_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPayload {
    pub version: String,
    pub secrets: HashMap<String, StoredSecretRecord>,
}
```

## 2. Service Operations & Privilege Matrix

### 2.1 Core Operations
- `store_secret(&mut self, entry: SecretEntry) -> Result<(), String>`
- `get_secret(&self, id: &str, caller_scope: &SecretScope) -> Result<SecretValue, String>`
- `get_secret_with_privilege(&self, id: &str, priv_ctx: &PrivilegeContext) -> Result<SecretValue, String>`
- `get_metadata(&self, id: &str) -> Result<SecretMetadata, String>`
- `list_metadata(&self, filter_kind: Option<SecretKind>, filter_scope: Option<&SecretScope>) -> Vec<SecretMetadata>`
- `rotate_secret(&mut self, id: &str, new_value: &[u8]) -> Result<(), String>`
- `revoke_secret(&mut self, id: &str) -> Result<(), String>`
- `save_to_path(&self, path: &Path) -> Result<(), String>`
- `load_from_path(path: &Path) -> Result<Self, String>`

### 2.2 Privilege Access Matrix
| Privilege Tier | Global Scope | Environment Scope | Matching Actor Scope | Foreign Actor Scope |
|---|---|---|---|---|
| **SystemKernel / Admin** | PERMIT | PERMIT | PERMIT | PERMIT |
| **Operator** | PERMIT | PERMIT | PERMIT | DENY |
| **User** | PERMIT | DENY | PERMIT | DENY |
| **Guest** | DENY | DENY | DENY | DENY |

## 3. Invariants
- `SECSVC1`: Scoped access gate checks `caller_scope.allows(&secret.metadata.scope)`. Returns `SECSVC_ERR_ACCESS_DENIED` if not permitted.
- `SECSVC2`: Expired or Revoked secrets cannot be read via `get_secret`. Returns `SECSVC_ERR_INACCESSIBLE`.
- `SECSVC3`: Maximum vault capacity is 1,024 secrets (`MAX_SECRETS_VAULT_CAPACITY`).
- `SECSVC4`: Maximum store file size is 1 MiB (`MAX_SECRETS_STORE_SIZE`).
- `SECSVC5`: Persistence writes use atomic write-and-rename (`.tmp.<pid>`) with immediate unlink on error.
- `SECSVC6`: Path traversal (`..`) and symbolic links (`symlink_metadata`) are rejected on all store paths.
- `SECSVC7`: `list_metadata` never exposes secret payloads or hex bodies.
