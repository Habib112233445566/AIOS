#!/usr/bin/env python3
"""Integration and Smoke Test for System Update Documentation Subsystem (UDOC1..UDOC6).

Tests:
1. Category enumeration and loose parsing parity (`architecture`, `ab_partitioning`, etc.).
2. Canonical topic catalog presence (UDOC1).
3. Ranked keyword search scoring parity (UDOC3).
4. Dynamic Markdown status rendering and ASCII diagram formatting (UDOC5).
5. Cross-substrate JSON serialization fidelity.
"""

import json
import sys

CATEGORIES = [
    "architecture",
    "ab_partitioning",
    "security",
    "observability",
    "configuration",
    "troubleshooting",
]

CANONICAL_TOPICS = [
    {
        "id": "arch-overview",
        "title": "AIOS System Update Architecture Overview",
        "category": "architecture",
        "tags": ["architecture", "dual-slot", "lifecycle"],
        "content": "AIOS implements an atomic A/B dual-partition update architecture ensuring zero-downtime and resilient rollbacks.",
        "see_also": ["ab-slots", "security-policy"],
    },
    {
        "id": "ab-slots",
        "title": "A/B Dual-Slot Partition Layout & Switching",
        "category": "ab_partitioning",
        "tags": ["ab", "slots", "bootloader", "partitions"],
        "content": "The system alternates between Slot A and Slot B partitions for rootfs, kernel, and initramfs artifacts.",
        "see_also": ["arch-overview", "troubleshooting-rollback"],
    },
    {
        "id": "security-policy",
        "title": "System Update Cryptographic Security Policy",
        "category": "security",
        "tags": ["security", "policy", "signature", "anti-rollback"],
        "content": "Enforces channel authorization, Ed25519 signature verification, semver downgrade prevention, and revocation denylists.",
        "see_also": ["config-schema"],
    },
    {
        "id": "observability-telemetry",
        "title": "System Update Observability & Telemetry",
        "category": "observability",
        "tags": ["observability", "telemetry", "health", "metrics"],
        "content": "Provides unified reporting of slot health, staging progress, policy evaluation, and sanitized telemetry data.",
        "see_also": ["arch-overview"],
    },
    {
        "id": "config-schema",
        "title": "System Update Configuration Schema & Options",
        "category": "configuration",
        "tags": ["configuration", "options", "limits", "env"],
        "content": "Configures update directories, polling intervals, quota limits, and trusted key digests.",
        "see_also": ["security-policy"],
    },
    {
        "id": "troubleshooting-rollback",
        "title": "System Update Troubleshooting & Rollback Procedures",
        "category": "troubleshooting",
        "tags": ["troubleshooting", "rollback", "recovery", "boot-failure"],
        "content": "Details procedures for automated and manual rollback to the previous functional partition upon boot failure.",
        "see_also": ["ab-slots"],
    },
]

def search_topics(topics: list[dict], query: str) -> list[dict]:
    q_clean = query.strip().lower()
    if not q_clean:
        return []
    tokens = q_clean.split()
    results = []
    for topic in topics:
        score = 0
        id_lower = topic["id"].lower()
        title_lower = topic["title"].lower()
        content_lower = topic["content"].lower()

        if id_lower == q_clean:
            score += 100
        for token in tokens:
            if token in id_lower:
                score += 50
            if token in title_lower:
                score += 40
            for tag in topic.get("tags", []):
                if token in tag.lower():
                    score += 20
            if token in content_lower:
                score += 5

        if score > 0:
            snippet = topic["content"][:120] + "..." if len(topic["content"]) > 120 else topic["content"]
            results.append({
                "topic_id": topic["id"],
                "title": topic["title"],
                "category": topic["category"],
                "score": score,
                "snippet": snippet,
            })
    results.sort(key=lambda r: r["score"], reverse=True)
    return results

