# T-00107 — Task Ledger Control / recovery & validation: Security Review

Date: 2026-08-23 · Status: REVIEW COMPLETE — **one low-severity finding
(F-1) identified and scheduled into T-00108 hardening**; no policy bypass
remains open at close of this task.

## Scope

The new `validate` action (Python `task_ledger.py`, Rust `ledger.rs`,
Rust+Python MCP surfaces, both CLIs). Threat model per SPEC §7 L1/L4 and
SECURITY.md scope (gate-order bypass, no-skip violation, secret exposure).

## Empirical probes (all run live this session)

| ID | Scenario | Result |
|---|---|---|
| S1 | Stray operand on CLI (`aiosh task validate 99`) | refused envelope on stderr, exit 1, audited ✅ |
| S2 | `validate` WITH `task_id` over Rust MCP wire | `ok:false` "does not take 'task_id'", `isError:true`, honest audit row (audit_id=3) ✅ |
| S3 | Same over Python MCP surface | identical refusal shape ✅ |
| S4 | Hostile event-controlled evidence paths (`../../../../etc/shadow`, `/etc/passwd`) injected into COMPLETIONS.jsonl | existence-oracle ONLY: names may appear in warning list; **no content ever read or returned** ("root:x:" absent from output) ⚠️→F-1 |
| S5 | Zero-mutation proof: snapshot sha256 of state/events/ledger before & after 6 pure validate/refusal calls | byte-identical ✅ |
| S6 | `rebuild` without grant over wire (regression pin of the historical P6 hole) | still `gate:"pep"` refused ✅ |
| S7 | Audit chain verify after all abuse traffic | verifies ok ✅ |

## Analysis

- **Input validation:** CLI accepts zero operands; MCP rejects unknown keys
  (`additionalProperties:false` → -32602), non-enum actions, and task_id
  presence on read-only validate. Both substrates agree (S2/S3).
- **Path/argument injection:** validate executes nothing; findings embed
  event-controlled strings only as JSON-escaped data. No shell-outs added.
- **PEP gating / audit emission:** validate is read-only ⇒ grant-free by
  design (recovery must be assessable during a permissions incident);
  every invocation — including refusals — emits exactly one hash-chained
  audit row through the unchanged classifier→PEP→audit gate (S2, S5-S7).
  The state-changing set (done/block/unblock/skip/rebuild) is untouched.
- **Untrusted-content handling:** event log content is treated as
  attacker-controllable; all outputs are structured JSON; size caps
  inherited from `AIOSH_LEDGER_MAX_*` readers.

## Finding F-1 (low, fix in T-00108)

Evidence-existence checks resolve event-supplied strings against the tasks
dir and repo root, but an **absolute path anywhere on disk is accepted as
"evidence exists"** (os.path.join / Path::join semantics) and produces no
finding. Impact: an actor able to write events already holds flock
(single-host trust model, SPEC L1), so this is tamper-evidence evasion,
not privilege escalation — severity LOW. Fix for hardening: classify
absolute or base-escaping evidence paths as `missing` (suspicious), never
as satisfied. No content disclosure in any case (verified S4).
