# T-00671 — Repository Health / security policy: Research

## 1. Research Context
The security policy sub-epic verifies the `check_security_governance` function in `aiosh-core::repo_health_service` and its integration with CLI and MCP surfaces.

## 2. Facts (Verified from Source Code)

| Fact | Source |
| :--- | :--- |
| `check_security_governance(repo_root: &Path) -> RepoHealthCheck` exists | [`repo_health_service.rs:137-178`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/repo_health_service.rs#L137-L178) |
| Checks: file existence, minimum 100 chars, absence of `TODO` markers | Same file, lines 149-168 |
| Category: `HealthCategory::SecurityGovernance` | Line 142 |
| Called by `check_repo_health` orchestrator | Line 180+ |
| `tools/check_security_policy.py` independently validates S1..S5 | [`tools/check_security_policy.py`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/tools/check_security_policy.py) |
| H4 criterion in `test_repo_health_suites.py` validates security governance | [`tools/test_repo_health_suites.py`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/tools/test_repo_health_suites.py) |

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **Implementation** | `check_security_governance` is fully implemented and tested. | No new code is needed; this sub-epic validates existing coverage. |
| **CLI Integration** | `aiosh repo health` includes security governance check in its report. | CLI output includes the check result in both prose and JSON modes. |
| **MCP Integration** | `aios.repo.health` MCP tool includes security governance in its response. | MCP response JSON contains the `security_governance` check entry. |

## 4. Key Design Decisions
1. No new code required — existing implementation is complete.
2. Sub-epic focuses on validation, documentation, and evidence capture.
