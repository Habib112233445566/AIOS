import os
import unittest
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
sys.path.insert(0, str(REPO / "tools"))

from ci_config import CiConfig

class TestCiConfig(unittest.TestCase):
    def test_defaults(self):
        cfg = CiConfig.from_source(lambda n: None)
        self.assertEqual(cfg.results_path, "/tmp/aiosh-ci-results.json")
        self.assertEqual(cfg.max_file_bytes, 1048576)
        self.assertEqual(cfg.timeout_default_s, 900)
        self.assertEqual(cfg.load_retries, 3)
        self.assertEqual(cfg.retry_sleep_ms, 500)
        
        j = cfg.to_json_with_sources_from(lambda n: False)
        self.assertEqual(j["max_file_bytes"]["source"], "default")

    def test_overrides(self):
        d = {
            "AIOSH_CI_RESULTS": "/foo/bar.json",
            "AIOSH_CI_MAX_FILE_BYTES": "2048",
            "AIOSH_CI_TIMEOUT_DEFAULT_S": "100",
            "AIOSH_CI_LOAD_RETRIES": "5",
            "AIOSH_CI_RETRY_SLEEP_MS": "1000",
        }
        cfg = CiConfig.from_source(d.get)
        self.assertEqual(cfg.results_path, "/foo/bar.json")
        self.assertEqual(cfg.max_file_bytes, 2048)
        self.assertEqual(cfg.timeout_default_s, 100)
        self.assertEqual(cfg.load_retries, 5)
        self.assertEqual(cfg.retry_sleep_ms, 1000)

        j = cfg.to_json_with_sources_from(lambda n: n in d)
        self.assertEqual(j["load_retries"]["source"], "env")

    def test_loud_failures(self):
        with self.assertRaisesRegex(ValueError, "invalid AIOSH_CI_MAX_FILE_BYTES='foo': must be integer"):
            CiConfig.from_source(lambda n: "foo" if n == "AIOSH_CI_MAX_FILE_BYTES" else None)
        
        with self.assertRaisesRegex(ValueError, "invalid AIOSH_CI_TIMEOUT_DEFAULT_S='5': must be >= 10"):
            CiConfig.from_source(lambda n: "5" if n == "AIOSH_CI_TIMEOUT_DEFAULT_S" else None)

if __name__ == "__main__":
    unittest.main()
