"""T-255: Unit tests for Release & Backup configuration."""
import json
import os
import pytest
from aiosh_mcp.release_config import load_config


def test_defaults_when_no_file():
    """Happy path: returns defaults when config file doesn't exist."""
    cfg = load_config("/nonexistent/path/release.json")
    assert cfg["max_file_size_bytes"] == 2 * 1024 * 1024 * 1024
    assert cfg["default_components"] == ["core"]
    assert cfg["output_dir"] == "output/release"
    assert cfg["backup_defaults"]["include_audit"] is True
    assert cfg["backup_defaults"]["include_memory"] is False


def test_load_from_file(tmp_path):
    """Happy path: loads config from valid JSON file."""
    cfg_file = tmp_path / "release.json"
    cfg_file.write_text(json.dumps({
        "max_file_size_bytes": 5000000000,
        "default_components": ["core", "agent"],
        "output_dir": "/custom/output",
    }))
    cfg = load_config(str(cfg_file))
    assert cfg["max_file_size_bytes"] == 5000000000
    assert cfg["default_components"] == ["core", "agent"]
    assert cfg["output_dir"] == "/custom/output"
    # Defaults preserved for missing keys
    assert cfg["backup_defaults"]["include_audit"] is True


def test_malformed_json_raises(tmp_path):
    """Negative: malformed JSON raises ValueError."""
    bad_file = tmp_path / "bad.json"
    bad_file.write_text("not valid json {{{")
    with pytest.raises(ValueError, match="Malformed"):
        load_config(str(bad_file))


def test_clamp_max_file_size_too_small(tmp_path):
    """Boundary: max_file_size_bytes below 1MB is clamped to 1MB."""
    cfg_file = tmp_path / "release.json"
    cfg_file.write_text(json.dumps({"max_file_size_bytes": 100}))
    cfg = load_config(str(cfg_file))
    assert cfg["max_file_size_bytes"] == 1 * 1024 * 1024


def test_clamp_max_file_size_too_large(tmp_path):
    """Boundary: max_file_size_bytes above 10GB is clamped to 10GB."""
    cfg_file = tmp_path / "release.json"
    cfg_file.write_text(json.dumps({"max_file_size_bytes": 999999999999}))
    cfg = load_config(str(cfg_file))
    assert cfg["max_file_size_bytes"] == 10 * 1024 * 1024 * 1024


def test_env_var_override(tmp_path, monkeypatch):
    """Config path can be set via env var."""
    cfg_file = tmp_path / "custom_release.json"
    cfg_file.write_text(json.dumps({"output_dir": "/from/env"}))
    monkeypatch.setenv("AIOSH_RELEASE_CONFIG", str(cfg_file))
    cfg = load_config()
    assert cfg["output_dir"] == "/from/env"
