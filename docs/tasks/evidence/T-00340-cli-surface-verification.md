# T-00340 — Dependency & Toolchain Pinning / CLI surface: Verification & Evidence

## Test Results
```
test result: ok. 13 passed; 0 failed (aiosh-cli)
test result: ok. 92 passed; 0 failed (aiosh-core)
test result: ok. 0 passed; 0 failed (aiosh-mcp)
test result: ok. 0 passed; 0 failed (aiosh-sandbox)
test result: ok. 0 passed; 0 failed (doc-tests)
```

**Total: 105 tests, 0 failures.**

## CLI Surface Epic Complete
The Dependency & Toolchain Pinning CLI surface sub-epic (T-00331 through T-00340) is complete.

### Shipped:
- `aiosh toolchain check [--config <path>]` — enforces host toolchain against manifest
- `aiosh toolchain show [--config <path>]` — read-only manifest inspection
- `--config` flag overrides env var and default path
- Unknown flags rejected with usage text
- All paths emit audit rows
- Security reviewed, hardened, documented
