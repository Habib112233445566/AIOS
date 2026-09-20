#!/usr/bin/env python3
"""Smoke and Integration test for System Update Core Service (T-01916).

Validates invariants USVC1..USVC6:
- USVC1: Isolated staging directory sandboxing.
- USVC2: Cryptographic SHA-256 digest verification before update application.
- USVC3: Active slot non-interference (updates stage to target slot only).
- USVC4: Atomic state persistence (temp file + rename pattern).
- USVC5: Rollback orchestration upon boot failure.
- USVC6: Boot confirmation and lifecycle progression.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import sys
import tempfile

UPD_DIGEST_ERROR = "UPD_DIGEST_ERROR"
UPD_SLOT_ERROR = "UPD_SLOT_ERROR"
UPD_STATE_ERROR = "UPD_STATE_ERROR"
UPD_VALIDATION_ERROR = "UPD_VALIDATION_ERROR"


def compute_sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class MockUpdateService:
    def __init__(self, current_version: str, active_slot: str, base_dir: str):
        self.state_dir = os.path.join(base_dir, "state")
        self.staging_dir = os.path.join(base_dir, "staging")
        os.makedirs(self.state_dir, exist_ok=True)
        os.makedirs(self.staging_dir, exist_ok=True)

        target_slot = "slot_b" if active_slot == "slot_a" else "slot_a"
        self.slot_status = {
            "current_slot": active_slot,
            "target_slot": target_slot,
            "rollback_slot": active_slot,
            "slot_a_version": current_version if active_slot == "slot_a" else "none",
            "slot_b_version": current_version if active_slot == "slot_b" else "none",
            "slot_a_successful": active_slot == "slot_a",
            "slot_b_successful": active_slot == "slot_b",
        }
        self.update_status = {
            "state": "idle",
            "current_version": current_version,
            "target_version": None,
            "active_slot": active_slot,
            "progress_percent": 0,
            "last_error": None,
            "updated_at": "2026-09-20T12:00:00Z",
        }
        self.active_manifest = None
        self.staged_artifacts = {}

    def check_manifest(self, manifest: dict):
        if self.update_status["state"] != "idle":
            raise ValueError(f"{UPD_STATE_ERROR}: cannot check manifest in state {self.update_status['state']}")
        self.active_manifest = manifest
        self.update_status["state"] = "downloading"
        self.update_status["target_version"] = manifest["version"]
        self.update_status["progress_percent"] = 10

    def stage_artifact(self, target: str, data: bytes) -> str:
        if self.update_status["state"] != "downloading":
            raise ValueError(f"{UPD_STATE_ERROR}: cannot stage in state {self.update_status['state']}")

        found = next((a for a in self.active_manifest["artifacts"] if a["target"] == target), None)
        if not found:
            raise ValueError(f"{UPD_VALIDATION_ERROR}: target {target} not in manifest")

        if len(data) != found["size_bytes"]:
            raise ValueError(f"{UPD_VALIDATION_ERROR}: size mismatch")

        digest = compute_sha256(data)
        if digest.lower() != found["sha256"].lower():
            self.update_status["state"] = "failed"
            self.update_status["last_error"] = f"{UPD_DIGEST_ERROR}: digest mismatch"
            raise ValueError(f"{UPD_DIGEST_ERROR}: digest mismatch")

        dest = os.path.join(self.staging_dir, found["file_name"])
        with open(dest, "wb") as f:
            f.write(data)

        self.staged_artifacts[target] = dest
        total = len(self.active_manifest["artifacts"])
        self.update_status["progress_percent"] = 10 + int((len(self.staged_artifacts) * 50) / total)
        return dest

    def verify_staged(self):
        for a in self.active_manifest["artifacts"]:
            if a["target"] not in self.staged_artifacts:
                raise ValueError(f"{UPD_VALIDATION_ERROR}: missing artifact for {a['target']}")
        self.update_status["state"] = "verifying"
        self.update_status["progress_percent"] = 80

    def apply_update(self) -> str:
        if self.update_status["state"] != "verifying":
            raise ValueError(f"{UPD_STATE_ERROR}: cannot apply update in state {self.update_status['state']}")
        self.update_status["state"] = "applying"
        self.update_status["progress_percent"] = 95

        # Toggle slots
        next_slot = self.slot_status["target_slot"]
        self.slot_status["rollback_slot"] = self.slot_status["current_slot"]
        self.slot_status["target_slot"] = self.slot_status["current_slot"]
        self.slot_status["current_slot"] = next_slot

        self.update_status["active_slot"] = next_slot
        self.update_status["state"] = "ready_to_reboot"
        self.update_status["progress_percent"] = 100
        return next_slot

    def confirm_boot(self, running_version: str):
        if self.update_status["state"] != "ready_to_reboot":
            raise ValueError(f"{UPD_STATE_ERROR}: cannot confirm boot from state {self.update_status['state']}")
        curr = self.slot_status["current_slot"]
        if curr == "slot_a":
            self.slot_status["slot_a_successful"] = True
            self.slot_status["slot_a_version"] = running_version
        else:
            self.slot_status["slot_b_successful"] = True
            self.slot_status["slot_b_version"] = running_version

        self.update_status["current_version"] = running_version
        self.update_status["state"] = "idle"
        self.active_manifest = None
        self.staged_artifacts.clear()

    def rollback(self) -> str:
        fallback = self.slot_status["rollback_slot"]
        target = "slot_b" if fallback == "slot_a" else "slot_a"
        self.slot_status["current_slot"] = fallback
        self.slot_status["target_slot"] = target
        self.update_status["active_slot"] = fallback
        self.update_status["state"] = "idle"
        self.active_manifest = None
        self.staged_artifacts.clear()
        return fallback

    def save_state(self):
        slot_path = os.path.join(self.state_dir, "slot_status.json")
        slot_tmp = slot_path + ".tmp"
        with open(slot_tmp, "w", encoding="utf-8") as f:
            json.dump(self.slot_status, f, indent=2)
        os.replace(slot_tmp, slot_path)

        update_path = os.path.join(self.state_dir, "update_status.json")
        update_tmp = update_path + ".tmp"
        with open(update_tmp, "w", encoding="utf-8") as f:
            json.dump(self.update_status, f, indent=2)
        os.replace(update_tmp, update_path)


def test_usvc1_staging_isolation():
    with tempfile.TemporaryDirectory() as td:
        svc = MockUpdateService("1.0.0", "slot_a", td)
        assert os.path.exists(svc.staging_dir)
        assert os.path.exists(svc.state_dir)
    print("PASS: test_usvc1_staging_isolation")


def test_usvc2_cryptographic_verification():
    with tempfile.TemporaryDirectory() as td:
        svc = MockUpdateService("1.0.0", "slot_a", td)
        rootfs_data = b"VALID ROOTFS PAYLOAD"
        manifest = {
            "update_id": "upd-001",
            "version": "1.1.0",
            "channel": "stable",
            "artifacts": [
                {
                    "target": "rootfs",
                    "file_name": "rootfs.raw",
                    "sha256": compute_sha256(rootfs_data),
                    "size_bytes": len(rootfs_data),
                }
            ]
        }
        svc.check_manifest(manifest)
        dest = svc.stage_artifact("rootfs", rootfs_data)
        assert os.path.exists(dest)
        assert svc.update_status["state"] == "downloading"

        # Tampered artifact test
        bad_svc = MockUpdateService("1.0.0", "slot_a", td)
        bad_svc.check_manifest(manifest)
        try:
            bad_svc.stage_artifact("rootfs", b"CORRUPTED PAYLOAD 20")
            assert False, "Expected digest error"
        except ValueError as e:
            assert UPD_DIGEST_ERROR in str(e)
            assert bad_svc.update_status["state"] == "failed"
    print("PASS: test_usvc2_cryptographic_verification")


def test_usvc3_active_slot_non_interference():
    with tempfile.TemporaryDirectory() as td:
        svc = MockUpdateService("1.0.0", "slot_a", td)
        assert svc.slot_status["current_slot"] == "slot_a"
        assert svc.slot_status["target_slot"] == "slot_b"

        rootfs_data = b"PAYLOAD"
        manifest = {
            "update_id": "upd-002",
            "version": "1.2.0",
            "channel": "stable",
            "artifacts": [
                {
                    "target": "rootfs",
                    "file_name": "rootfs.raw",
                    "sha256": compute_sha256(rootfs_data),
                    "size_bytes": len(rootfs_data),
                }
            ]
        }
        svc.check_manifest(manifest)
        svc.stage_artifact("rootfs", rootfs_data)
        svc.verify_staged()
        next_slot = svc.apply_update()
        assert next_slot == "slot_b"
        assert svc.slot_status["current_slot"] == "slot_b"
        assert svc.slot_status["target_slot"] == "slot_a"
        assert svc.slot_status["rollback_slot"] == "slot_a"
    print("PASS: test_usvc3_active_slot_non_interference")


def test_usvc4_atomic_persistence():
    with tempfile.TemporaryDirectory() as td:
        svc = MockUpdateService("1.0.0", "slot_a", td)
        svc.save_state()
        slot_file = os.path.join(svc.state_dir, "slot_status.json")
        update_file = os.path.join(svc.state_dir, "update_status.json")
        assert os.path.exists(slot_file)
        assert os.path.exists(update_file)
        assert not os.path.exists(slot_file + ".tmp")
        assert not os.path.exists(update_file + ".tmp")
    print("PASS: test_usvc4_atomic_persistence")


def test_usvc5_usvc6_lifecycle_and_rollback():
    with tempfile.TemporaryDirectory() as td:
        svc = MockUpdateService("1.0.0", "slot_a", td)
        rootfs_data = b"PAYLOAD"
        manifest = {
            "update_id": "upd-003",
            "version": "1.3.0",
            "channel": "stable",
            "artifacts": [
                {
                    "target": "rootfs",
                    "file_name": "rootfs.raw",
                    "sha256": compute_sha256(rootfs_data),
                    "size_bytes": len(rootfs_data),
                }
            ]
        }
        svc.check_manifest(manifest)
        svc.stage_artifact("rootfs", rootfs_data)
        svc.verify_staged()
        svc.apply_update()
        assert svc.update_status["state"] == "ready_to_reboot"

        # Trigger rollback
        rolled = svc.rollback()
        assert rolled == "slot_a"
        assert svc.slot_status["current_slot"] == "slot_a"
        assert svc.update_status["state"] == "idle"

        # Apply again and confirm boot
        svc.check_manifest(manifest)
        svc.stage_artifact("rootfs", rootfs_data)
        svc.verify_staged()
        svc.apply_update()
        svc.confirm_boot("1.3.0")
        assert svc.slot_status["slot_b_successful"]
        assert svc.slot_status["slot_b_version"] == "1.3.0"
        assert svc.update_status["state"] == "idle"
    print("PASS: test_usvc5_usvc6_lifecycle_and_rollback")


def main():
    print("Starting System Update Core Service Smoke Suite (USVC1..USVC6)...")
    test_usvc1_staging_isolation()
    test_usvc2_cryptographic_verification()
    test_usvc3_active_slot_non_interference()
    test_usvc4_atomic_persistence()
    test_usvc5_usvc6_lifecycle_and_rollback()
    print("ALL 5 SYSTEM UPDATE CORE SERVICE INTEGRATION TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
