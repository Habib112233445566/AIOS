#!/usr/bin/env bash
# AIOS CI runner — THIN SHIM (T-00116).
#
# The orchestrator now lives in tools/ci_run.py, driven by the suite
# registry in tools/ci_suites.py (single source; order IS CONTRACT —
# suites share code/aiosh-cli/dist rebuilds, never parallelize).
# This shim preserves the historical entrypoint and its env knobs
# (PYTHON). Machine-readable run summary: $AIOSH_CI_RESULTS
# (default /tmp/aiosh-ci-results.json).
#
# Usage: bash ci/run_all_smokes.sh
# Exit:  0 = all suites PASS; 1 = first failing suite (printed).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Note: disabling set -e temporarily because ci_run.py failure is caught in RC
set +e
python3 "$ROOT/tools/ci_run.py"
RC=$?
set -e

# Validate the output artifact using the core service (T-00126 integration)
if [ -f "${AIOSH_CI_RESULTS:-/tmp/aiosh-ci-results.json}" ]; then
    set +e
    python3 "$ROOT/tools/ci_service.py" check
    CHECK_RC=$?
    set -e
    
    if [ "$RC" -eq 0 ]; then
        exit "$CHECK_RC"
    fi
fi

exit "$RC"
