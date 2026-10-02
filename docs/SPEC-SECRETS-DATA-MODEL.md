# Specification: Secrets Handling Data Model (SPEC-SECRETS-DATA-MODEL)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Scope**: Core data model, memory protection, lifecycle states, and scoped isolation for Phase 2 Secrets Handling.

## 1. Domain Entities & Enums

### 1.1 SecretKind
Classification of secret payload:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    ApiKey,
    OAuthToken,
    DatabaseCredential,
    PrivateKey,
    TlsCertificate,
    SymmetricKey,
    Generic,
}
```

### 1.2 SecretScope
Scope boundary under which a secret is authorized:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "target")]
pub enum SecretScope {
    Global,
    Environment(String),
    Actor(String),
    Session(String),
}
```

### 1.3 SecretState
Operational lifecycle state machine:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretState {
    Active,
    Rotated,
    Revoked,
    Expired,
}
```
State Transition Rules:
- `Active` $\rightarrow$ `Rotated`, `Revoked`, `Expired`
- `Rotated` $\rightarrow$ `Revoked`, `Expired`
- `Revoked` $\rightarrow$ Terminal (No transitions permitted)
- `Expired` $\rightarrow$ Terminal (No transitions permitted)

### 1.4 SecretMetadata
Publicly auditable, safe-to-serialize metadata:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: SecretKind,
    pub scope: SecretScope,
    pub state: SecretState,
    pub version: u32,
    pub fingerprint: String, // hex-encoded SHA-256 of plaintext
    pub created_at: String,  // RFC3339
    pub updated_at: String,  // RFC3339
    pub expires_at: Option<String>,
    pub labels: HashMap<String, String>,
}
```

### 1.5 SecretValue
Protected secret payload with zeroization on drop:
```rust
pub struct SecretValue {
    bytes: Vec<u8>,
}
```
- Implements `Drop` with volatile writes and compiler barriers (`compiler_fence(Ordering::SeqCst)`).
- Implements constant-time equality check (`constant_time_eq`).
- Implements safe masked representation (`masked_display`).

### 1.6 SecretEntry
Vault storage record pairing metadata with protected value:
```rust
pub struct SecretEntry {
    pub metadata: SecretMetadata,
    pub value: SecretValue,
}
```

## 2. Invariants
- `SECDATA1`: `id` must be 1..64 characters matching `^[a-zA-Z0-9_\-]+$`.
- `SECDATA2`: `name` must be 1..128 characters, non-empty, and free of control characters.
- `SECDATA3`: Payload size bounded at 64 KiB (65,536 bytes) maximum (`MAX_SECRET_PAYLOAD_SIZE`).
- `SECDATA4`: `SecretValue` memory is volatile-zeroized upon `Drop` flanked by compiler fences to prevent sensitive material leaking into heap dumps.
- `SECDATA5`: SHA-256 fingerprint hex string computed deterministically from plaintext upon creation.
- `SECDATA6`: Masked display format: if length $\ge 12$, show `prefix(4) + "..." + suffix(4)`; otherwise show `"********"`.
- `SECDATA7`: Equality comparison on `SecretValue` executes in constant time with respect to payload length.
- `SECDATA8`: Metadata `labels` strictly bounded: max 32 entries, max 64 chars per key, max 256 chars per value, no control characters.
