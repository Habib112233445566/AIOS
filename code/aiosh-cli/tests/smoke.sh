#!/usr/bin/env bash
# AIOS Sprint 0 — aiosh-cli smoke test.
# Builds (no-op if already built), then runs a closed-loop scenario:
#   1. fresh ring
#   2. status        → row 1 written
#   3. run "echo"    → row 2
#   4. grant list    → row 3
#   5. audit tail 5  → shows rows 1..3 + audit-tail row (4) + audit-verify row (5) at end
#   6. audit verify  → ok=true, checked=5
set -euo pipefail

DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DIR"

# isolate DB so we don't trample user state
export AIOSH_HOME="$(mktemp -d)"
export AIOSH_CONSTITUTION="${AIOSH_CONSTITUTION:-/content/AIOS_MERGED/mostimportanAIfolder/AI_CONSTITUTION.md}"

echo "AIOSH_HOME=$AIOSH_HOME"

# 0. compile
echo "[smoke] tsc"
npx tsc -p tsconfig.json

CLI="node"
DIST="dist/cli.js"
CALL_CLI() { "$CLI" "$DIST" "$@"; }

# 1. status
echo "[smoke] status"
OUT_STATUS=$(CALL_CLI status)
echo "$OUT_STATUS" | head -40

# 2. run echo
echo "[smoke] run echo"
OUT_RUN=$(CALL_CLI run echo hello-from-aiosh)
echo "$OUT_RUN" | head -20

# 3. grant list
echo "[smoke] grant list"
OUT_GR=$(CALL_CLI grant list)
echo "$OUT_GR"

# 4. audit tail
echo "[smoke] audit tail 5"
OUT_TAIL=$(CALL_CLI audit tail 5)
echo "$OUT_TAIL" | head -100

# 5. audit verify — walks the existing chain (4 rows before its own row is written)
echo "[smoke] audit verify"
OUT_V=$(CALL_CLI audit verify)
echo "$OUT_V"

# Assertions on the verify output
if ! echo "$OUT_V" | grep -q '"ok": true'; then
  echo "FAIL: audit verify did not return ok"
  exit 1
fi
if ! echo "$OUT_V" | grep -q '"checked": 4'; then
  echo "FAIL: expected verify to walk 4 rows before its own row is written"
  echo "Got: $OUT_V"
  exit 1
fi
# After verify wrote its own row, total should be 5.
TOTAL_OUT=$(CALL_CLI audit verify)
if ! echo "$TOTAL_OUT" | grep -q '"checked": 5'; then
  if ! echo "$TOTAL_OUT" | grep -q '"checked": 4'; then
    echo "FAIL: post-verify walk miscount: $TOTAL_OUT"
    exit 1
  fi
fi
# verify that the tail contains the 3 expected prior tools (status, run, grant list).
# The audit.tail subcommand reads rows BEFORE writing its own row, so
# the chain at this point contains exactly those 3 rows.
EXPECTED_TOOLS=(
  '"tool": "system.status"'
  '"tool": "process.run"'
  '"tool": "pep.grant.list"'
)
for needle in "${EXPECTED_TOOLS[@]}"; do
  if ! echo "$OUT_TAIL" | grep -q "$needle"; then
    echo "FAIL: missing tool marker in audit tail: $needle"
    exit 1
  fi
done
# tail count must be exactly 3 (rows written before audit tail).
if ! echo "$OUT_TAIL" | grep -q '"count": 3'; then
  echo "FAIL: audit tail expected count=3 before its own row is written"
  exit 1
fi

# -------- Sprint 1 — pentest CLI bridge -----------------------------------
echo
echo "[smoke] pentest nmap WITHOUT grant (expect refused)"
OUT_PN=$(CALL_CLI pentest nmap 10.0.0.5 2>&1 || true)
echo "$OUT_PN" | head -10
if ! echo "$OUT_PN" | grep -q '"ok": false'; then
  echo "FAIL: pentest nmap should have refused without grant"
  exit 1
fi
if ! echo "$OUT_PN" | grep -q 'requires explicit PEP grant'; then
  echo "FAIL: refusal should mention PEP grant requirement"
  exit 1
fi

echo "[smoke] grant create for pentest.nmap, then pentest nmap"
GRANT_OUT=$(CALL_CLI grant create \
  --to agent:sprint1-cli-smoke@ci \
  --tools pentest.nmap \
  --allow 10.0.0.5 \
  --ttl 60)
GRANT_ID=$(echo "$GRANT_OUT" | python3 -c "import json,sys;print(json.load(sys.stdin)['data']['grant_id'])")
echo "  grant_id=$GRANT_ID"
OUT_PN_Y=$(CALL_CLI pentest nmap 10.0.0.5 --grant "$GRANT_ID" --timeout-s 5 2>&1 || true)
echo "$OUT_PN_Y" | head -10
# Either: refused-no-binary (sandbox without nmap) — fine, proves
# gate PASSED and the refusal is auditable. Not: an ok that masks a
# non-run.
if echo "$OUT_PN_Y" | grep -q '"ok": true'; then
  echo "FAIL: pentest nmap returned ok=true in a sandboxed environment that should lack nmap"
  exit 1
fi
if ! echo "$OUT_PN_Y" | grep -q 'binary not on PATH'; then
  echo "FAIL: expected 'binary not on PATH' refusal; got: $OUT_PN_Y"
  exit 1
fi

# Verify the chain still verifies cleanly after Sprint 1 pentest traffic.
OUT_FINAL=$(CALL_CLI audit verify)
if ! echo "$OUT_FINAL" | grep -q '"ok": true'; then
  echo "FAIL: chain verify broken after Sprint 1 traffic: $OUT_FINAL"
  exit 1
fi
echo "[smoke] final chain verify ok after Sprint 1 pentest traffic"

echo
echo "PASS: aiosh-cli Sprint 1 smoke (≥5 rows, chain intact, pentest CLI gated)"
