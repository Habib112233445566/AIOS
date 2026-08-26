# T-00041 — Task Ledger Control: MCP/API surface Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00040
**Artifact note:** instruction name `T-00041-research.md`; ledger row
declares `T-00041-mcp-api-surface-research.md` (mirrored).

Central question: the `aios.task` tool ALREADY shipped on the Rust MCP
server (T-00024/T-00026). What does this component actually owe?

## 1. Internal facts (read + probed 2026-08-22)

| # | Fact | Anchor |
|---|---|---|
| F1 | Rust `aiosh-mcp` exposes 13 tools incl. grouped `aios.task` (7 actions; mutations PEP-gated; `-32602` schema errors; one audit row per call) | `code/aiosh-rust/aiosh-mcp/src/main.rs` |
| F2 | **Legacy Python reference server has NO task tools**: exactly 12 registered (fs.read, process.list, audit×5, pentest×5) | `code/aiosh-mcp/aiosh_mcp/server.py` + `pentest.py` |
| F3 | Python gate pattern ready to reuse: `_dispatch.dispatch(tool, command, args, target, grant_id, require_grant)` → verdict dict, then business call, `commit()` on error — identical ordering to Rust | `aiosh_mcp/_dispatch.py`, server.py rotate example |
| F4 | Ledger logic in the legacy substrate lives in `tools/task_ledger.py` (module-level paths bound at import from `AIOSH_TASKS_DIR`; rebuild replay semantics present since T-00024) | `tools/task_ledger.py` |
| F5 | Naming conventions differ by design: Python functions `aios_fs_read` (FastMCP underscore names); pentest gate names derive via first-underscore→dot helper. Rust uses dotted wire names directly. Existing smoke asserts SUBSET, not equality | `tests/test_smoke.py` (~line 160), `pentest.py::_audit_tool_name` |
| F6 | SPEC §7 L5 residual (recorded T-00030): "legacy Python MCP server has no aios.task — by design, frozen reference" — i.e., an agent pointed at the PYTHON server cannot manage the ledger at all | `docs/SPEC-TASK-LEDGER.md` §7 |
| F7 | Grant scoping: Rust gates `aios.task` under tool string `"aios.task"`; grants minted with `--tools "aios.task"` work today on Rust only | T-00027 S2 |

## 2. External facts

Reused from T-00021 (fetched live 2026-08-21, re-affirmed unchanged):
tool-name charset (E1), inputSchema requirements (E2), deterministic
ordering (E3), isError-vs-protocol-error channels (E4), server MUSTs:
validate inputs/access-controls/rate-limit/sanitize (E5). Nothing new
needed for this component.

## 3. Gap analysis

The ADR-0035 §D-2 promise ("MCP is the ONLY protocol") is satisfied on
the ship substrate but NOT on the reference substrate: an agent bound
to the Python server cannot read or advance the ledger through any
gated path. The genuine deliverable of this component is **MCP
cross-substrate parity for the ledger surface** — closing L5's
residual honestly rather than leaving a documented hole.

### Candidates

| Candidate | Verdict |
|---|---|
| **A. Port `aios.task` to the Python reference server**, mirroring Rust semantics (grouped action tool, per-action grant gating, caps, envelope shapes), reusing `_dispatch` + `task_ledger` | **Recommended proposal (AIOS-specific)** |
| B. Declare Python server out of sync permanently | Rejected — contradicts the reference-contract purpose of the legacy tree |
| C. New HTTP API | Rejected — no prior art; violates D-2 MCP-only binding |

## 4. Assumptions (marked)

- A1: keeping each substrate's existing naming convention (Python
  underscore function name `aios_task`; gate string `"aios.task"`) is
  acceptable divergence, matching how pentest tools already behave (F5).

## 5. Decisions needed before Specification (T-00042)

- **D1:** port scope = full 7-action mirror on Python server? (default yes)
- **D2:** gate tool string `"aios.task"` so ONE grant works across both servers? (default yes)
- **D3:** validation parity: non-empty note/reason, 4096/16 caps enforced server-side pre-gate? (default yes)
- **D4:** envelope parity: mirror Rust result JSON keys (`ok/action/data/error/audit_id`)?
- **D5:** CI: extend `test_smoke.py` registered-set expectation + add wire-level status/refusal checks; rust_smoke untouched?

## 6. Acceptance check
- [x] Facts vs assumptions separated; citations carried forward.
- [x] Decisions listed explicitly. [x] No code changed.
