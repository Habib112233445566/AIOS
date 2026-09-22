# Task Evidence: T-02224 (Grant Lifecycle / CLI surface: Implementation)

## 1. Scope & Execution
Implemented the complete CLI surface for Grant Lifecycle management (`aiosh pep grant <issue|list|inspect|validate|attenuate|revoke|sweep>`) in `code/aiosh-rust/aiosh-cli/src/main.rs`:
1. **`issue`**: Full parameter parsing (`--id`, `--subject`, `--scope-type`, `--rights`, `--issuer`, `--scope-path`, `--expires-at`, `--not-before`, `--max-invocations`, `--max-bytes`, `--delegation-depth`), validation, atomic write, and structured emission.
2. **`list`**: Multi-index lookup with optional filtering by `--subject` and `--state`.
3. **`inspect`**: Full JSON / formatted tabular grant inspection including constraints and revocation records.
4. **`validate`**: Authorization check against subject, action right, and optional `--now` timestamp via `PepGrantService` evaluation.
5. **`attenuate`**: Derivation of child grants with reduced rights and delegation depth decrement.
6. **`revoke`**: Revocation with operator attribution reason and optional `--cascade` across descendant hierarchy.
7. **`sweep`**: Temporal expiration and quota exhaustion sweep with optional reference timestamp.
8. **Structured Error Envelopes & Audit Logging**:
   - Every subcommand emits structured audit telemetry (`classify_and_emit`).
   - Supports `--json` flag outputting `{ "code": i32, "data": Value, "error": Value }`.
   - All errors sanitized with `sanitize_terminal` to prevent escape sequence injection.

---

## 2. Test Execution Output
```
> aiosh pep grant issue --id g-cli-1 --subject alice --scope-type filesystem --scope-path /tmp --rights read,write,delegate --store g.json --json
issue: 0 {"code":0,"data":{"constraints":{"bytes_used":0,"expires_at":null,"invocations_used":0,"max_bytes":null,"max_delegation_depth":2,"max_invocations":null,"not_before":null},"id":"g-cli-1","issuer":"operator","rights":["read","write","delegate"],"scope":{"details":{"path":"/tmp","recursive":true},"type":"filesystem"},"state":"active","subject":"alice"},"error":null}

> aiosh pep grant attenuate g-cli-1 --child g-cli-child --subject bob --rights read --store g.json --json
attenuate: 0 {"code":0,"data":{"constraints":{"bytes_used":0,"expires_at":null,"invocations_used":0,"max_bytes":null,"max_delegation_depth":1,"max_invocations":null,"not_before":null},"id":"g-cli-child","issuer":"alice","parent_grant_id":"g-cli-1","rights":["read"],"scope":{"details":{"path":"/tmp","recursive":true},"type":"filesystem"},"state":"active","subject":"bob"},"error":null}

> aiosh pep grant list --store g.json --json
list: 0 {"code":0,"data":{"count":2,"grants":[...]},"error":null}
```

---

## 3. Acceptance Confirmation
- [x] All 7 grant lifecycle subcommands fully implemented and operational in `aiosh-cli`.
- [x] JSON envelope contracts and exit codes (0, 1, 2) strictly honored.
- [x] Compilation clean with 0 warnings (`cargo check -p aiosh-cli`).
