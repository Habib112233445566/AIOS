# T-00051 — Task Ledger Control: configuration Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00050
**Artifact note:** instruction name `T-00051-research.md`; ledger row
declares `T-00051-configuration-research.md` (mirrored).

Central question: what is configurable in the ledger stack today, what
is hardcoded that shouldn't be, and what mechanism should the
configuration component introduce?

## 1. Internal facts (grepped + read, 2026-08-22)

| # | Fact | Location |
|---|---|---|
| F1 | Env-var surface already exists: `AIOSH_TASKS_DIR` (data dir), `AIOSH_HOME` (audit db base), `AIOSH_CONSTITUTION` (constitution rev file), `AIOSH_MCP_ROOT` (legacy bridge) | ledger.rs:25-31, audit.rs:71-87, main.rs:30/62, agent_bridge.py:103 |
| F2 | Hardcoded operational knobs (all constants): lock timeout 5 s (`LOCK_TIMEOUT_SECS`, rust+py), file caps 64/16/4 MiB (`MAX_{LEDGER,EVENTS,STATE}_FILE_BYTES`), task text cap 4096 (`MAX_TASK_TEXT`), evidence cap 16 (`MAX_TASK_EVIDENCE`) | ledger.rs:76-78/583-, task_service.rs:75-76, task_ledger.py, server.py |
| F3 | There is NO config-file mechanism anywhere in the stack; every override today is an env var | grep "config" across core/cli: zero hits beyond comments |
| F4 | Caps/timeout are duplicated across TWO substrates and THREE call sites (rust consts vs python constants vs schema bounds in MCP manifest text) — drift risk is real but currently synchronized manually | inventory above |

## 2. External authoritative facts (fetched live 2026-08-22)

Source: The Twelve-Factor App, III. Config — <https://12factor.net/config>

| # | Fact |
|---|---|
| E1 | Config = everything likely to vary between deploys; **strict separation of config from code**; storing config as constants in code "is a violation of twelve-factor" |
| E2 | Env vars preferred over config files: language- and OS-agnostic standard; config files risk accidental check-in and scatter across formats — the source explicitly names those file-based weaknesses |
| E3 | Backing-service handles and per-deploy values belong in env; internal application wiring does not count as config |

## 3. Gap analysis

F2 violates E1: five operational knobs are code constants that
operators may legitimately need to vary (e.g., raising event-log cap
for a large ledger, lengthening lock timeout on slow storage). The fix
should follow the project's existing convention (F1) AND the authority
(E2): **environment variables**, not a new file format.

### Candidates

| Candidate | Verdict |
|---|---|
| **A. Env-var config layer**: one `LedgerConfig` sourced purely from `AIOSH_LEDGER_*` env vars with built-in defaults equal to today's values; identical loading in both substrates; `aiosh task config` prints effective values | **Recommended proposal (AIOS-specific)** |
| B. JSON/TOML config file | Rejected — contradicts E2's named file weaknesses; adds parse/locate/trust complexity; breaks substrate symmetry (no toml dep in core) |
| C. Status quo (constants) | Rejected — F2 is the literal violation E1 names |

## 4. Assumptions (marked)

- A1: operators only need these six knobs exposed now; other constants
  (schema version, rule string) are protocol identity, NOT deploy
  config — excluded.

## 5. Decisions needed before Specification (T-00052)

- **D1:** knob set = the six in F2? (default yes)
- **D2:** names `AIOSH_LEDGER_<KNOB>` exactly as above? (default yes)
- **D3:** invalid value handling: fail loudly at first use, naming the
  variable, zero silent fallback? (default yes)
- **D4:** expose via CLI `aiosh task config` (read-only print,
  audited)? (default yes)
- **D5:** deliberately NO MCP tool for config (agent-mutable security
  knobs = anti-security)? (default yes)
- **D6:** Python mirror reads the same vars with same defaults
  (parity law)? (default yes)

## 6. Acceptance check
- [x] Facts vs assumptions separated; citations with fetch date.
- [x] Decisions listed explicitly. [x] No code changed.
