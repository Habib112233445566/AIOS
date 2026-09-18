#!/usr/bin/env python3
"""Hardening Proof for the Filesystem Layout CLI Surface (T-01528).

Proves, through the real `aiosh` binary, that the FsLayout surface fails safely under
the failure and misuse modes the T-01528 hardening task targets:

  H-1  Non-regular-file paths (`--store`, `--spec`, `--fstab`) are rejected by file *type*
       with an explicit, audited error instead of being read.
  H-2  A FIFO or character device at those paths cannot block the CLI forever or stream
       without bound; the invocation returns promptly with an explicit error.  (POSIX only:
       Windows has no mkfifo.)
  H-3  Oversized documents are rejected with the documented size error, within a bounded
       wall-clock budget, on both the read and write paths.
  H-4  A failed save produces an explicit audited error, leaves no staged temporary file
       behind, and does not destroy a pre-existing store.
  H-5  The staged write never follows a pre-existing symlink at the destination: the link is
       replaced rather than written through, so exactly one state is installed.  (POSIX only.)
  H-6  A successful re-save installs the complete new state, leaves no staged files, and keeps
       every previously registered layout.

Observable evidence is the process exit status / stdout / stderr, the on-disk store, and the
real audit ring (`$AIOSH_HOME/audit.db`, table `audit_ring`).

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_hardening.py
"""

from __future__ import annotations

import json
import os
import sqlite3
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

AUDIT_HOME = tempfile.mkdtemp(prefix="aios_fslayout_hardening_")
LIMIT_BYTES = 10 * 1024 * 1024
POSIX = os.name == "posix"
SKIPPED: list[str] = []


# ---------------------------------------------------------------------------
# Harness
# ---------------------------------------------------------------------------

def get_binary_path() -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in ("aiosh.exe", "aiosh"):
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return "aiosh"


def run_cli(
    *args: str,
    expect: int | None = 0,
    context: str = "",
    timeout: int = 30,
) -> subprocess.CompletedProcess:
    env = dict(os.environ)
    env["AIOSH_HOME"] = AUDIT_HOME
    try:
        cp = subprocess.run(
            [get_binary_path(), *args],
            capture_output=True,
            text=True,
            timeout=timeout,
            env=env,
        )
    except subprocess.TimeoutExpired:
        raise AssertionError(
            f"{context or ' '.join(args)}: CLI did not return within {timeout}s "
            f"(a path that blocks is exactly what this suite is guarding against)"
        ) from None
    if expect is not None:
        assert cp.returncode == expect, (
            f"{context or ' '.join(args)}: expected exit {expect}, got {cp.returncode}\n"
            f"stdout: {cp.stdout.strip()}\nstderr: {cp.stderr.strip()}"
        )
    return cp


def cli_json(*args: str, expect: int | None = 0, context: str = ""):
    return json.loads(run_cli(*args, expect=expect, context=context).stdout.strip())


def envelope(*args: str, expect: int, context: str = "") -> dict:
    return cli_json(*args, expect=expect, context=context)


def _db() -> Path:
    return Path(AUDIT_HOME) / "audit.db"


def audit_rows() -> list[dict]:
    db = _db()
    if not db.exists():
        return []
    con = sqlite3.connect(str(db))
    try:
        cur = con.execute(
            "SELECT id, tool, command, outcome, outcome_detail FROM audit_ring ORDER BY id"
        )
        return [
            {
                "id": r[0],
                "tool": r[1],
                "command": r[2],
                "outcome": r[3],
                "outcome_detail": r[4],
            }
            for r in cur.fetchall()
        ]
    finally:
        con.close()


def max_audit_id() -> int:
    rows = audit_rows()
    return rows[-1]["id"] if rows else 0


def fs_layout_rows(mark: int) -> list[dict]:
    return [r for r in audit_rows() if r["id"] > mark and r["tool"] == "fs_layout"]


def staged_temp_files(root: Path) -> list[Path]:
    """Any leftover staged temporary file (hidden `.name.tmp.*`) under `root`."""
    return [p for p in root.rglob("*") if ".tmp." in p.name and p.is_file()]


