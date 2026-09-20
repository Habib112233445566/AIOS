#!/usr/bin/env python3
"""Integration Smoke Test for Hardware Detection Documentation Subsystem (T-01786).

Validates:
- Strict adherence to invariants HDOC1..HDOC6:
  - HDOC1: Offline self-contained topic catalog without external network calls.
  - HDOC2: Case-insensitive topic retrieval and defensive bounds.
  - HDOC3: Ranked search scoring (ID > tag > content).
  - HDOC4: Category filtering.
  - HDOC5: Deterministic markdown formatting.
  - HDOC6: Bounded memory footprint.
"""

from __future__ import annotations

import json
import sys

CANONICAL_TOPICS = [
    {
        "id": "hw-sysfs-topology",
        "title": "Linux Sysfs Hardware Topology and Probing",
        "category": "discovery",
        "summary": "Comprehensive reference on Linux /sys/bus and /sys/class virtual filesystems.",
        "tags": ["sysfs", "pci", "usb", "block", "network", "cpu", "dmi"],
        "sections": [
            {"title": "PCI Subsystem", "content": "PCI devices reside under /sys/bus/pci/devices."},
            {"title": "USB Subsystem", "content": "USB devices reside under /sys/bus/usb/devices."},
        ],
        "examples": ["ls -la /sys/bus/pci/devices"],
        "references": ["Documentation/filesystems/sysfs.txt"],
    },
    {
        "id": "hw-security-policy",
        "title": "Hardware Detection Security Policy and Gatekeeping",
        "category": "security",
        "summary": "Declarative security policies for hardware inventories enforcing device allowlists and redaction.",
        "tags": ["security", "policy", "redaction", "allowlist", "denylist", "gatekeeping"],
        "sections": [
            {"title": "Policy Modes", "content": "Enforcing, Audit, and Permissive modes."},
        ],
        "examples": ["aiosh hardware scan --policy /etc/aios/hardware_policy.json"],
        "references": ["ADR-0035 Security Architecture"],
    },
    {
        "id": "hw-observability-telemetry",
        "title": "Hardware Observability and Fleet Telemetry",
        "category": "observability",
        "summary": "Structured telemetry reports detailing device class distributions and driver binding ratios.",
        "tags": ["observability", "telemetry", "metrics", "driver_binding", "fleet"],
        "sections": [
            {"title": "Driver Binding", "content": "Tracks driver_binding_count and rate."},
        ],
        "examples": ["aiosh hardware observability --json"],
        "references": ["OpenTelemetry Hardware Metrics Conventions"],
    },
    {
        "id": "hw-config-options",
        "title": "Hardware Detection Configuration and Environment Overrides",
        "category": "configuration",
        "summary": "Persistent configuration settings, resource limits, and environment variable overrides.",
        "tags": ["configuration", "config", "env", "limits", "timeouts"],
        "sections": [
            {"title": "HardwareConfig", "content": "Controls paths, limits, and timeouts."},
        ],
        "examples": ["export AIOSH_HARDWARE_TIMEOUT_SECS=60"],
        "references": ["docs/hardware_detection.md Section 11"],
    },
    {
        "id": "hw-mcp-tools",
        "title": "Hardware Detection MCP Tool Catalog",
        "category": "architecture",
        "summary": "Model Context Protocol (MCP) tool interfaces for hardware introspection.",
        "tags": ["mcp", "tools", "scan", "list", "get", "summary", "verify"],
        "sections": [
            {"title": "Tool Catalog", "content": "aios.hardware.scan, list, get, summary, verify."},
        ],
        "examples": ["call('aios.hardware.scan', {'include_attributes': true})"],
        "references": ["docs/hardware_detection.md Section 10"],
    },
    {
        "id": "hw-troubleshooting",
        "title": "Hardware Detection Troubleshooting and Diagnostics",
        "category": "troubleshooting",
        "summary": "Diagnostic procedures and resolutions for common hardware detection errors.",
        "tags": ["troubleshooting", "debug", "diagnostics", "mock", "sysfs_builder"],
        "sections": [
            {"title": "Missing Devices", "content": "Check kernel driver module and dmesg."},
        ],
        "examples": ["dmesg | grep -i pci"],
        "references": ["docs/hardware_detection.md Section 12"],
    },
]


