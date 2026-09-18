# AIOS Security Audit — 2026-09-18

**Scope:** whole repository. Findings were reproduced against `target/debug` binaries built from
head **`55b43b2`** (`feat/user-session-bootstrap`), with `AIOSH_HOME` and `cwd` redirected into
throwaway temp directories so no real state was touched.
**Surfaces:** `code/aiosh-rust/*` (CLI + MCP + core + sandbox), `code/aiosh-cli` (TS reference),
`code/aiosh-mcp/aiosh_mcp` (Python reference), `tools/`, `ci/`, dependency lockfiles, git hygiene.

**Status of this document:** it is the audit as delivered, plus a corrections pass (§Corrections)
that removed a claim the audit had not earned and reclassified one finding the reproduction turned
out to contradict — F-06 was overstated as reproduced in the first draft and is now labelled
precisely; the git-history secret claim was asserted before the scan was run. What the reproduction
added is recorded as F-16.

---

## Severity summary

| # | Sev | Finding | Confirmed by |
|---|-----|---------|--------------|
| F-01 | **Critical** | Unauthenticated, unvalidated `store_path` write in the MCP mutation surface (and a path to destroying `audit.db`) | live |
| F-02 | **Critical** | Audit ring is not concurrency-safe — the SHA-256 chain forks and `verify()` reports it broken | live |
| F-03 | **High** | Mutating MCP tools missing from `is_irreversible` and dispatched with `require_grant=false` | live |
| F-04 | **High** | Release/backup PEP check accepts any non-null string as a grant token (forged token passes) | live |
| F-05 | **High** | Landlock FS confinement is inert: rules are never bound to a path (`AT_FDCWD` as `parent_fd`) | source + man page (**not executed on Linux**) |
| F-06 | **High** | `commit()` panics *after* the mutation (`dispatch.rs:214`), so a failed audit write can leave state changed with no row | live panic; consequence **inferred** |
| F-07 | Medium | Default-allow on empty scope dimensions (`scope.networks`, `scope.paths`, unknown tools) | source |
| F-08 | Medium | "Sandboxed" run on non-Linux executes fully unsandboxed, and the Rust audit row never records the component outcome | source |
| F-09 | Medium | Unknown seccomp denylist names are silently dropped; no filter installed and no log line | source |
| F-10 | Medium | No redaction filter on audit payloads; `redact_secret_value` only serves the scanner | source |
| F-11 | Medium | Grant ids on Windows come from `sqlite3_randomness`, not an OS CSPRNG (comment says otherwise) | source |
| F-12 | Medium | No CI pipeline in this branch → no dependency/SAST/advisory gate; unbounded Python deps; `.env`/`*.pem` not ignored | source |
| F-13 | Low | `$PATH` resolution for `aiosh run` (PATH hijack); non-Linux branch executes unresolved `argv[0]` | source |
| F-14 | Low | `.unwrap()` panics on a second connection to the audit DB (`audit.rotate`) | source |
| F-15 | Low | Dead code and stream-of-consciousness comments on the security surface (stale security claims) | source |
| F-16 | Medium | **MCP server panics at startup (exit 101) when the ring is busy** — `SQLITE_BUSY` is not tolerated | live |

---

## Coverage: which audits were performed, and which were not

The request was "every type of audit". Here is the honest ledger of what that produced.

**Performed**

