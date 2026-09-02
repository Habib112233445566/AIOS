# T-00247 — MCP/API surface: Security Review

## Abuse Scenarios Evaluated
1. **Grant bypass**: Both tools gate on `dispatch_mod.dispatch()` which enforces classifier + PEP. Without a valid grant, the tool returns `{"ok": false, "gate": "pep"}` with an audited refusal row. No bypass possible.
2. **Argument injection**: `target_path` is passed as a string directly to Python's `os.walk`/`zipfile` — no shell injection vector exists. Path traversal is bounded by the physical backup walker which skips symlinks.
3. **Size amplification (zip bomb)**: The 2GB per-file cap in `physical_create_zip` prevents decompression bombs from being archived.
4. **Double audit row**: The dispatch gate writes one refusal row OR the core function writes one success/error row — never both for the same invocation. ADR-0035 satisfied.

No policy bypass found.
