# AIOS_MERGED — Full-Source Security Audit

**Date:** 2026-09-20
**Scope:** every source file in the repo (296 non-doc source files: 116 Rust, 128 Python, 16 TypeScript src, 7 TOML, 2 YAML, 4 JSONL/JSON config, 3 shell)
**Method:** line-by-line review of the security boundary (classifier, PEP, dispatch, audit chain, sandbox, secrets scanner, file/path handling, process execution, archive/restore), plus exhaustive pattern sweeps (panics, byte-slicing, path joins, process spawn, SQL, deserialization, secrets, permissions) over every remaining file.
**Changes made:** none. This is a read-only audit; the only file written is this report.

---

## Executive summary

The platform's *architecture* is defensible: classifier → PEP grant → single audit row, hash-chained audit ring with segments/archives, bounded file reads, exclusive temp + fsync + rename for stores, zip-slip guards, and a real attempt at cross-substrate hash parity. The problems are at the **seams and the gates**:

1. **The authorization gate is mostly bypassed in practice.** 90 of 98 MCP tool call sites pass `require_grant=false`, and `PepStore::is_irreversible()` only recognises a hand-written allowlist of tool-name prefixes. Everything else — handoff, service, package, kernel_module, session/triage mutations, secrets scan — runs **with no grant at all**.
2. **The sandbox does nothing except a syscall blacklist.** Both implementations (Rust `sandbox.rs`, Python `sandbox.py`) pass `AT_FDCWD` as Landlock's `parent_fd`, so no path rule is ever installed. Every `aiosh run` / `process.run` therefore has **full filesystem access**, while the audit row still records `sandbox_applied`.
3. **`aiosh run` requires no grant.** `process.run` is neither irreversible nor refused (0.85 = "caution"), so arbitrary command execution is unauthenticated.
4. **Grant checks in module-level helpers accept any non-empty string** (`check_doc_index_policy`, `check_evidence_policy`, `check_toolchain_policy`, `check_release_policy`) — a forged grant id passes.
5. **Cross-substrate hash parity is broken for non-ASCII** (Python `json.dumps` defaults to `ensure_ascii=True`; Rust emits raw UTF-8); the audit chain silently fails to verify across surfaces.
6. **Six remotely-reachable panics** on multibyte input (byte-slicing / `String::truncate`) — trivial DoS of the CLI/MCP process.

---

## 1. CRITICAL

### C-1 — Landlock is never applied; the sandbox is a no-op file-ACL claim
* `code/aiosh-rust/aiosh-core/src/sandbox.rs:344-352` — `landlock_add_rule` is called with `parent_fd: AT_FDCWD` (-100). Landlock requires an **O_PATH fd** to the directory; the string path in `rule.path` is never used (dead `c_path`, `let _ = c_path;` at :372 confirms the intent). The kernel rejects the rule; the code then **skips `landlock_restrict_self` entirely** (`if ok { … }`, :368-375) and returns `FAIL`.
* `code/aiosh-mcp/aiosh_mcp/sandbox.py:337-340` — identical bug (`struct.pack("<Qi", bits, AT_FDCWD)`), in the Python executor that the TS CLI falls back to.
* `sandbox.rs:432-450` — the "unsupported → warn + continue" policy means the *child still execs* with no path restriction. `emit_sandbox_applied` (:466) then writes `{"event":"sandbox_applied", …}` so callers (TS `parseSandboxApplied`) treat the run as sandboxed even though the component line is `"FAIL: …"`.
**Impact:** `aiosh run`, `aiosh-sandbox`, and the Python shim provide **no filesystem isolation**; seccomp only blocks 17 syscalls. A command may read `~/.ssh/*`, write anywhere the user can, and exfiltrate.
**CWE-693/CWE-1188.** *Fix:* open each rule path with `O_PATH|O_CLOEXEC` and pass that fd; make any FAIL fatal unless `--allow-degraded` is explicit; never emit `sandbox_applied` when a component failed.