| Audit type | How | Result |
|---|---|---|
| Manual source review of every trust boundary | PEP/classifier/dispatch/audit/ledger/sandbox + both MCP servers + TS CLI | Findings F-01…F-16 |
| Static pattern sweep (SAST-lite, by hand) | `unsafe`, `Command::new`/`spawn`, `os.system`/`popen`/`shell=True`/`eval`/`exec`/`pickle`, `unwrap`/`expect`, path joins | No shell interpolation anywhere; `unsafe` confined to syscall wrappers |
| Authorization / authzZ model audit | `pep.rs`, `is_irreversible`, per-tool `require_grant`, release-surface policy fns | F-01, F-03, F-04, F-07 |
| Audit-integrity audit | hash chain, segments, retention, concurrency | F-02, F-06, F-16 |
| Input-validation audit | MCP `inputSchema` vs arm contracts, bounds, control chars, JSON parsing | C1 contract test; length checks exist but are not path checks (F-01) |
| Secrets scanning (pattern-based) | High-signal regexes over **all 59 reachable revisions**, not just the working tree | Matches only in `secrets_service.rs` test fixtures; no real credential in history |
| Crypto review | SHA-256 chain, entropy sources, comparison | F-11; chain construction itself sound |
| Dependency / supply-chain inventory | committed `Cargo.lock`, `package-lock.json`, `pyproject.toml` | F-12 |
| CI/CD audit | repository + `ci/` | F-12 (no pipeline in this branch) |
| Dynamic verification (DAST-lite) | live JSON-RPC over stdio against the real binary | F-01, F-02, F-03, F-04, F-16 |
| Regression / negative control | comparator fix reverted, rebuilt, re-run | C8 is load-bearing (see Corrections) |

**Not performed — declared, not silently skipped**

* **Advisory/CVE matching** (`cargo-audit`, `osv-scanner`, `npm audit`): none is installed and no
  advisory database is reachable from this environment. The dependency table is an inventory, not a
  vulnerability verdict.
* **Real SAST tooling** (semgrep/codeql/bandit) and **secret scanners** (gitleaks/trufflehog with
  entropy + verification): unavailable. My sweeps are regex- and review-based, so they inherit my
  pattern list's blind spots.
* **Fuzzing / property testing**, **mutation testing**, and **coverage-guided review**: not run.
* **Linux runtime verification**: host is Windows. F-05 (Landlock) and F-08/F-09 (seccomp) are
  source-and-documentation conclusions and were **not** executed; no `strace`/seccomp-trace evidence
  is offered for them.
* **Container/base-image, IaC, and licence-compliance audits**: nothing to scan in this repository
  (no Dockerfiles, no Terraform/Helm, `license = "TBD"` in the workspace manifest is the only signal).
* **Prompt-injection / LLM red-teaming of the agent loop**: only the static classifier rules
  (`PROMPT_INJECTION_FRAGMENTS`) and the C6 nested-injection case were examined; no adversarial
  session was run against `aiosh agent`.
* **Timestamp of this audit vs. the tree:** findings were re-run at `55b43b2`; `84c34ac` is the
  parent. Nothing in this document was verified against a later commit.

---

## F-01 — Unauthenticated, unvalidated `store_path` write (Critical)

**Where:** every MCP mutation that takes a `store_path` — `aios.handoff.*`
(`aiosh-mcp/src/main.rs:1211+`), `aios.service.action` (`:2371`, via `resolve_service_store`
`:4560`), `aios.package.apply` (`:2020`), and friends.

**What:** `store_path` is only checked for `len > 1024` and control characters. It is then used as
the file the tool loads, mutates and writes back, with **no grant** (`require_grant = false`) and
`path_subjects = &[]`, so `scope.paths` never sees it. `HandoffStore` writes via atomic rename.

**Live evidence at `55b43b2`** (temp `AIOSH_HOME`, no grant supplied anywhere):

```
control: fs.read ungranted             rc=0 gate='pep'  ok=False :: requires explicit PEP grant
control: fs_layout.register ungranted  rc=0 gate='pep'  ok=False :: requires explicit PEP grant
F-03: service.action ungranted         rc=0 gate=None   ok=False :: service store file ... does not exist
F-03: package.apply ungranted          rc=0 gate=None   ok=False :: failed to open package store at ...
F-01: handoff.initiate ungranted       rc=0 gate=None   ok=True   <-- mutation performed, no grant
F-01: aim store at live audit.db       rc=0 gate=None   ok=False :: Failed to atomically rename store file: Access is denied. (os error 5)
files created: ['.aios\\audit.db', '.aios\\audit.tmp', '.aios\\handoff_store.json']
```

Two conclusions:

1. An ungranted caller can create or overwrite **any file the process can write**, with
   attacker-influenced JSON content, wherever it likes — no grant, no `scope.paths` check. Note the
   `gate=None` cells: the response carries no gate field precisely because the gate passed.