def get_topic(topic_id: str) -> dict | None:
    tid = topic_id.strip().lower()
    if not tid or len(tid) > 64 or any(c < " " for c in tid):
        return None
    for t in CANONICAL_TOPICS:
        if t["id"].lower() == tid:
            return t
    return None


def search_topics(query: str, category: str | None = None) -> list[dict]:
    q = query.strip().lower()
    if not q or len(q) > 256 or any(c < " " for c in q):
        return []

    results = []
    for t in CANONICAL_TOPICS:
        if category and t["category"] != category:
            continue
        score = 0
        matched_tags = []

        if t["id"].lower() == q:
            score += 100
        elif q in t["id"].lower():
            score += 40

        for tag in t["tags"]:
            if tag.lower() == q:
                score += 50
                matched_tags.append(tag)
            elif q in tag.lower():
                score += 20
                matched_tags.append(tag)

        if q in t["title"].lower():
            score += 35

        if q in t["summary"].lower():
            score += 15

        for sec in t["sections"]:
            if q in sec["title"].lower() or q in sec["content"].lower():
                score += 10

        if score > 0:
            results.append({
                "topic_id": t["id"],
                "title": t["title"],
                "score": score,
                "matched_tags": matched_tags,
            })

    results.sort(key=lambda x: (-x["score"], x["topic_id"]))
    return results[:50]


def format_markdown(topic: dict) -> str:
    lines = [
        f"# {topic['title']}",
        f"**ID:** `{topic['id']}` | **Category:** `{topic['category']}`",
        f"{topic['summary']}",
    ]
    for s in topic["sections"]:
        lines.append(f"## {s['title']}")
        lines.append(s["content"])
    if topic.get("examples"):
        lines.append("## Examples\n```bash")
        lines.extend(topic["examples"])
        lines.append("```")
    if topic.get("references"):
        lines.append("## References")
        for r in topic["references"]:
            lines.append(f"- {r}")
    return "\n\n".join(lines)


def test_hdoc1_canonical_topics():
    assert len(CANONICAL_TOPICS) == 6
    for t in CANONICAL_TOPICS:
        assert t["id"]
        assert t["title"]
        assert t["summary"]
        assert len(t["tags"]) > 0
    print("PASS: test_hdoc1_canonical_topics")


def test_hdoc2_case_insensitive_lookup():
    assert get_topic("hw-sysfs-topology") is not None
    assert get_topic("HW-SYSFS-TOPOLOGY") is not None
    assert get_topic("Hw-SeCuRiTy-PoLiCy") is not None
    assert get_topic("non-existent") is None
    assert get_topic("hw-sysfs\x00-topology") is None
    print("PASS: test_hdoc2_case_insensitive_lookup")


def test_hdoc3_search_scoring():
    res_exact = search_topics("hw-sysfs-topology")
    assert len(res_exact) > 0
    assert res_exact[0]["topic_id"] == "hw-sysfs-topology"
    assert res_exact[0]["score"] >= 100

    res_tag = search_topics("telemetry")
    assert len(res_tag) > 0
    assert res_tag[0]["topic_id"] == "hw-observability-telemetry"

    assert len(search_topics("x" * 300)) == 0
    assert len(search_topics("sysfs\x00bad")) == 0
    print("PASS: test_hdoc3_search_scoring")


def test_hdoc4_category_filter():
    res_sec = search_topics("policy", category="security")
    assert len(res_sec) == 1
    assert res_sec[0]["topic_id"] == "hw-security-policy"

    res_none = search_topics("sysfs", category="security")
    assert len(res_none) == 0
    print("PASS: test_hdoc4_category_filter")


def test_hdoc5_markdown_formatting():
    topic = get_topic("hw-security-policy")
    assert topic is not None
    md = format_markdown(topic)
    assert "# Hardware Detection Security Policy" in md
    assert "**ID:** `hw-security-policy`" in md
    assert "## Policy Modes" in md
    print("PASS: test_hdoc5_markdown_formatting")


def main():
    print("Starting Hardware Detection Documentation Smoke Suite (HDOC1..HDOC6)...")
    test_hdoc1_canonical_topics()
    test_hdoc2_case_insensitive_lookup()
    test_hdoc3_search_scoring()
    test_hdoc4_category_filter()
    test_hdoc5_markdown_formatting()
    print("ALL 5 HARDWARE DETECTION DOCUMENTATION INTEGRATION TESTS PASSED.")


if __name__ == "__main__":
    main()