### C-2 — Unauthenticated arbitrary command execution (`process.run` / `aiosh run`)
* `code/aiosh-cli/src/pep.ts:65-71` — `isIrreversible()` = `fs.write*`, `pentest.*`, reboot, shutdown. `process.run` is absent. `code/aiosh-rust/aiosh-core/src/pep.rs:377-440` also omits it.
* `code/aiosh-rust/aiosh-core/src/classifier.rs:145-160` — R-05a/R-05b carry 0.85 confidence → verdict **"caution"**, not "refused" (locked in by the test at :594 `assert_eq!(r.overall_verdict, "caution")` for `rm -rf /`).
* `code/aiosh-cli/src/cli.ts:196-262` — `aiosh run` never checks a grant, never enforces `--yes`, and executes via `execFile` (no shell injection, good) with the broken sandbox above.
**Impact:** any caller (AI agent, MCP client, local user) gets RCE with the invoking user's privileges; the only filter is a 5-phrase string blacklist (`PROMPT_INJECTION_FRAGMENTS`) and 13 destructive-binary names.
**CWE-78/CWE-862.** *Fix:* treat `process.run` as irreversible (grant required) and refuse rather than caution on `DANGEROUS_BINS` + `DANGEROUS_ARG_FRAGMENTS`.

### C-3 — Authorization gap: 90/98 MCP tools run with no grant
* `code/aiosh-rust/aiosh-mcp/src/main.rs` — 98 `dispatch::recorded_call*` sites; only 8 pass `true` (`aios.session.action/create`, the four `aios.fs_layout.*` mutations, `aios.fs.read`, `aios.task`).
* `code/aiosh-rust/aiosh-core/src/pep.rs:377-440` — `is_irreversible()` covers `pentest.*`, `fs.write`, `aios.backup.*`, `aios.release.*`, `toolchain.set`, `doc.set`, `evidence.record/set`, `session.action/create`, the four `fs_layout` mutations, `system.reboot/shutdown`. Everything else is "reversible".
* Concretely ungated state mutations reachable from MCP: `aios.handoff.initiate/accept/reject/complete/cancel` (writes the handoff store; `main.rs:1441-1530`), `aios.service.action/order/config/policy`, `aios.package.apply/config/policy`, `aios.kernel_module.blacklist/unblacklist/autoload/unautoload/preset.apply`, `aios.triage.record/resolve`, `aios.session.config/policy`, `aios.distro.policy`, `aios.image.policy/config`.
**Mitigating nuance:** `service_service::execute_action` (:291) and the package/kernel_module stores are **in-memory simulations** (fake PIDs 1001/1002, no `systemctl`/`modprobe` spawn) — so today the blast radius is persistent JSON store state and audit-noise, not host mutation. That is a "not yet wired to the host" accident, not a control: the moment these are wired to real units, this becomes full system control.
**CWE-862.** *Fix:* invert the default — require a grant for every tool whose name is not in an explicit read-only allowlist.

### C-4 — Grant checks that accept any non-empty string
* `code/aiosh-rust/aiosh-core/src/release.rs:119-124` — `check_release_policy(Some("gr_xyz"), …)` → `Ok(())` (their own test at :135 asserts this).
* `doc_index_service.rs:266-276`, `evidence_service.rs:131-141`, `toolchain_service.rs:271-283` — same "presence" test.
**Impact:** a caller that relies on these helpers (they exist precisely to be the policy hook) accepts a fabricated/stale/expired grant id. No scope, expiry, revocation, or tool check is performed.
**CWE-287/CWE-863.** *Fix:* delete these helpers or make them delegate to `PepStore::check_with_paths`.

