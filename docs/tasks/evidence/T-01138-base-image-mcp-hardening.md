# T-01138 — Base Image Build / MCP/API Surface: Hardening

**Date:** 2026-09-03
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / MCP/API Surface

## 1. Hardening Deliverables
- Enforced strict length limits ($1 \le \text{len} \le 128$) and ASCII graphic character validation for all `id` parameters in `aios.image.get` and `aios.image.plan`.
- Enforced maximum path length (4096 bytes) for custom `store_path` inputs across all image tools.
- Added negative unit test assertions in `test_mcp_image_tools` verifying rejection of non-printable IDs, oversized IDs, and oversized paths.

## 2. Test Execution Output
```
running 1 test
test tests::test_mcp_image_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.07s
```