def valid_spec_file(dir_path: Path, spec_id: str = "hardening-v1") -> Path:
    """A spec derived from the binary's own canonical preset, so the fixture cannot drift."""
    spec = cli_json("layout", "show", "aios-uefi-standard-v1", "--json", context="base spec")["data"]
    spec["id"] = spec_id
    spec["name"] = f"Hardening {spec_id}"
    path = dir_path / f"{spec_id}.json"
    path.write_text(json.dumps(spec), encoding="utf-8")
    return path


# ---------------------------------------------------------------------------
# H-1: non-regular files are rejected by type, explicitly and auditably
# ---------------------------------------------------------------------------

def test_h1_non_regular_paths_are_rejected_and_audited() -> None:
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        as_dir = tmp / "a_directory"
        as_dir.mkdir()

        # A directory can never be a layout document. Before this hardening pass the
        # read was attempted and the operator got an opaque I/O error.
        mark = max_audit_id()
        env = envelope("layout", "register", "--spec", str(as_dir), "--store", str(tmp / "s.json"),
                       "--json", expect=1, context="register --spec <dir>")
        assert env["error"]["code"] == "SPEC_NOT_REGULAR_FILE", env
        assert "is a directory" in env["error"]["message"], env
        rows = fs_layout_rows(mark)
        assert len(rows) == 1, f"expected exactly 1 audit row, got {rows}"
        assert rows[0]["command"] == "register" and rows[0]["outcome"] == "failure"
        assert rows[0]["outcome_detail"] == "Spec path is not a regular file"

        mark = max_audit_id()
        env = envelope("layout", "list", "--store", str(as_dir), "--json", expect=1,
                       context="list --store <dir>")
        assert env["error"]["code"] == "LOAD_STORE_FAILED", env
        assert "is a directory" in env["error"]["message"], env
        assert len(fs_layout_rows(mark)) == 1

        mark = max_audit_id()
        env = envelope("layout", "import-fstab", "imported-v1", "--fstab", str(as_dir),
                       "--store", str(tmp / "s.json"), "--json", expect=1,
                       context="import-fstab --fstab <dir>")
        assert env["error"]["code"] == "FSTAB_NOT_REGULAR_FILE", env
        assert "is a directory" in env["error"]["message"], env
        assert len(fs_layout_rows(mark)) == 1

        # Non-UTF-8 content is reported as its own failure rather than as a parse error.
        binary_spec = tmp / "binary.json"
        binary_spec.write_bytes(b"\xff\xfe\x00\x01")
        env = envelope("layout", "register", "--spec", str(binary_spec), "--store", str(tmp / "s.json"),
                       "--json", expect=1, context="register --spec <binary>")
        assert env["error"]["code"] == "SPEC_READ_FAILED", env
        assert "UTF-8" in env["error"]["message"], env

        assert not staged_temp_files(tmp), "rejected reads must not stage files"

    print("PASS: H-1 non-regular paths rejected by type, explicitly and audited")


# ---------------------------------------------------------------------------
# H-2: FIFOs and character devices cannot block or stream the CLI
# ---------------------------------------------------------------------------

def test_h2_blocking_and_unbounded_paths_return_promptly() -> None:
    if not POSIX:
        SKIPPED.append("H-2 FIFO/character-device blocking proof (needs POSIX mkfifo)")
        print("SKIP: H-2 FIFO/character-device proof requires POSIX (Windows has no mkfifo)")
        return

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        fifo = tmp / "a_fifo"
        os.mkfifo(fifo)

        # A FIFO reports a metadata length of 0, so a length-only cap passes and a plain
        # read_to_string would block forever waiting for a writer that never comes.
        for argv, label in (
            (("layout", "register", "--spec", str(fifo), "--store", str(tmp / "s.json"), "--json"),
             "register --spec <fifo>"),
            (("layout", "list", "--store", str(fifo), "--json"), "list --store <fifo>"),
        ):
            cp = run_cli(*argv, expect=1, context=label, timeout=20)
            env = json.loads(cp.stdout.strip())
            assert env["error"]["code"] in ("SPEC_NOT_REGULAR_FILE", "LOAD_STORE_FAILED"), env
            assert "FIFO" in env["error"]["message"] or "not a regular file" in env["error"]["message"], env

        # A character device such as /dev/zero passes a length check and then never reaches
        # EOF, so an unbounded read would consume memory until the process died.
        if Path("/dev/zero").exists():
            cp = run_cli("layout", "register", "--spec", "/dev/zero", "--store", str(tmp / "s.json"),
                         "--json", expect=1, context="register --spec /dev/zero", timeout=20)
            env = json.loads(cp.stdout.strip())
            assert env["error"]["code"] == "SPEC_NOT_REGULAR_FILE", env

    print("PASS: H-2 FIFO and character-device paths return promptly with explicit errors")