### C-5 — Audit-chain hash parity is broken for non-ASCII content
* `code/aiosh-mcp/aiosh_mcp/audit_client.py:34-39` — `json.dumps(obj, sort_keys=True, separators=(",",":"))` uses the **default `ensure_ascii=True`**, escaping non-ASCII as `\uXXXX`.
* `code/aiosh-rust/aiosh-core/src/canonical.rs:22-34` — `escape_json_string` emits raw UTF-8 (only `"`/`\`/control chars are escaped).
* The module docstrings in both files claim byte-identical output; `tests/test_smoke.py` only exercises ASCII.
**Impact:** any audit row containing a non-ASCII path, note, or model output hashes differently in each substrate. Mixed-substrate chains (TS CLI + Rust CLI + Python MCP — which is the documented design) will report `ok:false, broken_at:<row>` on a **pristine** ring. Forensics/retention archives inherit the same mismatch. Also `audit.rs:556` returns `"v0.0"` silently when the constitution file is unreadable (fail-open provenance).
**CWE-345.** *Fix:* `ensure_ascii=False` in Python (and a migration note), plus a cross-substrate golden-vector test with emoji/CJK/combining marks.

---

## 2. HIGH

### H-1 — Expectation-only grant checks in the Python gate + weaker irreversible list
`audit_client.py:520-580` `grant_check()` implements a *second* gate whose irreversible set is `pentest.*`, `fs.write`, reboot/shutdown only — no `aios.fs_layout.*`, `aios.backup.*`, `aios.release.*`, `aios.session.*`. The Python MCP server (`server.py:584`) therefore authorizes things the Rust surface refuses. Same class as C-3/C-4 and a direct cross-substrate policy divergence. **CWE-863.**

### H-2 — Path-scope checks in TS and Python are alias-evadable (deny bypass)
* `code/aiosh-cli/src/pep.ts:44-63` and `code/aiosh-mcp/aiosh_mcp/audit_client.py:365-395` do raw `startsWith` comparison with zero canonicalization — exactly the class the Rust port fixed (T-01537 S-18/S-19/S-22, see `pep.rs:150-330`).
* Bypasses on Windows (the dev platform): case variants (`C:\Secret` vs `c:\secret\x`), 8.3 short names (`C:\PROGRA~1`), trailing dot/space (`C:\Secret.\x`), device prefixes (`\\?\C:\Secret\x`), junctions. On macOS: NFD/NFC normalization differences.
**Impact:** a grant scoped `paths.deny: ["C:\\Secret"]` can still read/write that directory through any alias. **CWE-22/CWE-178.** *Fix:* port `canonical_path_key`/`canonical_store_key` semantics into `pep.ts` and `audit_client.py`, or route path decisions to the Rust core.

### H-3 — `restore_backup` zip-bomb bound is per-file, not cumulative
`code/aiosh-rust/aiosh-core/src/release.rs:333-364` — the accumulated check uses the archive's **declared** sizes (`file.size()`), then each entry is copied with `take(&mut file, MAX_UNCOMPRESSED_SIZE)` = 10 GB **per file**. A crafted zip whose central directory under-declares sizes passes the accumulator and can still write up to 10 GB × 100 000 entries. `validate_backup` (:280) also never checks the declared hash. **CWE-409.** *Fix:* `take(remaining_budget)`; verify `expected_hash` against the file in `validate_release` (:249 — currently uses it only for a length check, making "release validation" a no-op integrity claim).

### H-4 — Attacker-controlled release artifact path → traversal + whole-CWD packaging
`release.rs:26-36` builds `output/release/aios_{target_os}_{version}.iso` from unvalidated manifest fields and passes it to `genisoimage -o <path> .` (:172). `target_os="../../../../home/user/x"` writes outside the tree; the `.` source means the ISO contains **everything in the process CWD** (including `.env`, keys, the audit DB). **CWE-22/CWE-668.** *Fix:* allowlist `target_os`/`version` (`[A-Za-z0-9._-]{1,32}`), package an explicit staging dir.

### H-5 — Unauthenticated handoff state tampering (identity is caller-supplied)
`code/aiosh-rust/aiosh-core/src/handoff.rs:110-140` — `can_agent_act` grants universal access to the literal strings `operator`/`admin`/`root`, and otherwise compares the caller's `actor_id` to the record's `sender`/`receiver`. Both come from the same request; nothing authenticates them. `signature`/`id` are a plain content hash with no key, so "tamper-evident" is nominal. Combined with C-3 (no grant), any client can accept/cancel/complete any handoff by claiming the receiver identity. **CWE-287/CWE-345.**

### H-6 — Reachable panics on multibyte input (DoS of CLI/MCP)
Byte-slicing/`truncate` without char-boundary checks:
| Location | Code |
|---|---|
| `aiosh-core/src/handoff.rs:84,87` | `summary.truncate(4096)`, `payload.truncate(65536)` |
| `aiosh-core/src/triage.rs:71,76,81` | `test_target/error_message/repro_command.truncate(…)` |
| `aiosh-core/src/toolchain_service.rs:298` | `&trimmed[..max_len]` |
| `aiosh-core/src/secrets_service.rs:70,94,115` | `&line[..4096]`, `&scan_text[pos..pos+20]`, `[pos..pos+40]` |
| `aiosh-core/src/release.rs:153` | `&stderr[..4096]` |
A single 4096-byte multibyte line in a file (or a CJK/emoji arg over the cap) aborts the process. For `secrets_service` this is remotely triggerable by content the scanner is pointed at (any untrusted repo). **CWE-20/CWE-248.** *Fix:* `is_char_boundary`-safe truncation helpers (`floor_char_boundary` / iterate chars).

### H-7 — Secrets-scanner symlink recursion + partial secret disclosure
`secrets_service.rs:170-200` — `walk_dir` follows symlinks with no visited-inode set and no depth cap: a symlink loop (or a link to `/`) causes unbounded recursion → stack overflow; a link out of the root silently widens the scan scope. `secrets.rs:120-130` `redact_secret_value` keeps the first/last 4 characters of every secret in the report, i.e. the evidence artifacts and MCP responses leak 8 characters of live credentials (for a 20-char AWS key that is 40% of the secret). `secrets_config.rs` `allow_patterns`, `max_line_bytes`, `require_clean` are validated but **never used** (dead policy knobs giving a false sense of configurability). **CWE-674/CWE-532/CWE-1188.**

### H-8 — Audit durability and anchoring weaknesses
* `audit.rs:246-260` `write()` reads the head then inserts, with **no transaction and no lock**: two processes (documented design: TS + Rust + Python all write the same DB) can chain off the same `prev_hash` → fork → verification permanently fails. `UNIQUE(hash)` catches only byte-identical rows.
* `audit.rs:418` `delete_rows_le` / `retention.rs` rotation: there is **no external anchor**, so an attacker with DB access can truncate the tail and still verify "clean" (documented design limit — worth stating in SECURITY.md as a known limitation).
* `audit.rs:80-90` default DB path falls back to `/tmp/.aios/audit.db` when `HOME` is unset (world-writable dir), and the DB file is created with default umask (0644) while it stores **full grant tokens** (`grant_token` column) — archive files are 0600 (`retention.rs:432`) but the live DB is not. **CWE-276/CWE-345.** *Fix:* sqlite transaction around head+insert, `chmod 0600`, refuse to run with an unset/unsafe HOME.

### H-9 — `handoff`/`triage` records are not validated on load
`HandoffStore::load_or_recover` (used by every `aios.handoff.*`) parses records from a caller-named `store_path` (`main.rs:1418` default `.aios/handoff_store.json`); `validate_handoff_record` exists but is only enforced on write paths in tests. A crafted store file injects records with any sender/receiver/status — and the store path is a **free-form MCP argument** with no path-scope check when no grant is supplied (C-3), i.e. arbitrary file write (as JSON) to any path the user can write. **CWE-73.**

---

## 3. MEDIUM

* **M-1 `tool_glob_match` over-matches** — `pep.rs:18-32` strips `.*` then uses `starts_with`, so `pentest.*` also matches `pentestfoo`. Use `tool == prefix || tool.starts_with(prefix + ".")`. Same in `pep.ts:36` and `audit_client.py:346`.
* **M-2 `max_irreversible` is never enforced** — declared, serialized, hashed (`types.rs:170`), and checked nowhere. Dead policy field.
* **M-3 R-10 is dead** — `target_out_of_grant_scope` (`classifier.rs:205-215`) is never set by any caller in any substrate, so the "cross-ref" C-1 rule can never fire.
* **M-4 The prompt-injection control is a 5-phrase English blacklist** (`classifier.rs:24-28`), matched only against *argument* values (targets are exempt). `"ignore  constitution"` (double space), `"ignore-constitution"`, homoglyphs, or any other language evades it. Report it as defence-in-depth only.
* **M-5 Windows ledger lock is a lie** — `ledger.rs:660-670` `acquire_lock_timeout`'s `#[cfg(not(unix))]` branch returns a `FileLock` without locking anything, while the module doc claims "advisory flock guards against concurrent runs". On Windows (dev platform) concurrent `complete_task` runs lose updates / duplicate `seq`. Same for `append_event`'s read-then-append seq assignment.
* **M-6 `append_event`/`save_state_atomic` modes** — state/lock/events are created 0644; `cleanup_stale_tmp` (`ledger.rs:203`) deletes any `<state>.tmp.*` in the directory (predictable-name deletion in a possibly shared dir). Low practical impact; worth tightening.
* **M-7 `emit`/`commit` ordering** — `dispatch.rs:246` runs the tool body **before** `commit()`, which uses `.expect("audit row write failed")`: if the DB write fails, the side effect already happened and the process then panics. No compensating action. *Fix:* write the intent row first, or refuse to run with an unwritable ring.
* **M-8 `aiosh run` executor path is env-overridable** — `cli.ts:296` `AIOSH_SANDBOX_BIN`; `ai_agent.py:124-135` searches relative paths (`./code/aiosh-rust/target/debug/aiosh-mcp`) before installing absolutes — a checked-out repo can plant a fake MCP server / sandbox binary that runs with the user's privileges (CWE-427). `pentest.rs:80` likewise spawns `argv[0]` via PATH even though `host_has` already resolved an absolute path (`bin_path` is only reported, never used) → PATH-hijack of nmap/nikto/sqlmap when the grant is scoped only to `tool`/`target`.
* **M-9 `canonical()` escape gaps** — U+2028/2029 and DEL are emitted raw (valid JSON, but `</script>`-style hosts and some JSON consumers reject them); serde_json float formatting (`1e100`) vs Python `repr` (`1e+100`) diverges for exponent forms; Python `json.dumps` can emit `NaN`/`Infinity` (invalid JSON) that Rust cannot parse back.
* **M-10 `AuditRing::open` ignores failures** — `audit.rs:163` `let _ = create_dir_all(parent)`; a failed mkdir is swallowed before `Connection::open` reports a less precise error. `active_constitution_rev` failing open to `"v0.0"` is noted in C-5.
* **M-11 `unchecked_transaction` in rotation** — `retention.rs:470` begins a transaction on a shared `&Connection`; with another transaction already open this degrades to savepoint semantics, weakening the "all or none" claim for the segment row + delete + rotation row.
* **M-12 Non-network-target grants skip scope entirely** — `pep.rs:520` and `audit_client.py:560`: when `scope.networks` is empty, a `pentest.*` grant authorizes **any** address (the classifier is then the only net). Worth making empty `networks` + pentest an explicit, documented choice.
* **M-13 `validate_doc_links` / doc-index reads escape the root** — `doc_index_service.rs:198-260` `repo_root.join(rel_path)` accepts absolute/`..` caller paths (read-only, but an arbitrary-file read primitive that feeds `aios.doc.search` results).
* **M-14 Duplicate/divergent canonicalization** — `canonical_path_key` (anchored) vs `canonical_store_key` (unanchored) vs TS/Python `startsWith` vs `folder Windows aliases`: four implementations of "is this path the same path". Consolidate or expect the next bypass.

---

## 4. LOW / INFORMATIONAL

* `correlation` of `stdout_truncated`: `pentest.rs:158-160` compares **byte** length against a **char** cap → wrong `*_truncated` flags for multibyte output.
* `sandbox.rs:520-530` `fork()` inside a Rust process that may have threads — child runs allocator/stderr code after fork (deadlock risk). Use `posix_spawn`/direct exec.
* `sandbox.rs:26-30` syscall numbers/arch are x86_64-only; the BPF arch check *kills* every syscall on aarch64 (SIGSYS at startup) rather than refusing cleanly.
* 64-bit grant ids (`pep.rs:640`, `pep.ts:107`, `api_client` equivalent) for a bearer credential; no per-grant secret, no rate limit, stored in cleartext in the audit DB.
* `restore_backup` writes an audit row only on success; refusals (zip bomb, non-empty target) are unlogged.
* `release.rs:70,130` write `outcome: "success"` instead of the contract's `ok`/`refused`/`error`, and set all C-flags false — these rows will read as anomalies/are invisible to `ok`-filtering tooling.
* `generate_release`/`create_backup`'s `physical_create_zip` writes a **dummy manifest only** (`release.rs:195-215`): a "successful" backup contains no data while the audit row claims success — data-loss deception, worth a functional bug report.
* `recovery.rs` modules rename the corrupt store aside without an audit row (`distro_recovery.rs:101`).
* `tools/ci_run.py` + `ci_config.py` default results/logs to `/tmp/aiosh-ci-*.log` (predictable world-readable paths; symlink pre-planting).
* `docs` versus code: `SECURITY.md`/kernel-module docs claim controls (Landlock isolation, grant-gated mutations) that this audit shows are not enforced — the documentation overstates the security posture (in scope for a bug-bounty report as "security-relevant documentation defect").

## 5. Scan results that came back clean
* No hardcoded credentials anywhere in source/config (`api_key|secret|password|token = "…"` sweep over `.rs/.py/.ts/.js/.toml/.json/.yaml` — zero hits).
* No `shell=True`, `os.system`, `eval`/`exec`, `pickle`, or `yaml.load` in shipped source (only `sys.executable -c` in tests).
* No `unsafe` outside `sandbox.rs`, `ledger.rs` (flock), `pep.rs` (sqlite3_randomness), all of which were reviewed.
* No string-interpolated SQL with user input (the two `execute(f"…")` sites use compile-time constants).
* `aiosh-cli/src/*.ts` uses `execFile` with an argv array — no command-string interpolation.
* Rust `fs_layout` validation is genuinely strong (control-char/whitespace refusal on device/options, path hygiene, `deny_unknown_fields`, bounded reads, exclusive temp + fsync + rename + residue cap).
