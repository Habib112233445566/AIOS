# T-00244 — MCP/API surface: Implementation
Implemented `aios.release.generate` and `aios.backup.create` MCP tool wrappers in `release.py`.
Both use `dispatch_mod.dispatch()` for PEP gating, then delegate to core `generate_release`/`create_backup`.
Classifier kwargs passed through. Import clean, tests green (3/3).