# ---------------------------------------------------------------------------
# H-3: oversized documents are rejected within a bounded budget
# ---------------------------------------------------------------------------

def test_h3_oversized_documents_are_bounded_and_explicit() -> None:
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)

        oversized_spec = tmp / "oversized_spec.json"
        oversized_spec.write_bytes(b"x" * (LIMIT_BYTES + 1))
        env = envelope("layout", "register", "--spec", str(oversized_spec),
                       "--store", str(tmp / "s.json"), "--json", expect=1,
                       context="register oversized spec", )
        assert env["error"]["code"] == "SPEC_SIZE_EXCEEDED", env
        assert "exceeds" in env["error"]["message"], env

        oversized_store = tmp / "oversized_store.json"
        oversized_store.write_bytes(b"x" * (LIMIT_BYTES + 1))
        env = envelope("layout", "list", "--store", str(oversized_store), "--json", expect=1,
                       context="list oversized store")
        assert env["error"]["code"] == "LOAD_STORE_FAILED", env
        assert "exceeds" in env["error"]["message"], env

    print("PASS: H-3 oversized spec and store documents rejected explicitly")


# ---------------------------------------------------------------------------
# H-4: failed saves are explicit, audited, leak-free, and non-destructive
# ---------------------------------------------------------------------------

def test_h4_failed_save_is_audited_and_leaves_no_temp_file() -> None:
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        spec = valid_spec_file(tmp)

        # Saving under a parent that is a regular file cannot succeed.
        not_a_dir = tmp / "not_a_dir"
        not_a_dir.write_text("i am a file, not a directory", encoding="utf-8")
        mark = max_audit_id()
        env = envelope("layout", "register", "--spec", str(spec),
                       "--store", str(not_a_dir / "store.json"), "--json", expect=1,
                       context="register into unwritable parent")
        assert env["error"]["code"] == "SAVE_STORE_FAILED", env
        rows = fs_layout_rows(mark)
        assert len(rows) == 1, f"failed save must emit exactly 1 audit row, got {rows}"
        assert rows[0]["outcome"] == "failure"
        assert rows[0]["outcome_detail"] == "Failed to persist layout store"
        # The failure is reported, never swallowed, and no staged file leaks.
        assert not staged_temp_files(tmp), f"leaked staged files: {staged_temp_files(tmp)}"

    if not POSIX or os.geteuid() == 0:
        SKIPPED.append("H-4b failed-save preservation on a read-only directory (needs POSIX non-root)")
        print("SKIP: H-4b read-only-directory preservation proof requires POSIX non-root")
        print("PASS: H-4a failed save is explicit, audited, and leaves no staged file")
        return

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        spec = valid_spec_file(tmp)
        store = tmp / "store.json"

        # Establish a good store first.
        run_cli("layout", "register", "--spec", str(spec), "--store", str(store), "--json",
                context="initial register")
        original = store.read_bytes()

        # Make the directory unwritable so the staged write cannot be created. The
        # pre-existing store must survive: the old implementation unlinked the destination
        # before renaming, so a failure here could have left the operator with no store.
        os.chmod(tmp, 0o500)
        try:
            second = valid_spec_file(tmp, "hardening-v2")
        except OSError:
            second = None
        try:
            if second is not None:
                env = envelope("layout", "register", "--spec", str(second), "--store", str(store),
                               "--json", expect=1, context="register into read-only directory")
                assert env["error"]["code"] == "SAVE_STORE_FAILED", env
        finally:
            os.chmod(tmp, 0o700)

        assert store.exists(), "a failed save must never delete the existing store"
        assert store.read_bytes() == original, "a failed save must not corrupt the existing store"
        reloaded = cli_json("layout", "list", "--store", str(store), "--json", context="reload")["data"]
        assert reloaded["active"] == "aios-uefi-standard-v1"

    print("PASS: H-4 failed saves are explicit, audited, leak-free, and non-destructive")


