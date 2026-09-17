# T-01536: Filesystem Layout - MCP/API Surface: Integration

## Metadata
- **Task ID:** `T-01536`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (6/10) — MCP/API Surface Integration
- **Dependencies:** `T-01535` (MCP/API Surface Unit Test)
- **Next Task:** `T-01537` (Filesystem Layout / MCP/API surface: Security Review)
- **Code changed:** `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py`,
  `tools/test_fs_layout_suites.py`

---

## 1. Scope & Objective

`T-01535` corrected the surface and proved the three defects fixed in isolation. This task wires the
new contract test into the subsystem's aggregate runner and extends the **cross-surface** suite so the
integrated path — CLI and MCP against the same store, the same audit ring and the same transition —
is what is asserted, not just the MCP side.

## 2. Integration Detail

### 2.1 New aggregate criterion FL8

`tools/test_fs_layout_suites.py` gains **FL8 — filesystem layout MCP contract (advertised schema,
audit target, destructive verdict)**, running `code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py`.
The runner's summary line moves from `(FL1..FL7)` to `(FL1..FL8)`. FL8 sits beside the existing
criteria rather than replacing any of them, so the unit-level contract and the cross-surface smoke
remain separately attributable.

### 2.2 FL6 extended for the integrated path (`test_manifest_contract_and_verdict_parity`)

The cross-surface suite grows from 9 to **10 groups**. The new group asserts, through the real
binaries:

1. **Advertised schema == accepted arguments for all ten tools** (plus
   `additionalProperties: false`), covering the widened `get.layout_id`/`get.store_path` and
   `validate.store_path` at the integration boundary.
2. **Audit-target parity across surfaces** — a CLI `register` and an MCP `register` of the same spec
   both record `target = 'parity-target-v1'`; the MCP row is located by the `audit_id` returned to
   the caller and read back through the CLI's audit tail (one shared audit DB). This is the
   cross-substrate form of the defect where the MCP spec-path form logged `None`.
3. **One transition, one verdict** — MCP `set_active` on a layout whose every partition is a quarter
   of the preset's reports `destructive_transition: true`, the CLI `layout diff` of the same
   transition on the same store reports `destructive: true`, and growing back reports `false`.

## 3. Verified Evidence

```
$ python tools/test_fs_layout_suites.py
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity
[+] FL7 filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)
[+] FL8 filesystem layout MCP contract (advertised schema, audit target, destructive verdict)

PASS: fs_layout_suites criteria (FL1..FL8)
```

```
$ python code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py
PASS: tools/list exposes all 10 aios.fs_layout.* tools; mutations require store_path
PASS: aiosh layout is advertised in root help and reachable via the fs-layout alias
PASS: CLI <-> MCP parity on fstab, validate, diff and probe
PASS: shared canonical store - CLI register/set-active visible to MCP and vice versa
PASS: store_path semantics parity (fallback, corrupt, oversized, control chars)
PASS: mutations require BOTH a PEP grant and an explicit store_path; reads stay ungated
PASS: granted register/get/list/set_active/probe/import_fstab/remove round-trip persisted
PASS: CLI <-> MCP mutation parity (identical layouts, mutual pointer visibility)
SKIP: FIFO stall case needs POSIX mkfifo (Windows host) - covered on POSIX CI
PASS: spec/fstab reads are type-checked, bounded and non-blocking (F-1 closed)
PASS: advertised schema contract + audit-target and destructive-verdict parity

ALL FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION TESTS PASSED!
```

Crate-level regression across the touched packages:

```
$ cargo test -p aiosh-mcp   →  passed=15 failed=0
$ cargo test -p aiosh-core  →  passed=582 failed=0
$ cargo test -p aiosh-cli   →  passed=24  failed=0
```

### 3.1 The three audited defects, re-checked end-to-end after integration

| Defect | End-to-end evidence in this task |
|---|---|
| 1 — `get`/`validate` schema drift | FL6 group 10 asserts all ten tools' advertised properties equal the accepted set, so the widening survives the integration boundary; FL8/C1 asserts the same in isolation |
| 2 — `register` audit target differed by input form | FL8/C2 (MCP ring + audit tail) and FL6 group 10 (CLI row and MCP row both `target = 'parity-target-v1'`) |
| 3 — destructive verdict only ever asserted `false` | FL8/C3 and FL6 group 10 (`true` on shrink via MCP, `true` from the CLI's own `diff` on the same store, `false` on grow) |

### 3.2 Acceptance mapping

| Acceptance criterion (`T-01536`) | Result |
|---|---|
| Feature reachable through its production surface | all ten tools exercised over stdio through the real `aiosh-mcp` binary; CLI reaches the same store |
| Integration smoke passes end-to-end | FL6 10/10 groups, FL1..FL8 aggregate green |
| (ledger) update the registration point so the surface is discoverable | FL8 registered in `tools/test_fs_layout_suites.py`; manifest discoverability asserted for all ten tools |
| (ledger) confirm cross-substrate parity on the shared canonical JSON | CLI↔MCP parity on layout bodies, active pointer, audit target and destructive verdict |

## 4. Honest Limitations

1. **FL6 and FL8 overlap by design** — C1/C2/C3 assert the same three facts in isolation while FL6
   group 10 asserts them across surfaces. The duplication is the price of attributing a failure to
   either the contract or the integration; a shared assertion module would remove it.
2. **The accepted-argument table now exists three times** (in-tree Rust, FL8, FL6 group 10). The
   integration copy was kept because it is the one that catches a manifest regression after a
   rebuild, but drift between the three copies is possible; deriving the schema from the arms would
   close it properly (carried from `T-01535` §4.3).
3. **The FIFO stall case still skips on this Windows host** (POSIX-only `os.mkfifo`); the type
   refusal is proven via a directory on the same code path.
4. **Nothing was wired into the Python MCP bridge** (`code/aiosh-mcp/aiosh_mcp/`) — the ten tools
   exist only in the Rust server, which is the binary the sub-epic targets; the bridge surfaces
   `fs_layout` not at all.
5. **Documentation is still stale** (`code/aiosh-mcp/README.md`, `docs/filesystem_layout.md` §5) and
   remains owned by `T-01539`.

## 5. Citations

1. `docs/tasks/evidence/T-01535-mcp-api-surface-unit-test.md` — the three fixes and C1..C4.
2. `docs/tasks/evidence/T-01532-spec.md` — §9 (audit target), §10 (acceptance/test plan for
   `T-01534`/`T-01535`/`T-01536`).
3. `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py` — FL6, group 10.
4. `tools/test_fs_layout_suites.py` — FL1..FL8 aggregate runner.
