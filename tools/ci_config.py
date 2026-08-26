"""Configuration resolution for CI Smoke Orchestration (T-00153).

Contract: docs/tasks/evidence/T-00152-spec.md.
"""

import os
from typing import Any, Callable

class CiConfig:
    def __init__(self, results_path: str, max_file_bytes: int,
                 timeout_default_s: int, load_retries: int,
                 retry_sleep_ms: int) -> None:
        self.results_path = results_path
        self.max_file_bytes = max_file_bytes
        self.timeout_default_s = timeout_default_s
        self.load_retries = load_retries
        self.retry_sleep_ms = retry_sleep_ms

    @classmethod
    def from_env(cls) -> "CiConfig":
        return cls.from_source(os.environ.get)

    @classmethod
    def from_source(cls, get_fn: Callable[[str], str | None]) -> "CiConfig":
        def parse_int(name: str, raw: str, floor: int) -> int:
            try:
                v = int(raw)
            except ValueError:
                raise ValueError(f"invalid {name}={raw!r}: must be integer")
            if v < floor:
                raise ValueError(f"invalid {name}={raw!r}: must be >= {floor}")
            return v

        results_path = get_fn("AIOSH_CI_RESULTS") or "/tmp/aiosh-ci-results.json"
        if not results_path:
            raise ValueError("invalid AIOSH_CI_RESULTS='': must not be empty")

        raw_max_file = get_fn("AIOSH_CI_MAX_FILE_BYTES")
        max_file_bytes = parse_int("AIOSH_CI_MAX_FILE_BYTES", raw_max_file, 1024) if raw_max_file else 1048576

        raw_timeout = get_fn("AIOSH_CI_TIMEOUT_DEFAULT_S")
        timeout_default_s = parse_int("AIOSH_CI_TIMEOUT_DEFAULT_S", raw_timeout, 10) if raw_timeout else 900

        raw_retries = get_fn("AIOSH_CI_LOAD_RETRIES")
        load_retries = parse_int("AIOSH_CI_LOAD_RETRIES", raw_retries, 0) if raw_retries else 3

        raw_sleep = get_fn("AIOSH_CI_RETRY_SLEEP_MS")
        retry_sleep_ms = parse_int("AIOSH_CI_RETRY_SLEEP_MS", raw_sleep, 10) if raw_sleep else 500

        return cls(results_path, max_file_bytes, timeout_default_s, load_retries, retry_sleep_ms)

    def to_json_with_sources(self) -> dict[str, Any]:
        return self.to_json_with_sources_from(lambda n: n in os.environ)

    def to_json_with_sources_from(self, is_set: Callable[[str], bool]) -> dict[str, Any]:
        def src(name: str) -> str:
            return "env" if is_set(name) else "default"
        return {
            "results_path": {"value": self.results_path, "source": src("AIOSH_CI_RESULTS")},
            "max_file_bytes": {"value": self.max_file_bytes, "source": src("AIOSH_CI_MAX_FILE_BYTES")},
            "timeout_default_s": {"value": self.timeout_default_s, "source": src("AIOSH_CI_TIMEOUT_DEFAULT_S")},
            "load_retries": {"value": self.load_retries, "source": src("AIOSH_CI_LOAD_RETRIES")},
            "retry_sleep_ms": {"value": self.retry_sleep_ms, "source": src("AIOSH_CI_RETRY_SLEEP_MS")},
        }
