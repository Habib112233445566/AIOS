# T-01039 — Distro Selection & Justification / MCP/API Surface: Documentation

## 1. Documentation Updates
Updated `docs/README.md` §8.10 with full MCP tool specifications, JSON-RPC invocation examples, and testing instructions.

### MCP Tool Endpoints
- `aios.distro.list`: Enumerate registered distro profiles.
- `aios.distro.show`: Inspect single profile by `id`.
- `aios.distro.evaluate`: Calculate multi-factor evaluation scores for target profile or all profiles.
- `aios.distro.recommend`: Query reference distribution profile.

### Copy-Pasteable Tool Call Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.distro.evaluate",
    "arguments": {
      "id": "debian-12-minimal-x86_64"
    }
  }
}
```

### Python MCP Smoke Suite
```bash
python code/aiosh-mcp/tests/test_distro_mcp_smoke.py
# ALL DISTRO MCP SMOKE TESTS PASSED!
```

## 2. Honest Limitations
- Custom stores loaded via `store_path` must strictly conform to `DistroStore` schema and adhere to the 10 MiB hard cap (`MAX_STORE_BYTES`).
- ISO creation and physical image building are scheduled for downstream rootfs and bootable target sub-epics.