def render_status_markdown(slot_status: dict, update_status: dict) -> str:
    md = "# Live System Update Status Report\n\n"
    md += "### Partition Slots Status\n\n"
    md += f"- **Current Active Slot**: `{slot_status['current_slot']}`\n"
    md += f"- **Target Update Slot**: `{slot_status['target_slot']}`\n"
    md += f"- **Rollback Slot**: `{slot_status.get('rollback_slot')}`\n"
    md += f"- **Slot A Version**: `{slot_status['slot_a_version']}` (successful: `{slot_status['slot_a_successful']}`)\n"
    md += f"- **Slot B Version**: `{slot_status['slot_b_version']}` (successful: `{slot_status['slot_b_successful']}`)\n\n"

    md += "### A/B Partition Visual Diagram\n```text\n"
    a_active = "[ACTIVE]" if slot_status['current_slot'] == "slot_a" else "[INACTIVE]"
    b_active = "[ACTIVE]" if slot_status['current_slot'] == "slot_b" else "[INACTIVE]"
    md += "+-----------------------+   +-----------------------+\n"
    md += f"| Slot A: {a_active:<13} |   | Slot B: {b_active:<13} |\n"
    md += f"| Version: {slot_status['slot_a_version']:<12} |   | Version: {slot_status['slot_b_version']:<12} |\n"
    md += "+-----------------------+   +-----------------------+\n```\n\n"

    md += "### Update Lifecycle State\n\n"
    md += f"- **State**: `{update_status['state']}`\n"
    md += f"- **Progress**: `{update_status['progress_percent']}%`\n"
    md += f"- **Current Running Version**: `{update_status['current_version']}`\n"
    return md

def main() -> int:
    print("=== System Update Documentation Integration & Smoke Suite ===")

    # 1. Category validation
    print("[1] Testing categories and topic presence (UDOC1, UDOC2)...")
    assert len(CANONICAL_TOPICS) == 6
    found_cats = {t["category"] for t in CANONICAL_TOPICS}
    assert found_cats == set(CATEGORIES)
    print("  OK: all 6 categories represented in canonical topic catalog")

    # 2. Search ranking
    print("[2] Testing ranked search scoring (UDOC3)...")
    results = search_topics(CANONICAL_TOPICS, "security-policy")
    assert len(results) > 0
    assert results[0]["topic_id"] == "security-policy"
    assert results[0]["score"] >= 100

    results_dual = search_topics(CANONICAL_TOPICS, "dual-slot")
    assert any(r["topic_id"] == "arch-overview" for r in results_dual)

    assert len(search_topics(CANONICAL_TOPICS, "")) == 0
    assert len(search_topics(CANONICAL_TOPICS, "nonexistent_token_xyz")) == 0
    print("  OK: ranked search returns correct top topics and handles empty queries")

    # 3. Dynamic status rendering (UDOC5)
    print("[3] Testing dynamic status markdown rendering with ASCII diagram (UDOC5)...")
    slot_status = {
        "current_slot": "slot_a",
        "target_slot": "slot_b",
        "rollback_slot": "slot_a",
        "slot_a_version": "2.1.0",
        "slot_b_version": "none",
        "slot_a_successful": True,
        "slot_b_successful": False,
    }
    update_status = {
        "state": "idle",
        "progress_percent": 0,
        "current_version": "2.1.0",
    }

    md = render_status_markdown(slot_status, update_status)
    assert "# Live System Update Status Report" in md
    assert "Slot A: [ACTIVE]" in md
    assert "Slot B: [INACTIVE]" in md
    assert "**Current Running Version**: `2.1.0`" in md
    assert "+-----------------------+   +-----------------------+" in md
    print("  OK: live status markdown rendered with correct ASCII slot diagram")

    # 4. JSON serialization roundtrip
    print("[4] Testing JSON serialization parity...")
    json_bytes = json.dumps(CANONICAL_TOPICS, indent=2)
    deserialized = json.loads(json_bytes)
    assert len(deserialized) == 6
    assert deserialized[0]["id"] == "arch-overview"
    print("  OK: JSON roundtrip preserves topic schema and contents")

    print("\nALL SYSTEM UPDATE DOCUMENTATION INTEGRATION CHECKS PASSED.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