2. Item 5 is the alarming one. The call reached the *rename onto `audit.db`* step and was stopped
   only by a **Windows sharing violation**, not by a policy. On POSIX an `rename()` over an open file
   succeeds, so the same request would replace the live audit ring with a handoff store — the
   in-scope violation in `SECURITY.md` ("Breaking the audit-ring hash chain, truncation
   detection…"). The current protection is accidental and platform-specific.

**Fix:** treat `store_path` as a path subject (`path_subjects=[store_path]`) so `scope.paths`
governs it; require a grant for every store-mutating tool; and refuse any store path outside an
allow-listed root (default `$AIOSH_HOME`).

## F-02 — Audit chain forks under concurrency (Critical)

**Where:** `aiosh-core/src/audit.rs` → `AuditRing::write` (head read, then `INSERT`).

**What:** `write()` does `head_hash()` (a `SELECT`) and then a separate `INSERT` — no
`BEGIN IMMEDIATE`, no `busy_timeout`, no flock. The task *ledger* takes an advisory `flock`
(`ledger.rs`); the audit ring does not. Two writers (CLI + MCP, or two agents) can read the same
head and both extend it, producing a forked, still-committed chain.

**Live evidence at `55b43b2`** — 12 concurrent gate-passing calls into one ring:

```
return codes: [0,0,0,0,0,0,101,0,0,0,0,0]
verify: ok=False checked=8 broken_at=9
rows=19 non-linear links=6
```

The chain self-reports as broken, so an operator cannot distinguish concurrent use from tampering —
the tamper-evidence guarantee is lost under ordinary parallelism. The `101` is a panic from a sibling
path: `dispatch.rs:214` when a row write loses the race (F-06) or `main.rs:32` when the ring cannot
be opened at all (F-16).

**Fix:** perform head-read + hash + insert inside one `BEGIN IMMEDIATE` transaction with
`PRAGMA busy_timeout`, or take a single-writer flock on the ring (mirroring the ledger).

## F-03 — Mutating tools the PEP does not consider irreversible (High)

`pep::is_irreversible` is the PEP's own defense-in-depth list, but it omits:

`aios.service.action` (start/stop/restart/reload/enable/disable/mask/unmask),
`aios.package.apply`, `aios.handoff.initiate|accept|reject|complete|cancel`,
`aios.triage.record|resolve`, `aios.web.gobuster`.

All of these are dispatched with `require_grant = false`, so with no grant at all the gate returns
`Ok(())` and the body runs. Live proof above: they returned results with **no `gate` field** (they
ran) while `aios.fs.read`, `aios.fs_layout.register` and `aios.audit.rotate` correctly returned
`gate=pep`. This is the exact class of omission that the T-01537 S-3 note in `pep.rs` describes
("they were reachable *only* because every call site also passes `require_grant = true`").

**Fix:** extend `is_irreversible` to the list above, and derive the flag per tool rather than
hand-passing it at 84 call sites.

## F-04 — Forged grant tokens pass the release/backup gate (High)

**Where:** `aiosh-core/src/release.rs:119` → `check_release_policy(_grant, action)` tests only
`is_none()` (not even the empty-string case that `check_doc_index_policy`, `check_toolchain_policy`
and `check_evidence_policy` do test).

**Live evidence at `55b43b2`** — same forged token, two tools:

```
backup.restore, forged grant_id=x   rc=0 gate=None  ok=False :: Failed to open backup: ...   <-- gate passed
audit.rotate,  forged grant_id=x    rc=0 gate='pep' ok=False :: unknown or revoked grant: x  <-- correct
```

`aios.backup.restore` takes caller-controlled `backup_path` and `target_dir`, so a forged token buys
an unauthenticated restore/write path. The zip handling itself is solid (§Solid), so this is a gate
failure, not a zip failure.

**Fix:** route it through `PepStore::check_with_paths` (existence, revocation, expiry,
`scope.tools`, `scope.paths` on `target_dir`).

## F-05 — Landlock confinement is inert (High, **source-level; not executed on Linux**)

**Where:** `aiosh-core/src/sandbox.rs:293-380` (same bug in `aiosh_mcp/sandbox.py:301-367`).

`landlock_add_rule`'s `rule_attr` carries `{ u64 allowed_access; s32 parent_fd }` — the *path* is
not in the structure; `parent_fd` must be "an **opened file descriptor**, preferably with the O_PATH
flag, which identifies the parent directory of the file hierarchy or just a file"
(`landlock_add_rule(2)`; the same page lists `EBADF` for "a member of rule_attr is not a file
descriptor as expected"). This implementation never opens `rule.path`:

```rust
let c_path = std::ffi::CString::new(rule.path.as_str()).unwrap_or_default();
let attr = LandlockPathBeneathAttr { allowed_access: bits, parent_fd: AT_FDCWD };
...
let _ = c_path;                       // path computed, then discarded
```

`AT_FDCWD` (−100) is not an open fd, so every `landlock_add_rule` is expected to fail; `ok` becomes
false and **`landlock_restrict_self` is never called** — no filesystem restriction installed, even
on a kernel ≥ 5.13 built with `CONFIG_SECURITY_LANDLOCK=y`. Corroborating evidence: the
`ENOENT → continue // path doesn't exist — non-fatal` branch is unreachable, because no path is ever
looked up. The `SPRINT-0.md` claim that "with Landlock enforced, `/etc/shadow` read blocked by
EACCES" therefore cannot hold. Effective confinement reduces to seccomp + `no_new_privs`.

**Caveat:** the host is Windows, so this is a code-and-manpage conclusion. The decisive experiment —
run `aiosh run` on a Landlock-capable kernel with `strace` or a seccomp trace and observe the
`landlock_add_rule` errno — has not been performed.

Related: the Rust `aiosh run` path never records the component outcome. `sandbox_exec` leaves the
child's stderr attached to the terminal, so `emit_sandbox_applied`'s line is never parsed by the
parent — `parse_sandbox_applied` is dead code referenced only by its own unit test. The audit row
carries the *requested* policy JSON, so in the Rust surface an inert/FAILed sandbox is
indistinguishable from an enforced one (the Python server does capture it correctly).

**Fix:** `open(path, O_PATH|O_CLOEXEC)` per rule and pass that fd as `parent_fd` (closing it after
`add_rule`); pipe the child's stderr and persist the parsed components in the audit row.

## F-06 — Audit write failure panics after the mutation (High)

`dispatch::commit` ends with `.expect("audit row write failed")` at
`aiosh-core/src/dispatch.rs:214` and is called *after* the tool body has run
(`recorded_call_with_body_target`). If that write fails (SQLITE_BUSY, disk full), the failure mode
is: state changed → no audit row → process aborts, which is what `SECURITY.md` calls a
vulnerability ("mutating state without exactly one honest audit row").

**Live evidence at `55b43b2`** — 24 concurrent instances, two died on that exact line:

```
nonzero exits: [(101, "thread 'main' panicked at aiosh-core\src\dispatch.rs:214:6:"),
                (101, "thread 'main' panicked at aiosh-core\src\dispatch.rs:214:6:")]
```

The panic site is confirmed to be the `.expect("audit row write failed")` line. It is **flaky**:
across four bursts of 16–24 concurrent instances it appeared as 1/24, 2/24, 0/24 and 0/24,
and it was only ever observed on read-only tools (`aios.audit.tail`).

**What is inferred, not observed:** the consequence that matters — a *mutating* tool whose body has
already written its store, followed by a panic in `commit` — was not captured. A targeted attempt
(24 concurrent `aios.handoff.initiate` calls, one store file each) produced no panic at all: 24
files written and 24 matching audit rows. The ordering in `recorded_call_with_body_target`
(body runs, then `commit`) makes the breach a certainty *if* the panic lands there, but the
observation is of the panic, not of the lost row.

**Fix:** never panic in the gate; make the body + row a unit of work, or on write failure return the
error and mark the mutation for reconciliation rather than losing the row.

## F-16 — MCP server panics at startup when the ring is busy (Medium, live)

**Where:** `aiosh-mcp/src/main.rs:32` — `AuditRing::open(...).expect("open audit db")`.

**What:** the ring is opened without any `busy_timeout`, so a second instance (or any process holding
the write lock) makes startup panic instead of retrying or degrading.

**Live evidence at `55b43b2`** — 24 concurrent instances, one died:

```
nonzero exits: 1 of 24
  rc=101 stderr: thread 'main' (11744) panicked at aiosh-mcp\src\main.rs:32:60:
  open audit db: SqliteFailure(Error { code: DatabaseBusy, extended_code: 5 },
  Some("database is locked"))
leftover files in AIOSH_HOME: ['audit.db']
```

Same root cause family as F-02 (no `busy_timeout` anywhere on the ring): the difference is that
F-02 corrupts the chain while this one simply kills the server. Availability impact is real for any
deployment that runs the CLI and the MCP server against one `AIOSH_HOME`.

**Fix:** `PRAGMA busy_timeout` (and/or a startup retry loop) before the ring is used; never
`.expect()` on the audit DB.

## F-07 — Default-allow on omitted scope dimensions (Medium)

* `scope.networks` empty on a `pentest.*` grant ⇒ `network_allowed` is never consulted, so any
  target host (including the public internet) is in scope.
* `GrantScope::paths` empty ⇒ `path_allowed` returns `true`.
* `check_with_paths(None, tool, …)` returns `Ok(())` for every tool not in `is_irreversible`.

Each is defensible in isolation; together they mean "a grant exists" is doing the work that "a grant
authorizes *this*" should do. Recommend deny-by-default for consequential tools and requiring an
explicit `0.0.0.0/0` for open network scope.

## F-08/F-09 — Sandbox honesty gaps (Medium)

* `sandbox_exec` (`#[cfg(not(target_os = "linux"))]`) simply runs the command; every component
  reports `FAIL`, and the `aiosh run` audit row (F-05) doesn't carry it. Calling that "sandboxed"
  overstates the control on Windows/macOS.
* `resolve_denylist` silently drops names absent from its 17-entry table (e.g. `unshare`, `bpf`,
  `userfaultfd`, `process_vm_readv` are not deniable), and if the resolved list is empty no seccomp
  line is emitted at all. A policy that denies nothing should say so loudly.

## F-10/F-11 — Data hygiene (Medium)

* **No redaction on audit payloads.** The only `redact` call sites in the Rust tree are
  `secrets.rs`/`secrets_service.rs` (building the scanner's own findings) and the CLI's findings
  display; `args` and `outcome_detail` are persisted verbatim by `AuditRing::write`. The scanner
  heuristics themselves (SEC-004/005) are substring tests that miss most real formats, and
  `redact_secret_value` discloses first+last 4 characters.
* **Grant id entropy on Windows.** `pep.rs::random_hex` uses `/dev/urandom` on Unix but
  `sqlite3_randomness` elsewhere, with a comment claiming "CryptGenRandom / BCryptGenRandom".
  SQLite's PRNG is an internal, RC4-based generator seeded from the VFS, not a documented CSPRNG.
  Grant ids are bearer tokens for irreversible actions and are 8 bytes (64 bits) even when strong.
  Use `BCryptGenRandom`/`getrandom`, and raise to 16 bytes.

## F-12 — Supply chain and CI (Medium)

* **No pipeline exists in this branch.** `.github/` is absent; the only entrypoint is
  `ci/run_all_smokes.sh`, which shells out to `tools/ci_run.py`. Nothing pins, audits, or gates
  dependencies, and nothing runs on push.
* `Cargo.lock` and `package-lock.json` are committed (good) but never verified. Inventoried
  versions look current (`rusqlite 0.32.1` bundled, `libsqlite3-sys 0.30.1`, `zip 2.4.2`,
  `chrono 0.4.45`, `serde_json 1.0.151`, `sha2 0.10.9`, `rustls 0.23.43`) — see the coverage table:
  no advisory database was consulted, so this is inventory, not a clean bill of health.
* `code/aiosh-mcp/pyproject.toml` pins nothing (`fastmcp>=0.4.0`, `mcp>=1.0.0`) → non-reproducible
  builds for a security-relevant component.
* `.gitignore` covers audit DBs, target dirs and model weights but **not** `.env`, `*.pem`, `*.key`,
  `id_rsa`, `*.p12` — the standard accidental-credential vectors.

## F-13/F-14/F-15 — Low

* `sandbox::execve_path` resolves `argv[0]` through `$PATH` (and appends `.exe`), so a writable
  first-PATH entry redirects `aiosh run <tool>`; the non-Linux branch then executes the
  *unresolved* `&argv[0]` while the Linux branch executes the resolved path — inconsistent.
* `aios.audit.rotate` opens the ring file a **second** time with `.unwrap()` (`main.rs:4110-4112`)
  while `self.ring` already holds it: panic-on-error plus a second writer (see F-02).
* `aios.backup.restore`'s comment block still argues with itself ("but wait, recorded_call also
  emits! … Actually, let's just call it directly"), and `parse_sandbox_applied` is unreachable from
  production. Security-relevant surfaces should not ship with unresolved deliberation in them.

---

## What is genuinely solid

* **Gate ordering is correct.** `dispatch()` implements classifier → PEP → exactly-one-row, and a
  classifier `refused` verdict beats a valid grant; refusals are written, not swallowed.
* **Path canonicalization** (`pep::canonical_path_key`, incl. the T-01537 S-22 device/UNC prefix
  work landed in `55b43b2`) is careful and fails closed: unmappable device spellings return `None`
  and are denied rather than left unmatched; `..`/`.` and separators are normalized before
  comparison. Independently re-verified by negative control — see Corrections.
* **No shell anywhere.** Every subprocess is an argv vector (`Command::new(...).args(...)`,
  `subprocess.Popen(list)`, `os.execv`); zero `shell=True`, `os.system`, `eval`/`exec`, `pickle`,
  `yaml.load` in shipped code.
* **`unsafe` is confined** to `libc` syscall wrappers (seccomp/Landlock/prctl/fork/waitpid), one
  `sqlite3_randomness` FFI call, and `flock`; no pointer arithmetic or unchecked slice work.
* **Zip/backup handling is hardened**: `enclosed_name()` blocks zip-slip, 100k-entry and 10 GB caps,
  non-empty target dir refused.
* **No secrets in the repository.** High-signal patterns (AWS keys, GitHub tokens, `sk-`, Slack,
  PEM private keys) across **all 59 reachable revisions** match only `secrets_service.rs`'s
  deliberate test fixtures. Pattern-based only — no entropy scanner was available.
* **README/deployment hygiene**: audit DBs, `*.gguf`, build outputs are gitignored; `SECURITY.md`
  and per-epic `*-security.md` evidence exist.

---

## Corrections made during the delivery pass

1. **C8 test data was wrong, and the comparator fix is load-bearing.** The C8 contract case carried
   one backslash too few in its two device-namespace spellings, so `\.\PhysicalDrive0` (2
   backslashes) was tested instead of `\\.\PhysicalDrive0` (3). A 2-backslash spelling is not a
   device path at all — it normalizes to `/PhysicalDrive0`, which no deny entry covers — so those
   two cases passed for the wrong reason and C8 as committed **failed** (on a stale duplicate-layout
   error from the shared store). The spellings were corrected and are now asserted
   (`startswith(two-backslashes) and count >= 3`) so the data cannot rot again, and the
   `_long_path` docstring was made raw to silence a `SyntaxWarning`.
   **Verification:** with the comparator fix in place, `test_fs_layout_mcp_contract.py` passes
   C1–C8 against the real binary; with `pep.rs` reverted to `84c34ac` and rebuilt, C8 fails at the
   first device spelling (`extended-length: expected the PEP gate`) while C7 still passes — so the
   test exercises the fix and the fix holds. File restored byte-identical afterwards (sha256
   compared).
2. **F-06 reclassified twice, and now stated precisely.** The first draft of this report presented
   F-06 as source-verified. When the delivery pass reproduced an `exit 101` panic, the first panic
   found was a *different* one (`main.rs:32`, the ring cannot be opened while busy), so F-06 was
   marked "source only — not reproduced" and the startup panic was recorded separately as F-16. A
   further burst then caught `dispatch.rs:214` — the `.expect("audit row write failed")` line F-06 is
   about — so F-06's *panic* is now live-confirmed while its *consequence* (mutation performed, row
   lost) remains inferred, because the attempt to observe that consequence directly did not
   reproduce. Both states are now labelled in the summary table.
3. **The git-history secret claim is now earned.** The audit asserted the history was clean on the
   strength of a working-tree `git grep`; it has since been run across all 59 reachable revisions.
   The claim survives, and is now scoped as pattern-based.
4. **"Every type of audit" is now a ledger, not an implication.** The coverage table above names the
   audit types performed and the seven that were not, with reasons. Several were unavailable rather
   than skipped: no advisory DB, no SAST/secret-scanning tooling, no Linux host, nothing to scan for
   container/IaC/licence, and no adversarial LLM session.

---

## Appendix — reproducing the findings

Everything below ran against `code/aiosh-rust/target/debug/aiosh-mcp.exe` built from `55b43b2`,
with `AIOSH_HOME` and the child's `cwd` pointed at a fresh temp directory. No arguments were altered
in the transcript except truncation of long strings.

```python
import json, os, subprocess, tempfile, concurrent.futures as cf
BIN = os.path.abspath("aiosh-mcp.exe")
td = tempfile.mkdtemp(prefix="aios-audit-")
env = dict(os.environ, AIOSH_HOME=os.path.join(td, ".aios"), HOME=td)
os.makedirs(env["AIOSH_HOME"], exist_ok=True)

def call(tool, arguments):
    """One JSON-RPC tools/call over stdio; returns (result, exit code, stderr)."""
    p = subprocess.Popen([BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, text=True, cwd=td, env=env)
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": tool, "arguments": arguments}}
    out, err = p.communicate(json.dumps(req) + "\n", timeout=120)
    lines = out.strip().splitlines()
    # An instance that panics at startup (F-16) writes nothing to stdout. Report that
    # instead of letting the harness die on the missing response.
    result = (json.loads(lines[-1])["result"]["structuredContent"]["result"]
              if lines else {"gate": None, "ok": False, "error": "no stdout: server died before replying"})
    return result, p.returncode, err

# F-01/F-03: ungranted mutations pass the gate (no "gate" key) while controls are refused
for label, tool, args in [
    ("control", "aios.fs.read", {"path": os.path.join(td, "x.txt")}),
    ("F-03", "aios.service.action", {"name": "ssh.service", "action": "stop",
                                     "store_path": os.path.join(td, "state", "svc.json")}),
    ("F-01", "aios.handoff.initiate", {"sender": "a", "receiver": "b", "summary": "s"}),
    ("F-01", "aios.handoff.initiate", {"sender": "a", "receiver": "b", "summary": "s",
                                       "store_path": os.path.join(env["AIOSH_HOME"], "audit.db")}),
]:
    r, rc, _ = call(tool, args)
    print(label, "gate=%r" % r.get("gate"), "ok=%r" % r.get("ok"), str(r.get("error") or "")[:60])

# F-04: the same forged token, accepted by one surface and rejected by another
for tool, args in [
    ("aios.backup.restore", {"backup_path": os.path.join(td, "no.zip"),
                             "target_dir": os.path.join(td, "r"), "grant_id": "x"}),
    ("aios.audit.rotate", {"keep_rows": 0, "grant_id": "x"}),
]:
    r, rc, _ = call(tool, args)
    print(tool, "gate=%r" % r.get("gate"), "ok=%r" % r.get("ok"), str(r.get("reason") or r.get("error") or "")[:60])

# F-02/F-16: concurrent writers fork the chain, and a busy ring panics at startup
with cf.ThreadPoolExecutor(24) as ex:
    results = list(ex.map(lambda _: call("aios.audit.tail", {"n": 1}), range(24)))
print("nonzero exits:", [(rc, err.strip().splitlines()[0][:120]) for _, rc, err in results if rc != 0])
v, _, _ = call("aios.audit.verify", {})
print("verify: ok=%r checked=%r broken_at=%r" % (v.get("ok"), v.get("checked"), v.get("broken_at")))
```

For F-05 (Landlock) there is no reproduction here by design: it needs a Linux host with
`CONFIG_SECURITY_LANDLOCK=y`. The experiment to run is `strace -f -e trace=landlock_add_rule aiosh run true`
and to read the errno — `EBADF` confirms that `parent_fd` was rejected and `restrict_self` never ran.
