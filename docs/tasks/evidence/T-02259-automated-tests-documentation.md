# T-02259 Evidence: Automated Tests — Documentation

**Task:** Document the automated tests of Grant Lifecycle for operators and agents.  
**Status:** COMPLETE  
**Date:** 2026-09-22  

## Documentation Deliverables

### Section 20 Added to PEP Decision Engine Documentation
Location: `docs/design/pep_decision_engine.md` (Section 20: Automated Grant Lifecycle Tests)

### Test Suite Reference

#### Rust Integration Tests (`aiosh-core/tests/test_pep_grant_automated.rs`)
| Vector | Test ID | Description | Run Command |
|---|---|---|---|
| AUTOGRANT1 | `test_autogrant1_scale_indexing` | 1000-grant store stress test | `cargo test -p aiosh-core --test test_pep_grant_automated` |
| AUTOGRANT2 | `test_autogrant2_attenuation_depth_chain` | 7-tier delegation depth enforcement | same |
| AUTOGRANT3 | `test_autogrant3_branching_cascade_revocation` | Recursive tree revocation (parent+children) | same |
| AUTOGRANT4 | `test_autogrant4_mass_expiration_sweep` | Bulk sweep of 100 expired grants | same |
| AUTOGRANT5 | `test_autogrant5_atomic_persistence_reload` | Save+reload round-trip fidelity | same |
| AUTOGRANT6 | `test_autogrant6_adversarial_fuzzing` | Invalid ID rejection (control chars, empty, overlong) | same |
| AUTOGRANT7 | `test_autogrant7_concurrent_thread_safety` | 10-thread parallel grant operations | same |
| AUTOGRANT8 | `test_autogrant8_cross_substrate_json` | JSON wire-format serialization parity | same |

#### MCP Integration Smoke Test (`code/aiosh-mcp/tests/test_pep_grant_automated_smoke.py`)
| Step | Tool Tested | What It Validates |
|---|---|---|
| [1] | `aios.pep.grant.issue` | Root grant issuance with scope_type, rights, delegation_depth |
| [2] | `aios.pep.grant.attenuate` | Multi-tier derivation, depth decrement, right expansion rejection |
| [3] | `aios.pep.grant.list` + `inspect` | Enumeration with subject filter, inspection by ID |
| [4] | `aios.pep.grant.validate` | Active validation, subject mismatch rejection, right mismatch rejection |
| [5] | `aios.pep.grant.revoke` | Cascade revocation (parent + children), post-revoke validation |
| [6] | `aios.pep.grant.sweep` | Expired grant cleanup, post-sweep state inspection |
| [7] | Negative boundaries | Duplicate ID, non-existent grant |

**Run:** `python code/aiosh-mcp/tests/test_pep_grant_automated_smoke.py`

#### MCP Unit Tests (`code/aiosh-mcp/tests/test_pep_grant_mcp.py`)
- 4 test functions, 16+ assertions covering happy-path and negative cases
- **Run:** `python code/aiosh-mcp/tests/test_pep_grant_mcp.py`

### For Operators
- All tests are self-contained and require no external dependencies
- Tests create temporary directories and clean up after themselves
- Build prerequisite: `cargo build -p aiosh-mcp` from `code/aiosh-rust/`
- Python 3.10+ required for MCP smoke tests

### For AI Agents
- Grant store is a JSON file at a configurable `store_path`
- All MCP grant tools accept `store_path` parameter for testing against isolated stores
- Standard JSON-RPC 2.0 protocol over stdin/stdout
