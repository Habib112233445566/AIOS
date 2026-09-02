# T-01019 — Distro Selection & Justification / Core Service: Documentation

## 1. Documentation Updates
- Updated `docs/README.md` §8.10 with full operator and agent usage guidelines for the Distro Selection & Justification core service.
- Documented CLI commands (`aiosh distro list`, `aiosh distro show`, `aiosh distro evaluate`, `aiosh distro recommend`).
- Documented MCP tools (`aios.distro.list`, `aios.distro.show`, `aios.distro.evaluate`, `aios.distro.recommend`).
- Linked evidence chain from `T-01001` through `T-01019`.

## 2. Copy-Pasteable Usage Examples
```bash
# 1. List registered distro profiles
aiosh distro list

# 2. Inspect Debian 12 minimal specification
aiosh distro show debian-12-minimal-x86_64 --json

# 3. Evaluate criteria scores across all profiles
aiosh distro evaluate

# 4. View recommended base OS profile
aiosh distro recommend
```

## 3. Honest Limitations
- Built-in profiles provide baseline evaluation and selection data; image building and packaging are handled by downstream Phase 1 image generation services.
- Store file loading enforces a bounded 10 MiB limit (`MAX_STORE_BYTES`).
