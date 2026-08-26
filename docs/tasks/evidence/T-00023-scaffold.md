# T-00023 — Task Ledger Control core service: Scaffold

**Date:** 2026-08-22
**Type:** scaffold (interfaces only; bodies fail loudly)
**Depends on:** T-00022 spec (`T-00022-spec.md`)
**Artifact note:** instruction name `T-00023-scaffold` is realized as this
file; the ledger row's declared artifact name
(`T-00023-core-service-scaffold.md`) is mirrored byte-for-byte.

## What shipped

New module `code/aiosh-rust/aiosh-core/src/task_service.rs`, wired into
the crate root (`lib.rs`: `pub mod task_service;`). Typed interfaces
per the T-00022 spec, bodies `todo!("T-00024")`:

| Interface | Kind | Contract anchor |
|---|---|---|
| `TaskAction` {Status, Check, Done, Block, Unblock, Skip, Rebuild} | enum | spec §3 `action` enum |
| `TaskAction::parse / as_str / requires_grant` | fns | spec §3.1 grant column |
| `TaskCall<'a>` {action, task_id, note, reason, evidence} | struct | spec §3 inputSchema |
| `MAX_TEXT_LEN=4096`, `MAX_EVIDENCE_ITEMS=16` | consts | spec §3 schema bounds |
| `TaskCall::validate()` | fn | spec §3.3 pre-validation |
| `TaskCall::execute()` | fn | spec §3.1 persistence effects |
| `tasks_dir() -> Result<PathBuf, String>` | fn | spec §5 resolver (D3) |
| `status_call()` | test/call-site helper | wiring proof |

Deliberate separation: the service module owns validation + dispatch
into `ledger::`; gate ordering and audit wrapping stay at the MCP call
site (Implementation task), so no ledger logic is duplicated.

## Verification (real output)

```
$ cargo build            (code/aiosh-rust)
Finished `dev` profile … in 3.11s        # zero errors, zero warnings

$ cargo test
test task_service::tests::scaffold_bodies_are_unimplemented - should panic ... ok
test task_service::tests::scaffold_request_type_is_constructible ... ok
test result: ok. 54 passed; 0 failed          # was 52; +2 scaffold tests

$ bash code/aiosh-rust/ci/rust_smoke.sh
== RUST SMOKE SUITE PASS ==                    # no regression (wire contract untouched)
```

Scaffold warnings found during build (unused import ×2, unused param)
were fixed immediately to keep the repo's zero-warning standard; final
build above is clean.

## Acceptance check

- [x] Project builds with zero errors (and zero warnings).
- [x] New interfaces exist and are referenced by test stubs
      (`scaffold_bodies_are_unimplemented` calls `tasks_dir()`;
      `scaffold_request_type_is_constructible` constructs `TaskCall`).
- [x] Bodies fail loudly (`todo!` → panic "not yet implemented"),
      asserted by a `should_panic` test.
- [x] MCP surface deliberately NOT touched yet — manifest stays 12
      tools so the wire-contract smoke keeps passing until T-00024
      implements behavior + updates the count per spec §2.

## Handoff to T-00024 (Implementation)

Implement all `todo!` bodies exactly per spec §3.1/§3.3/§5/§6 (incl.
D4 replay semantics in `ledger.rs` + Python reference), register the
`aios.task` tool in `aiosh-mcp` (manifest 12→13), update
`rust_smoke.sh` count assertion + add task-tool wire cases, replace
`scaffold_bodies_are_unimplemented` with real behavior tests, keep the
zero-warning standard, then run the full CI suite.
