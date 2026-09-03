# T-01132 — Base Image Build / MCP/API Surface: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / MCP/API Surface

## 1. Tool Schemas & Behavior
### `aios.image.list`
- **Description**: Returns all registered base image manifests with optional format and distribution filters.
- **Parameters**: `format` (optional string), `distro_id` (optional string).
- **Output**: JSON array of manifests.

### `aios.image.get`
- **Description**: Returns detailed manifest for specified image identifier.
- **Parameters**: `id` (required string).
- **Output**: JSON serialized `BaseImageManifest`.

### `aios.image.plan`
- **Description**: Generates reproducible 4-stage build execution plan for specified image.
- **Parameters**: `id` (required string).
- **Output**: JSON serialized `BuildPlan`.
