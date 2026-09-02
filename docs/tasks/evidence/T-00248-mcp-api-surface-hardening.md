# T-00248 — MCP/API surface: Hardening
- Errors from `generate_release`/`create_backup` are caught by `try/except Exception` in the MCP wrappers and returned as `{"ok": false, "error": ...}` — never silent.
- Core functions enforce 2GB file-size caps and symlink guards (from T-0228).
- DB connections are managed by `dispatch_mod.conn_ctx()` context manager — no leaks on error.
- The MCP wrappers return the error envelope directly; the core functions write honest audit rows internally.
- No temp file leaks: `zipfile.ZipFile` uses context managers, and ISO mock writes are atomic.
