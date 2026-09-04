# T-01169 — Base Image Build / Security Policy: Documentation

**Date:** 2026-09-04
**Type:** Documentation
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Documentation Summary
Updated `docs/README.md` with:
- Overview of `aiosh-core::base_image_policy` and `BaseImageSecurityPolicy`.
- Invariant definitions: `P1..P7`.
- Policy enforcement modes: `Enforcing`, `Audit`, and `Permissive`.
- CLI commands: `aiosh image policy [<id>] [--json] [--store <path>]`.
- MCP tools: `aios.image.policy`.

## 2. Invocation Examples
### CLI
```bash
# Check all images in store
aiosh image policy

# Check single image in JSON mode
aiosh image policy debian-12-minimal-raw --json
```

### MCP Tool Call
```json
{
  "name": "aios.image.policy",
  "arguments": {
    "id": "debian-12-minimal-raw"
  }
}
```
