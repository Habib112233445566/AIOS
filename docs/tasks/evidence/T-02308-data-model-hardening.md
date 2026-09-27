# T-02308: Audit Chain Extensions Data Model Hardening

## Overview
This task documents the defensive hardening and resilience guarantees implemented for the Audit Chain Extensions data model and SQLite persistence integration.

## Hardening Controls

### 1. Structural Limits & Memory Bounds
- `MAX_CAUSAL_LINKS` = 16: Bounds DAG fan-out per audit record to prevent exponential graph expansion during retrospective audits.
- `MAX_EXTENSION_ENTRIES` = 32: Limits dictionary keys to prevent hash collision attacks or excessive JSON parsing overhead.
- `MAX_EXTENSION_PAYLOAD_BYTES` = 65,536 (64 KB): Hard byte cap on serialized extension metadata.
- Identifier bounds: `MAX_SESSION_ID_LEN` = 128, `MAX_TRACE_ID_LEN` = 128, preventing buffer stuffing.

### 2. Standardized Error Envelope
Validation and integrity failures emit typed standard error prefixes:
- `AUDIT_EXT_ERR_VALIDATION`: Missing or illegal format in required fields.
- `AUDIT_EXT_ERR_HASH`: Cryptographic SHA-256 hash or prev_hash mismatch.
- `AUDIT_EXT_ERR_SIGNATURE`: Malformed public key, empty algorithm, or invalid signature length.
- `AUDIT_EXT_ERR_BOUNDS`: Capacity exceedance on links, extensions, or payloads.

### 3. Resource & Database Protection
- Query limits in `tail_extended` clamped between 1 and 1024 (`safe = n.clamp(1, 1024)`).
- Full SQLite WAL durability and parameterized binding to prevent SQL injection.
- Zero heap-leak guarantees through standard RAII resource management.
