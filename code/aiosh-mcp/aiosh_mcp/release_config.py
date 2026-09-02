"""Release & Backup configuration loader.

Reads from $AIOSH_RELEASE_CONFIG or config/release.json, falling back
to hardcoded defaults if the file is absent.
"""
import json
import os
from pathlib import Path
from typing import Any

_DEFAULTS = {
    "max_file_size_bytes": 2 * 1024 * 1024 * 1024,  # 2 GB
    "default_components": ["core"],
    "output_dir": "output/release",
    "backup_defaults": {
        "include_audit": True,
        "include_memory": False,
    },
}

_MIN_FILE_SIZE = 1 * 1024 * 1024       # 1 MB
_MAX_FILE_SIZE = 10 * 1024 * 1024 * 1024  # 10 GB


def load_config(path: str | None = None) -> dict[str, Any]:
    """Load release configuration from JSON file or return defaults.

    Args:
        path: Explicit path. If None, checks $AIOSH_RELEASE_CONFIG,
              then falls back to config/release.json relative to the
              project root.

    Returns:
        dict with validated configuration values.

    Raises:
        ValueError: If the file exists but contains malformed JSON.
    """
    if path is None:
        path = os.environ.get("AIOSH_RELEASE_CONFIG")
    if path is None:
        # Default: config/release.json relative to repo root
        repo_root = Path(__file__).resolve().parents[3]
        path = str(repo_root / "config" / "release.json")

    try:
        with open(path, "r", encoding="utf-8") as f:
            raw = json.load(f)
    except FileNotFoundError:
        return dict(_DEFAULTS)
    except json.JSONDecodeError as e:
        raise ValueError(f"Malformed release config at {path}: {e}")

    # Merge with defaults (config file overrides defaults)
    merged = dict(_DEFAULTS)
    if isinstance(raw, dict):
        for key in _DEFAULTS:
            if key in raw:
                merged[key] = raw[key]

    # Clamp max_file_size_bytes
    size = merged.get("max_file_size_bytes", _DEFAULTS["max_file_size_bytes"])
    if isinstance(size, (int, float)):
        merged["max_file_size_bytes"] = max(_MIN_FILE_SIZE, min(int(size), _MAX_FILE_SIZE))

    return merged
