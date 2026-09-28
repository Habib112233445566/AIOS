#!/usr/bin/env python3
import sqlite3
import os
import subprocess
from pathlib import Path

bin_path = Path("code/aiosh-rust/target/debug/aiosh.exe")
res = subprocess.run([str(bin_path), "sandbox", "exec", "--profile", "permissive", "--", "python", "-c", "print('audit_verify_smoke')"], capture_output=True, text=True)
print("Execution code:", res.returncode)
print("Stdout:", res.stdout.strip())

db_path = os.path.expanduser("~/.aiosh/aiosh.db")
if not os.path.exists(db_path):
    ai_home = os.environ.get("AI_HOME", os.path.expanduser("~/.aiosh"))
    db_path = os.path.join(ai_home, "aiosh.db")

print("Checking DB:", db_path, "exists:", os.path.exists(db_path))
if os.path.exists(db_path):
    conn = sqlite3.connect(db_path)
    cur = conn.cursor()
    cur.execute("SELECT id, tool, command, outcome, ts FROM audit_ring WHERE tool='sandbox' ORDER BY id DESC LIMIT 5")
    rows = cur.fetchall()
    print("Recent sandbox audit rows:")
    for r in rows:
        print(" ", r)
    conn.close()
    assert len(rows) > 0, "Expected at least one sandbox audit record"
print("INTEGRATION SMOKE SUCCEEDED!")
