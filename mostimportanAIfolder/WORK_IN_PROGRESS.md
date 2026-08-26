# WORK IN PROGRESS — Userland process execution (run-to-exit)

> **Status: INCOMPLETE — do not treat this as done.**
> Last updated: 2026-08-15.
>
> **v2 amendment (2026-08-20):** the product vision has been restated —
> *"a Linux system for ethical hacking on the inside, a Windows-style desktop on
> the outside, with AI as a first-class S-rank kernel subsystem"*. In v2 the
> `kernel/` work is reclassified as **research substrate**, and the SMP
> `sscratch`-drift blocker in this file is preserved as a real bug worth fixing
> (the audit-ring primitive depends on the same context-restoration patterns)
> **but it is no longer the blocking critical path**. The new v2 critical path
> is **Pillar C (S-rank AI subsystem)**: MCP server + inference adapters + PEP
> + audit ring, before any Pillar A or Pillar B expansion.
> See `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0) and
> `mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0).
>
> This file is the single source of truth for what is finished, what is broken,
> and how to resume. Read it before continuing this line of work.

## Goal

The user asked to (1) **actually execute** the compiler-generated `hello` and
`uname` userland binaries so they print their output and exit (not just
validate/load/spawn), and (2) build a real RISC-V libc + standard tools
(`sh`, `ls`, `cat`) as ELF binaries.

Only (1) is in progress here. (2) has **not been started**.

## What is already committed (before this WIP)

Commit `fcfdede` — "feat: add RISC-V userland crate and load real ELF binaries
from disk":
- `userland/` crate (`hello`, `uname`) compiled by `kernel/build.rs`.
- `userland_smoke()` validates ELFs, loads, reads back through VFS, spawns.
- CI marker `AIOS-USERLAND ELF binaries: OK`.

That milestone only *spawned* the binaries; it did **not** execute them.

## What this WIP adds (uncommitted)

Four modified files (all uncommitted as of this snapshot):

1. **`kernel/src/user.rs`** — a synchronous `run_process_to_exit(pid)` runner:
   - `ResumeContext { ra, sp, satp }` + `static KERNEL_RESUME`.
   - `enter_user_saved` (naked) saves the kernel continuation and `sret`s into
     user mode.
   - `exit_trampoline` (naked) restores the kernel stack/satp/ra and returns to
     the runner's caller after the process exits.
   - `RUN_TO_EXIT_ACTIVE` atomic flag + `run_to_exit_active()`.

2. **`kernel/src/syscall.rs`**:
   - `USER_PROCESS_EXITED` atomic + `user_exited()` (read-and-clear).
   - Output capture: `user_out_start` / `user_out_stop` / `user_out_bytes`,
     plus `capture_user_byte` hooked into `sys_write`.
   - `sys_exit` now sets `USER_PROCESS_EXITED`.

3. **`kernel/src/interrupt.rs`**:
   - ECALL arm: after `dispatch_ecall`, if `user_exited() && run_to_exit_active()`,
     redirect `sret` to `exit_trampoline` (SPP=1, SPIE=1) instead of `sepc += 4`.
   - Diagnostic panic message (temporary) that also dumps `sscratch`, `satp`, `sp`.

4. **`kernel/src/main.rs`**:
   - `userland_smoke()` now actually RUNS `hello` then `uname`, asserts their
     captured `sys_write` output and `ProcState::Zombie`, and prints
     `AIOS-USERLAND ELF binaries: OK (hello + uname ran, printed, exited)`.

## What works

The happy path is **verified against real QEMU**:

```
hello from AINOS userland
[proc] killed PID 33 'hello'
[proc] PID 33 exited
[userland] hello ran, printed, and exited (Zombie)
AIOS-USERLAND ELF binaries: OK (hello + uname ran, printed, exited; PID 33 + 34)
```

Both binaries enter U-mode, trap on WRITE (prints), trap on EXIT (kills the
process), and control returns cleanly to the runner via the trampoline. The
kernel satp (Bare mode, `satp == 0`) is saved/restored correctly.

## What is BROKEN

There is an **intermittent panic late in the boot** (during the post-SMP smoke
suite, e.g. AIOS-0062 audit stress). Baseline commit `fcfdede` boots green
3/3 runs; with the run-to-exit enabled, the boot panics. Skipping only the
`run_process_to_exit` calls (keeping the trampoline + gating + capture code)
boots green 3/3, so the trigger is the actual user-mode execution, not the
auxiliary changes.

### Debug findings (5 boot runs, diagnostic panic dump)

