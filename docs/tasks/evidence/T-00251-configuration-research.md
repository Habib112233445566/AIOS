# T-00251 — Release Packaging & Backup / configuration: Research

## Facts
1. **No dedicated config module exists** for Release & Backup. Parameters are currently hardcoded as function arguments or defaults (e.g., `MAX_FILE_SIZE = 2GB`, ISO mock path format).
2. **Existing config patterns**: The project uses environment variables (`AIOSH_TASKS_DIR`, `AIOSH_CONSTITUTION`, `AIOSH_CI_RESULTS`) and JSON files (`TASK_STATE.json`) for configuration.
3. **Release defaults**: `components=["core"]`, `include_audit=True`, `include_memory=False` are hardcoded in the MCP/CLI tool wrappers.

## Assumptions
- Configuration should follow the existing env-var + JSON file pattern rather than introducing a new config framework.
- A `release_config.json` file can hold default values for release/backup operations.
- The configuration should be loadable from both Python (MCP) and Rust (CLI) substrates.

## Decisions Needed
1. **Config file location**: Use `$AIOSH_RELEASE_CONFIG` env var pointing to a JSON file, defaulting to `config/release.json` relative to the project root.
2. **Config schema**: `{"max_file_size_bytes": int, "default_components": [str], "output_dir": str, "backup_defaults": {"include_audit": bool, "include_memory": bool}}`.