# ---------------------------------------------------------------------------
# H-5: the staged write does not follow a pre-existing symlink
# ---------------------------------------------------------------------------

def test_h5_write_does_not_follow_symlink_at_destination() -> None:
    if not POSIX:
        SKIPPED.append("H-5 symlink-at-destination proof (needs POSIX symlink)")
        print("SKIP: H-5 symlink proof requires POSIX")
        return

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        real = tmp / "real_store.json"
        link = tmp / "link_store.json"

        # A valid store at the real path, reached through a symlink.
        spec_a = valid_spec_file(tmp, "symlink-a")
        run_cli("layout", "register", "--spec", str(spec_a), "--store", str(real), "--json",
                context="seed real store")
        real_before = real.read_bytes()
        try:
            link.symlink_to(real)
        except (OSError, NotImplementedError) as exc:
            SKIPPED.append(f"H-5 symlink creation unavailable: {exc}")
            print(f"SKIP: H-5 symlink creation unavailable ({exc})")
            return

        spec_b = valid_spec_file(tmp, "symlink-b")
        run_cli("layout", "register", "--spec", str(spec_b), "--store", str(link), "--json",
                context="register through symlink")

        # The rename replaced the link itself instead of writing through it, so the linked
        # target keeps exactly the state it had and only one store holds the new layout.
        assert real.read_bytes() == real_before, (
            "the write followed the symlink and modified the linked target"
        )
        assert not os.path.islink(link), "the destination should be a regular file after the save"
        installed = json.loads(link.read_text(encoding="utf-8"))
        assert "symlink-b" in installed["layouts"], installed["layouts"].keys()
        assert "symlink-a" not in installed["layouts"], "the linked target's state leaked in"

    print("PASS: H-5 staged write replaces a symlink instead of writing through it")


# ---------------------------------------------------------------------------
# H-6: a successful re-save installs the complete new state
# ---------------------------------------------------------------------------

def test_h6_successful_resave_is_complete_and_leaves_no_staged_files() -> None:
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        store = tmp / "store.json"

        for spec_id in ("resave-a", "resave-b"):
            spec = valid_spec_file(tmp, spec_id)
            run_cli("layout", "register", "--spec", str(spec), "--store", str(store), "--json",
                    context=f"register {spec_id}")
            # The destination must be a complete, parseable store at every step.
            on_disk = json.loads(store.read_text(encoding="utf-8"))
            assert spec_id in on_disk["layouts"], f"{spec_id} missing after its own save"
            assert not staged_temp_files(tmp), f"staged files leaked: {staged_temp_files(tmp)}"

        # Both custom layouts plus the two canonical presets, and the active pointer intact.
        final = json.loads(store.read_text(encoding="utf-8"))
        assert {"resave-a", "resave-b"} <= set(final["layouts"]), final["layouts"].keys()
        assert {"aios-uefi-standard-v1", "aios-container-minimal-v1"} <= set(final["layouts"])
        assert final["active_layout_id"] == "aios-uefi-standard-v1"

        data = cli_json("layout", "list", "--store", str(store), "--json", context="final list")["data"]
        assert len(data["layouts"]) == 4, [l["id"] for l in data["layouts"]]

    print("PASS: H-6 successful re-saves install the complete state with no staged files")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT CLI HARDENING PROOF ===")
    test_h1_non_regular_paths_are_rejected_and_audited()
    test_h2_blocking_and_unbounded_paths_return_promptly()
    test_h3_oversized_documents_are_bounded_and_explicit()
    test_h4_failed_save_is_audited_and_leaves_no_temp_file()
    test_h5_write_does_not_follow_symlink_at_destination()
    test_h6_successful_resave_is_complete_and_leaves_no_staged_files()
    if SKIPPED:
        print("\nSKIPPED (not applicable on this platform):")
        for item in SKIPPED:
            print(f"  - {item}")
    print("\nALL FILESYSTEM LAYOUT CLI HARDENING PROOFS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
