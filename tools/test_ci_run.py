import os
import subprocess
import sys
import tempfile
import json
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
sys.path.insert(0, str(HERE))

import ci_run
import ci_suites

class TestCiRunOrchestrator(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self.tmp.name)
        self.results_path = self.tmp_dir / "results.json"
        
        # Override config via env
        self.env = os.environ.copy()
        self.env["AIOSH_CI_RESULTS"] = str(self.results_path)
        self.env["AIOSH_CI_TIMEOUT_DEFAULT_S"] = "2"
        self.env["AIOSH_CI_MAX_FILE_BYTES"] = "1024"

    def tearDown(self):
        self.tmp.cleanup()
        
    @mock.patch.object(ci_suites, 'SUITES', new=[])
    def test_happy_path(self):
        ci_suites.SUITES.append({
            "name": "mock_pass",
            "command": ["python3", "-c", "print('ok')"],
            "timeout_s": 5
        })
        ci_suites._seen.add("mock_pass")
        ci_suites.SUITE_NAMES = ("mock_pass",)

        with mock.patch.dict(os.environ, self.env):
            rc = ci_run.main()
            self.assertEqual(rc, 0)
            
            with open(self.results_path) as f:
                data = json.load(f)
                self.assertTrue(data["all_pass"])
                self.assertEqual(data["passed"], 1)

    @mock.patch.object(ci_suites, 'SUITES', new=[])
    def test_fail_fast(self):
        ci_suites.SUITES.extend([
            {
                "name": "mock_fail",
                "command": ["python3", "-c", "import sys; print('fail'); sys.exit(1)"],
                "timeout_s": 5
            },
            {
                "name": "mock_skip",
                "command": ["python3", "-c", "print('should not run')"],
                "timeout_s": 5
            }
        ])
        ci_suites._seen.update(["mock_fail", "mock_skip"])
        ci_suites.SUITE_NAMES = ("mock_fail", "mock_skip")

        with mock.patch.dict(os.environ, self.env):
            rc = ci_run.main()
            self.assertEqual(rc, 1)
            
            with open(self.results_path) as f:
                data = json.load(f)
                self.assertFalse(data["all_pass"])
                self.assertEqual(data["passed"], 0)
                self.assertEqual(data["failed"], 1)
                self.assertEqual(len(data["results"]), 1)
                self.assertEqual(data["results"][0]["suite"], "mock_fail")

if __name__ == "__main__":
    unittest.main()
