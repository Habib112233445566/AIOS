# T-00249 — MCP/API surface: Documentation
Updated `docs/README.md` (T-239) with MCP tool call examples and CLI examples. Both surfaces documented.

**MCP Tool Call Example:**
```json
{"name": "aios.release.generate", "arguments": {"target_os": "debian-13", "version": "1.0.0", "components": ["aiosh-core"], "grant_id": "grant-12345"}}
```

**Limitations:** Windows ISO mock, 2GB file cap, symlink exclusion, Rust CLI won't compile on Windows (sandbox.rs libc). All documented in README.
