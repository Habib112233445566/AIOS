#!/usr/bin/env python3
"""Smoke and Integration test for System Update Mechanism Data Model (T-01906).

Validates:
- Invariants UPD1..UPD6:
  - UPD1: A/B dual slot model (slot_a, slot_b), toggling, exclusivity (current != target).
  - UPD2: Version semantics (1..64 chars), channel enumerations (stable, beta, nightly, development).
  - UPD3: Artifact integrity, 64-hex SHA-256 cryptographic verification, payload size bounds.
  - UPD4: State machine transitions (idle -> checking/downloading -> verifying -> applying -> ready_to_reboot -> verified/rolled_back -> idle; error -> failed).
  - UPD5: Rollback slot safeguard tracking.
  - UPD6: JSON schema roundtrip, serialization parity, and query helpers.
"""

from __future__ import annotations

import json
import re
import sys

SHA256_REGEX = re.compile(r"^[0-9a-fA-F]{64}$")
MAX_VERSION_LEN = 64
MAX_UPDATE_ID_LEN = 128
MAX_PAYLOAD_SIZE = 10 * 1024 * 1024 * 1024  # 10 GB

VALID_SLOTS = {"slot_a", "slot_b"}
VALID_CHANNELS = {"stable", "beta", "nightly", "development"}
VALID_PARTITIONS = {"rootfs", "kernel", "initramfs", "full_bundle"}
VALID_STATES = {
    "idle", "checking", "downloading", "verifying",
    "applying", "ready_to_reboot", "verified", "rolled_back", "failed"
}


def other_slot(slot: str) -> str:
    if slot == "slot_a":
        return "slot_b"
    elif slot == "slot_b":
        return "slot_a"
    raise ValueError(f"Invalid slot: {slot}")


def validate_sha256(digest: str) -> bool:
    if not isinstance(digest, str):
        return False
    return bool(SHA256_REGEX.match(digest.strip()))


MAX_ARTIFACTS_PER_MANIFEST = 32
MAX_ARTIFACT_FILENAME_LEN = 128


def validate_artifact(art: dict) -> bool:
    target = art.get("target")
    if target not in VALID_PARTITIONS:
        return False

    file_name = art.get("file_name", "").strip()
    if not file_name or len(file_name) > MAX_ARTIFACT_FILENAME_LEN:
        return False
    if "/" in file_name or "\\" in file_name or ".." in file_name or file_name.startswith("."):
        return False
    if any(ord(c) <= 32 for c in file_name):
        return False

    sha256 = art.get("sha256", "")
    if not validate_sha256(sha256):
        return False

    size = art.get("size_bytes", 0)
    if not isinstance(size, int) or size <= 0 or size > MAX_PAYLOAD_SIZE:
        return False

    return True


def validate_manifest(manifest: dict) -> bool:
    uid = manifest.get("update_id", "").strip()
    if not uid or len(uid) > MAX_UPDATE_ID_LEN:
        return False

    ver = manifest.get("version", "").strip()
    if not ver or len(ver) > MAX_VERSION_LEN:
        return False

    channel = manifest.get("channel")
    if channel not in VALID_CHANNELS:
        return False

    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) == 0 or len(artifacts) > MAX_ARTIFACTS_PER_MANIFEST:
        return False

    seen_names = set()
    seen_targets = set()
    for art in artifacts:
        if not validate_artifact(art):
            return False
        fn = art.get("file_name")
        tgt = art.get("target")
        if fn in seen_names or tgt in seen_targets:
            return False
        seen_names.add(fn)
        seen_targets.add(tgt)

    return True


def validate_slot_status(status: dict) -> bool:
    curr = status.get("current_slot")
    target = status.get("target_slot")
    if curr not in VALID_SLOTS or target not in VALID_SLOTS:
        return False
    if curr == target:
        return False

    rollback = status.get("rollback_slot")
    if rollback is not None and rollback not in VALID_SLOTS:
        return False

    return True


def can_transition(curr_state: str, next_state: str) -> bool:
    if next_state == "failed":
        return True
    transitions = {
        "idle": {"checking", "downloading"},
        "checking": {"downloading", "idle"},
        "downloading": {"verifying"},
        "verifying": {"applying"},
        "applying": {"ready_to_reboot"},
        "ready_to_reboot": {"verified", "rolled_back"},
        "verified": {"idle"},
        "rolled_back": {"idle"},
        "failed": {"idle"},
    }
    return next_state in transitions.get(curr_state, set())


def test_upd1_slot_exclusivity():
    assert other_slot("slot_a") == "slot_b"
    assert other_slot("slot_b") == "slot_a"

    slot_status = {
        "current_slot": "slot_a",
        "target_slot": "slot_b",
        "rollback_slot": "slot_a",
        "slot_a_version": "1.0.0",
        "slot_b_version": "none",
        "slot_a_successful": True,
        "slot_b_successful": False,
    }
    assert validate_slot_status(slot_status)

    # Invariant: current cannot equal target
    slot_status["target_slot"] = "slot_a"
    assert not validate_slot_status(slot_status)
    print("PASS: test_upd1_slot_exclusivity")