| Run | scause | sepc | satp | sscratch | note |
|---|---|---|---|---|---|
| 1 | 0x2 illegal instr | 0x803ad992 | — | 0x0 | |
| 2 | 0x1 instr access fault | 0x1000000803702f8 | **0x0 (Bare)** | 0x80370210 | normal satp |
| 3 | 0x1 instr access fault | 0x168 | **0x0 (Bare)** | 0x8037c260 | |
| 4 | 0xf store fault | 0x8020111a (`trap_vector` `sd ra,0(sp)`) | **0x80000000000819e5 (user PT)** | **0xfef0** | |
| 5 | 0xf store fault | 0x8020111a | **user PT** | **0xfef0** | |

### Interpretation

- The kernel normally runs in **Bare mode** (`satp == 0`); user page tables are
  Sv39 (`satp == 0x8000_0000_0000_xxxx`).
- Runs 4/5 fail inside `trap_vector` at `sd ra, 0(sp)` with `sp = 0xfef0`,
  i.e. `sscratch` was already corrupted to `0x10000` (the user stack top) and
  the trap frame was allocated over ROM. The combined state
  `satp = user PT` + `sscratch = 0x10000` is exactly the state **mid-trap**
  while a hart is executing user-mode code.
- So a hart is left "stuck" in a user-mode trap state: `sscratch` holds the
  user stack pointer and the trap-exit `csrrw sp, sscratch, sp` that should
  restore `sscratch` to the trap stack is never reached, OR a hart is
  re-entering user mode with a stale `sscratch`.

### Most likely root cause (unconfirmed)

Pre-existing SMP fragility in `trap_vector` + `sched_tick`:

1. `trap_vector` has a known **sscratch drift**: the exit path does
   `csrrw sp, sscratch, sp` (a swap), so after every trap `sscratch` ends up
   at the trap *frame* pointer (`trap_stack_top - 272`), not the trap stack
   top. Each trap drifts the trap stack down 272 bytes.
2. The secondary harts pick up the `Ready` shell process (`/bin/sh`, left
   registered by `user_shell::smoke`) via `pick_next_scan`, switch to it
   (`context_switch_to` → `enter_current_user` → `enter_user`), and run it in
   U-mode.
3. When a timer ISR fires during that U-mode execution, `sched_tick` can call
   `context_switch_to` **from the trap handler on the trap stack** — saving the
   trap-handler context as if it were a thread context. Combined with the
   `sscratch` drift, this can leave a hart with `satp = user PT` and a
   `sscratch` pointing at user memory.

My run-to-exit changes make the primary hart actually enter/leave U-mode twice
early in boot, which shifts timing (and frees two TCB slots), exposing this
pre-existing race. The trampoline itself returns correctly on the happy path.

## Next steps (in order)

1. **Confirm the root cause** — instrument `sched_tick` / `context_switch_to` /
   `enter_current_user` to log `satp`/`sscratch`/`sp` and the TCB being
   switched to. Determine whether the faulting hart is the primary hart or a
   secondary hart running the shell.
2. **Fix the `sscratch` drift** in `trap_vector` (store the trap-stack top in
   a per-hart static and restore `sscratch` on exit instead of swapping), or
   make the exit path re-load `sscratch` from `PERCPU_TRAP_STACKS[hart]`.
3. **Decide the shell's fate during the boot smoke** — either reap the `Ready`
   shell before `init_smp()`, or ensure secondary harts don't context-switch
   from the timer ISR while a user process is running (guard `context_switch_to`
   against being called from the trap path, or defer it to the idle loop).
4. Once the boot is stable again, **re-enable the full `ci/smoke.sh` green** and
   commit.
5. **Then start part (2)** — the real RISC-V libc + tools (`sh`, `ls`, `cat`),
   which is the larger, not-yet-started follow-up.

## Files touched (uncommitted)

- `kernel/src/user.rs`
- `kernel/src/syscall.rs`
- `kernel/src/interrupt.rs`  (includes a temporary diagnostic panic dump)
- `kernel/src/main.rs`

## How to reproduce

```bash
cd /content/AIOS
source "$HOME/.cargo/env"
cd kernel && cargo build --target riscv64gc-unknown-none-elf
# boot with the CI-style QEMU command (smp 4, virtio-blk + net):
# see ci/smoke.sh for the exact invocation, or run `bash ci/smoke.sh`.
```

The panic is intermittent across runs (~2–3 of 5 runs), and the faulting hart
varies because QEMU/OpenSBI pick the boot hart non-deterministically.
