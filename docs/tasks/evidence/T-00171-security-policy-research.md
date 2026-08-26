# T-00171 — CI Smoke Orchestration / security policy: Research

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration security policy

## 1. Scope & Objective
Establish facts, constraints, and prior art for the security policy governing the CI Smoke Orchestration domain. 

## 2. Facts vs. Assumptions

### F1: Existing Policy (FACTS)
- **Repo Policy:** The root `SECURITY.md` defines the formal vulnerability disclosure terms and supported surfaces (OpenSSF Scorecard criteria S1-S5 enforced by `tools/check_security_policy.py`).
- **Security Knowledge Index:** `SECURITY.md` currently lists the task evidence documents for earlier epics (e.g. `T-00017`, `T-00027`, etc.) but lacks references to the CI Smoke Orchestration epics (`T-00111` through `T-00170`).

### F2: Orchestrator Security Profile (FACTS)
- The CI Orchestrator (`tools/ci_run.py`) manages arbitrary child process execution. Its security boundaries depend on fail-fast isolation (process group SIGTERM) and strict log bounds (truncation at `1MB` default) to prevent disk/memory exhaustion vectors.
- The Core CI Service (`aiosh-core/src/ci.rs`) and MCP surface (`aios.ci`) adhere to the strict ADR-0035 audit row standards.

## 3. Decisions Needed Before Implementation
- **D1 (Policy Update):** `SECURITY.md` needs to be updated to explicitly encompass the CI Smoke Orchestrator in the "Supported Surfaces" matrix.
- **D2 (Vulnerability Definition):** We must define what constitutes a vulnerability in the CI domain (e.g., escaping the timeout process group, log buffer overflow DOS).
- **D3 (Knowledge Index):** We must append the CI Smoke Orchestration security reviews to the Knowledge Index section of `SECURITY.md` while ensuring `tools/check_security_policy.py` continues to pass (S5 requires all referenced files to actually exist).