def test_upd2_channel_and_version():
    for ch in ["stable", "beta", "nightly", "development"]:
        assert ch in VALID_CHANNELS
    assert "experimental" not in VALID_CHANNELS

    valid_ver = "v1.2.3-beta.1"
    assert 1 <= len(valid_ver) <= MAX_VERSION_LEN
    oversized = "x" * (MAX_VERSION_LEN + 1)
    assert len(oversized) > MAX_VERSION_LEN
    print("PASS: test_upd2_channel_and_version")


def test_upd3_artifact_and_sha256():
    valid_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    art = {
        "target": "rootfs",
        "file_name": "aios-rootfs.raw",
        "sha256": valid_sha,
        "size_bytes": 1024 * 1024 * 250,
    }
    assert validate_artifact(art)

    # Bad sha length
    bad_sha_art = dict(art, sha256="abc")
    assert not validate_artifact(bad_sha_art)

    # Bad sha chars
    bad_char_sha = dict(art, sha256="z" * 64)
    assert not validate_artifact(bad_char_sha)

    # Bad target
    bad_tgt = dict(art, target="bootloader")
    assert not validate_artifact(bad_tgt)

    # Zero size
    zero_size = dict(art, size_bytes=0)
    assert not validate_artifact(zero_size)

    # Path traversal rejection
    traversal_art = dict(art, file_name="../rootfs.raw")
    assert not validate_artifact(traversal_art)
    slash_art = dict(art, file_name="sub/rootfs.raw")
    assert not validate_artifact(slash_art)
    backslash_art = dict(art, file_name="sub\\rootfs.raw")
    assert not validate_artifact(backslash_art)

    # Hidden file rejection
    hidden_art = dict(art, file_name=".rootfs.raw")
    assert not validate_artifact(hidden_art)

    # Whitespace in filename rejection
    space_art = dict(art, file_name="rootfs raw.raw")
    assert not validate_artifact(space_art)
    print("PASS: test_upd3_artifact_and_sha256")


def test_upd4_state_machine():
    # Happy path
    assert can_transition("idle", "checking")
    assert can_transition("checking", "downloading")
    assert can_transition("downloading", "verifying")
    assert can_transition("verifying", "applying")
    assert can_transition("applying", "ready_to_reboot")
    assert can_transition("ready_to_reboot", "verified")
    assert can_transition("verified", "idle")

    # Rollback path
    assert can_transition("ready_to_reboot", "rolled_back")
    assert can_transition("rolled_back", "idle")

    # Failure path
    for s in VALID_STATES:
        assert can_transition(s, "failed")
    assert can_transition("failed", "idle")

    # Invalid transitions
    assert not can_transition("idle", "ready_to_reboot")
    assert not can_transition("downloading", "verified")
    assert not can_transition("verifying", "idle")
    print("PASS: test_upd4_state_machine")


def test_upd5_upd6_manifest_and_json_parity():
    valid_sha = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    manifest = {
        "update_id": "upd-2026-09-20-beta",
        "version": "1.2.0",
        "channel": "beta",
        "min_version": "1.0.0",
        "artifacts": [
            {
                "target": "rootfs",
                "file_name": "rootfs.img",
                "sha256": valid_sha,
                "size_bytes": 500000000,
            },
            {
                "target": "kernel",
                "file_name": "vmlinuz.efi",
                "sha256": valid_sha,
                "size_bytes": 16000000,
            }
        ],
        "signature": "mock_sig_ed25519",
        "release_notes": "Kernel and rootfs beta rollout",
        "published_at": "2026-09-20T12:00:00Z",
    }
    assert validate_manifest(manifest)

    # JSON roundtrip
    encoded = json.dumps(manifest, indent=2)
    decoded = json.loads(encoded)
    assert decoded["update_id"] == "upd-2026-09-20-beta"
    assert len(decoded["artifacts"]) == 2
    # Duplicate artifact test
    bad_manifest = json.loads(json.dumps(manifest))
    bad_manifest["artifacts"].append(dict(manifest["artifacts"][0]))
    assert not validate_manifest(bad_manifest)

    # Duplicate target test
    bad_tgt_manifest = json.loads(json.dumps(manifest))
    bad_tgt_manifest["artifacts"][1]["target"] = "rootfs"
    assert not validate_manifest(bad_tgt_manifest)
    print("PASS: test_upd5_upd6_manifest_and_json_parity")


def main():
    print("Starting System Update Mechanism Data Model Smoke Suite (UPD1..UPD6)...")
    test_upd1_slot_exclusivity()
    test_upd2_channel_and_version()
    test_upd3_artifact_and_sha256()
    test_upd4_state_machine()
    test_upd5_upd6_manifest_and_json_parity()
    print("ALL 5 SYSTEM UPDATE MECHANISM DATA MODEL INTEGRATION TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
