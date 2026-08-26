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
        
    def test_happy_path(self):
        mock_suites = [{
            "name": "rust_smoke",
            "command": [sys.executable, "-c", "print('ok')"],
            "timeout_s": 5
        }]

        with mock.patch.object(ci_run, 'SUITES', mock_suites), \
             mock.patch.object(ci_suites, 'SUITES', mock_suites), \
             mock.patch.object(ci_suites, 'RESULTS_PATH', str(self.results_path)), \
             mock.patch.dict(os.environ, self.env):
            rc = ci_run.main()
            self.assertEqual(rc, 0)
            
            with open(self.results_path) as f:
                data = json.load(f)
                self.assertTrue(data["all_pass"])
                self.assertEqual(data["passed"], 1)

    def test_fail_fast(self):
        mock_suites = [
            {
                "name": "rust_smoke",
                "command": [sys.executable, "-c", "import sys; print('fail'); sys.exit(1)"],
                "timeout_s": 5
            },
            {
                "name": "rust_audit_tail",
                "command": [sys.executable, "-c", "print('should not run')"],
                "timeout_s": 5
            }
        ]

        with mock.patch.object(ci_run, 'SUITES', mock_suites), \
             mock.patch.object(ci_suites, 'SUITES', mock_suites), \
             mock.patch.object(ci_suites, 'RESULTS_PATH', str(self.results_path)), \
             mock.patch.dict(os.environ, self.env):
            rc = ci_run.main()
            self.assertEqual(rc, 1)
            
            with open(self.results_path) as f:
                data = json.load(f)
                self.assertFalse(data["all_pass"])
                self.assertEqual(data["passed"], 0)
                self.assertEqual(data["failed"], 1)
                self.assertEqual(len(data["results"]), 1)
                self.assertEqual(data["results"][0]["suite"], "rust_smoke")

if __name__ == "__main__":
    unittest.main()
