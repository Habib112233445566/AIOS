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

---

# SECOND PASS — 2026-09-20 (deeper sweep of the seams)

This pass re-read the entry points (Rust CLI, Rust MCP, Python MCP, TS CLI), the previously only
pattern-screened core modules, and proved/disproved the suspicions left open in pass 1.

## New: CRITICAL

### C-6 — `aios.backup.restore` has no gate at all (forged grant accepted)
`code/aiosh-rust/aiosh-mcp/src/main.rs:4179-4194` — the handler calls
`aiosh_core::release::restore_backup(&mut ReleaseCtx{…}, backup_path, target_dir, grant_id)`
**directly**: no `dispatch::recorded_call`, no classifier gate, no `PepStore` look-up.
The only check is `check_release_policy(grant, "aios.backup.restore")` (`release.rs:119-124`),
which tests `grant.is_none()` → **any non-empty string authorizes** (their own unit test asserts
`check_release_policy(Some("gr_xyz"), …).is_ok()`).
Consequences: an unauthenticated MCP caller can extract an attacker-supplied ZIP into any *empty*
directory (bounded only by H-3's broken zip-bomb bound), and the row the routine writes itself
carries `c_flags` all-false and `constitution_rev: "v0.0"` — violating the C-4 "audit always"
contract and carrying no classifier provenance (`policy_revision: None`).
The read-only siblings `aios.release.validate` / `aios.backup.validate` *do* go through
`recorded_call` (→ classifier refusal works there) but with `require_grant=false` and a
caller-named path, giving an unauthenticated existence/size/ZIP-structure oracle.
**CWE-862 + CWE-287.** *Fix:* route through `dispatch::recorded_call`; make
`check_release_policy` delegate to `PepStore::check_with_paths`.

### C-7 — The Rust CLI has no enforcement point; `emit()` accepts forged grant provenance
`grep -n 'pep\.check|check_with_paths|is_irreversible|require_grant' code/aiosh-rust/aiosh-cli/src/main.rs`
→ **zero matches.** Every subcommand (`run`, `mod`, `service`, `session`, `layout`, `package`,
`handoff`, `triage`, `release`, `backup`, `doc`, `evidence`, `distro`, `image`, `toolchain`)
reaches `classify_and_emit()` (`main.rs:122-141`), which classifies and writes a row but never
authorizes anything; `emit()` (`main.rs:87-119`) copies the caller-supplied `grant_token` straight
into the row's `grant_token` column with no validation. So:
* `aiosh run <cmd>` = unauthenticated host command execution (with the C-1 dead sandbox);
* every CLI mutation (`aiosh mod blacklist`, `aiosh service action`, `aiosh layout remove`, …)
is unauthenticated;
* the audit ring can be made to *look* as though a grant authorized an action that never had one
— an audit-integrity defect, not just an authorization one (CWE-862 + CWE-345).
*Fix:* one `pep.check_with_paths` call in `classify_and_emit` (refusing on `Err` before the side
effect), and record `grant_token` only after it validates.

## New: HIGH

### H-10 — Python MCP exposes ungated release/backup writers (exfiltration + arbitrary file write)
`code/aiosh-mcp/aiosh_mcp/release.py:184-215, 246-273` — both tools call
`dispatch_mod.dispatch(tool=…, grant_id=grant_id)` **without `require_grant=True`**, and
`audit_client.grant_check`'s irreversible set is only `pentest.*` / `fs.write` / reboot / shutdown,
so both are unauthenticated despite the docstrings saying "Requires PEP grant"
(and the Rust surface requires one). Impact:
* `aios.backup.create` → `physical_create_zip` walks **any caller-named directory**
(`os.walk(snapshot.target_path)`) and writes `aios_backup_<ts>.zip` into the process CWD
— an arbitrary-directory exfiltration primitive (`target_path: "/home/user"`).
* `aios.release.generate` → `artifact_path = f"{output_dir}/aios_{target_os}_{version}.iso"`
is caller-controlled text with `..`/`/` support, then `open(path, "wb")` truncates/creates it
→ arbitrary file creation/truncation (content `AIOS_ISO_MOCK`).
**CWE-862 + CWE-22 + CWE-668.**

### H-11 — Evidence verification trusts the manifest it is handed
`code/aiosh-rust/aiosh-core/src/evidence_service.rs:85-129` — `verify_evidence_manifest` validates
the manifest's *shape* and then compares each file against **the hash stored in that same
caller-supplied manifest**, with no anchor in the audit ring or any signed record. Regenerating the
manifest over tampered evidence makes `aios.evidence.verify` report PASS. Additionally
`repo_root.join(&record.file_path)` takes `file_path` from caller JSON, so a `..`/absolute entry
turns the verifier into an arbitrary-path existence + hash oracle. **CWE-345.**
*Fix:* pin manifests to the audit chain (record the manifest's own sha256 in a row, or require
`constitution_rev`-scoped signing) and reject paths that escape `repo_root`.

### H-12 — Python classifier still has the nested-argument injection blind spot
`code/aiosh-mcp/aiosh_mcp/classifier.py:282-305` `_scan_arg_text_for_pi` walks only *top-level*
strings and list elements, whereas the Rust implementation
(`classifier.rs:330-375 scan_value_for_pi`, documented as T-01537 S-2) recurses into nested objects.
The S-2 fix never reached the Python MCP server, so a payload nested one level down
(`{"layout": {"name": "ignore constitution"}}`) is refused by the Rust surface and **invisible to
C-3 in Python** — the substrate most people actually run. **CWE-693.**

## New: MEDIUM

* **M-15 — "Validated" is a library property, not a service property.** 30+ `validate_*` functions
have **zero production call sites** (only their own module + tests): `validate_handoff_record/report`,
`validate_triage_record/report`, `validate_kernel_module_store`, `validate_service_status`,
`validate_user_session_status`, `validate_package_transaction`, `validate_distro_profile`,
`validate_base_image_manifest`, `validate_repo_health_report`, `validate_manifest`, `validate_config`,
`validate_doc_links`, `validate_store_health`, `validate_hex_id`, `validate_path`, `validate_device_id`.
The CLI/MCP handlers deserialize stores with serde and act on them without invoking the validator,
so any invariant the model enforces is unenforced in production (this is the systemic root cause
behind H-9 and much of C-3/C-4).
* **M-16 — `deny_unknown_fields` on only 7 of 47 config/policy/store structs.** A misspelled policy
key is silently ignored — the same failure mode the fs_layout store fix (T-01544) called out
("a store spelled `active_layout` loaded successfully and silently kept the default") remains true
for the other ~40 documents, including grant-adjacent policy documents.
* **M-17 — The audit verifier panics on a tampered segment row instead of reporting tampering.**
`retention.rs:947-970` — `hex_to_bytes` silently truncates/skips invalid nibbles, while `bloom_test`
(`retention.rs:88-95`) indexes `bits[idx >> 3]` using `bloom_m_bits`/`bloom_k` **read from the same
row**; a short `bloom_hex` or an inflated/negative `bloom_m_bits` (`as usize`) panics
`aios.audit.seen` / `verify_full` — i.e. the tool used to detect DB tampering is crashed by DB
tampering (CWE-248). `seen(exact=true)` also reads `archive_path` from the row and opens it, so a
edited row makes the server touch arbitrary files.
* **M-18 — Rust port regressions vs the Python reference (both security-relevant).**
(i) `pentest.rs:80` spawns `argv[0]` through PATH although `host_has()` already resolved an absolute
`bin_path` — the Python wrapper uses the resolved path (`pentest.py:118`), so the Rust port
re-introduced the PATH-hijack that the reference avoids.
(ii) Rust builds the audit `command` string **unquoted** (`format!("nmap {}", target)`,
`pentest.rs:300`), while Python uses `shlex.quote` — a target containing `\n` or control characters
injects lines into the audit row's `command` column and into any terminal/JSON rendering of it.
(iii) Python `pentest_nmap(mode=…)` accepts `mode` (docstring advertises `"syn"`) and ignores it.
* **M-19 — Argument injection into the pentest binaries.** `target`/`url`/`pcap_path`/`wordlist`/
`interface` are appended as raw argv elements (`pentest.rs:290-430`), so a value beginning with `-`
is parsed as a *flag* by nmap/nikto/sqlmap/tshark/gobuster/airmon-ng (e.g. an nmap target of
`-oX/tmp/out` writes a file; `-iL` reads a target list from a file). The grant constrains only
`scope.networks`, which is skipped entirely when empty (M-12), and the classifier never sees a
leading-dash target. *Fix:* reject `-`-prefixed targets, and put `--` before the positional target
where the tool supports it (CWE-88).
* **M-20 — `--yes` is decorative.** `cli.ts:196` documents `--yes` as the C-3 acknowledgement and
neither the TS nor the Rust `run` path reads it.

## Verified clean this pass (suspicions disproved)

* **`aios.audit.rotate` is properly gated in all three substrates** — Rust `main.rs:4296-4303`
(`require_grant=true`, tool `audit.rotate`) and Python `server.py:297-300` (`require_grant=True`).
Retention cannot be used to truncate the live ring without a grant.
* **`aios.fs.read`'s safe-root check holds.** Rust canonicalizes *before* the prefix test
(`main.rs:4122-4133`); the `unwrap_or_else(|_| path.to_string())` fallback on a canonicalize failure
is not exploitable, because the subsequent `read_to_string` resolves the path the same way the
canonicalizer tried to (a `..` chain that canonicalize cannot resolve cannot be opened either).
Python's `Path.resolve()` + prefix test is likewise sound.
* **Python `retention.rotate` is the strongest implementation of the three**: `tempfile.mkstemp` in
the destination dir, `chmod 0600`, `os.replace`, `FileExistsError` before overwrite, and
`rollback()` + archive unlink on DB failure.
* **TS `canonicalJson` uses `JSON.stringify`**, which emits raw UTF-8 — so TypeScript and Rust
*agree* and **Python (`ensure_ascii=True`) is the sole outlier** in C-5. That narrows the fix to one
function (`audit_client.canonical`).

## Coverage note for this pass

Read closely: `aiosh-cli/src/main.rs` structure + `classify_and_emit`/`emit`/`cmd_run`/grant and
district handlers, `aiosh-mcp/src/main.rs` (fs.read, release/backup, audit.*, process.list,
handoff, fs_layout gate flags), `retention.rs` (+`seen`/hex/bloom), `handoff.rs`, `triage.rs`,
`release.rs`, `evidence_service.rs`, `secrets_service.rs`/`secrets_config.rs`, `sandbox.rs`,
`pep.rs`, `classifier.rs`, `dispatch.rs`, `audit.rs`, `canonical.rs`, `fs_layout*.rs`, `ledger.rs`,
Python `audit_client.py`/`sandbox.py`/`retention.py`/`release.py`/`pentest.py`/`classifier.py`/
`server.py`, `ai_agent.py`, TS `pep.ts`/`cli.ts`/`audit.ts`/`retention.ts` (plus pattern sweeps over
the whole tree). Still only pattern-screened (next candidates for a third pass): the pure data
models of `session*`, `distro*`, `base_image*`, `kernel_module*` (`_policy`, `_config`),
`package_policy`, `repo_health*`, `service_policy/config/recovery`, `ci*`, `doc_index*`,
`tools/*.py`, and `code/aiosh-cli/src/{agent,constitution,types}.ts`.

---

# THIRD PASS — 2026-09-20 (write paths, policy provenance, and the grant supply chain)

Method this pass: follow every caller-supplied string into a filesystem sink or an authorization
decision. Prior passes covered *reads* well; the **write** side and the *issuance* side of the grant
system had gaps. Five new findings, one of them root-level.

## New: HIGH

### N-1 — `store_path` is a caller-chosen, unvalidated write target (arbitrary file overwrite)

Fifty-seven call sites take `store_path` from the MCP arguments, and the store writers treat it as an
absolute destination:

* `aiosh-mcp/src/main.rs:1347-1349, 1416-1434, 3631, 3682, 3711, 3743, 3768` — default
`./.aios/handoff_store.json` / `./.aios/triage_store.json`, but any string is accepted. The only
validation that exists anywhere is a length/control-character check, and it is applied to
**9 of ~20** write-capable sites (`main.rs:2312, 2367, 2404, 2671, 2734, 2765, 2883, 2951, 3007`).
The handoff/triage/image/kernel-module tools have none at all.
* Sinks: `handoff_service.rs:230-247` (`create_dir_all(parent)` → `File::create(path.with_extension("tmp"))`
→ `fs::rename`), `triage_service.rs:134`, `base_image_service.rs:207-219`, `kernel_module_service.rs:144-181`,
`package_service.rs:470-497`, `distro_service.rs:97-107`.
* Every one of those tools passes `require_grant=false` (`aiosh-mcp/src/main.rs`, handoff/distro/image/
package/service/triage `recorded_call` sites), so no grant is involved.

**Impact.** Any MCP client can make the server create/overwrite a file at an arbitrary path with a
JSON body whose string fields the caller controls — e.g. `~/.ssh/config`, `~/.gitconfig`, a repo's
`.git/hooks/*`, another tool's config. `package_service.rs:485,496` and `base_image_service.rs:219`
force mode `0644` on both the temp file and the destination, so overwriting a previously `0600`
file **widens its permissions**. It is not code execution on its own (the content is JSON and the
rename does not follow a symlink), but it is an unauthenticated integrity/DoS primitive against the
operator's home directory, and it is the same primitive the store-tampering findings (H-9, M-15)
need to seed a hostile store. Reading is affected too: the same field loads JSON from any path.

**Fix.** One `resolve_store_path()` used by every tool: canonicalize, require the result to sit under
the AIOS home/workspace root, reject symlinked parents, never `create_dir_all` outside that root, and
write `0600` via the shared atomic-write helper (N-5).

### N-2 — The caller chooses its own enforcement level (`policy_path` + `mode: "audit"`)

Every policy type resolves as **file > env > default**, and the file comes straight from the request:

* `package_policy.rs:404-410`, `service_policy.rs:477-483`, `kernel_module_policy.rs:516-522`,
plus `session_config.rs:203`, `service_config.rs:245`, `package_config.rs:202`,
`base_image_config.rs` — `resolve(custom_path)` returns `from_file(path)` unconditionally.
* `from_file` only caps size (`MAX_POLICY_FILE_BYTES`) and calls `validate()`; **`validate()` never
constrains the mode field** (PP1 checks lengths/bounds only).
* Mode short-circuits the verdict: `package_policy.rs:284-288` `Audit => true`; same in
`service_policy.rs:391-395`; `session_policy.rs:327-331` has `Permissive => true` **unconditionally**;
`kernel_module_policy.rs:384-390, 433-437` keep only one or two rules alive in permissive mode.
* MCP exposure: `aios.package.policy`, `aios.package.config`, `aios.service.policy`,
`aios.session.policy`, `aios.kernel_module.config/policy` (`main.rs:1860, 2273, 2294, 2317, 2370, 2635`),
all `require_grant=false`.
* Env route: `from_source`/`from_env` accept `AIOS_PACKAGE_POLICY_MODE=audit`,
`AIOS_PACKAGE_REQUIRE_CHECKSUM=0`, `AIOS_PACKAGE_REQUIRE_HTTPS=0` (`package_policy.rs:369-395`) from
the server process environment.

**Impact.** A file containing `{"mode":"audit", ...}` makes every subsequent verdict
`allowed: true` while the report still shows a policy was evaluated (`mode` is echoed in the verdict).
This is the same defect class as C-4, but worse: it does not require forging a grant string, only
naming a path — and it silently converts a documented control into a logger. It also explains why the
earlier `check_*_policy` helpers looked so weak: policy strictness was never treated as a trust anchor.

**Fix.** A tool must never accept a policy weaker than its built-in floor. Either drop `custom_path`
from agent-facing tools (operators use the env/file contract at startup) or require the file to be
rooted under the AIOS config dir with an integrity hash, and force `mode >= Enforcing` at evaluation
time regardless of what the file says. Restrict env overrides to stricter-than-default values.

### N-3 — `kernel-module export` writes root-executed `modprobe.d` content with no policy check (root RCE)

`modprobe.d` is a root-controlled execution surface: `install <mod> <command>` is run **as root** by
`modprobe`. AIOS generates that syntax and writes it to an operator-chosen path, unaudited by policy:

* `kernel_module.rs:291-329` `to_modprobe_conf()` emits `install {module} {command}` and
`remove {module} {command}` **verbatim** — no escaping.
* `kernel_module.rs:247-267` `validate_config` (the only validation the store enforces) requires just a
non-empty command. There is no allowlist and **no check for control characters or newlines**, so a
command string can inject additional directives. The allowlist that exists
(`kernel_module_policy.rs:269-296`, `SP-KM4-UNAPPROVED-INSTALL-CMD` /
`SP-KM4-INSTALL-COMMAND-INJECTION`) lives in the *policy*, and the export path never evaluates it:
* `aiosh-cli/src/main.rs:9979-10083` loads the store, calls `export_modprobe_conf()`, and does
`std::fs::write(path_str, &modprobe_conf)` for `--modprobe <path>` with no `pep.check`, no policy
verdict, no `--yes` (see C-7). `std::fs::write` truncates and follows symlinks.
* `import --modprobe <path>` (`main.rs:10085+`) parses an existing conf back into the store, so an
already-present `install` line round-trips losslessly into any later export.
* The store JSON itself is the injection vector: `load_from_path` bounds size but validation permits
an arbitrary `Install.command`, and the MCP writes that store ungated (`aios.kernel_module.*` are all
`require_grant=false`).

**Impact.** `sudo aiosh kernel-module export --modprobe /etc/modprobe.d/aios.conf` — the command the
feature exists for, and the one the human operator will type — plants an attacker-chosen root command
that fires on the next `modprobe <module>` (including one triggered by hotplug). Any agent that can
write the store JSON (ungated MCP tool, or just a file write) supplies the payload; the human supplies
the root privilege. This is the most direct privilege-escalation chain in the codebase.

**Fix.** Run `KernelModulePolicy` in Enforcing mode immediately before export and refuse the write on
any violation; pin the destination to `config.modprobe_d_path` (or an explicit `--force-path` guarded
by a grant); reject control characters in `Install/Remove.command` in `validate_config` so the
data-model layer is safe on its own; require a PEP grant for the write.

### N-4 — Grants are self-service: the requester mints its own authorization

A grant is a bearer token in a shared SQLite table, and **minting it is an unauthenticated CLI verb**:

* `aiosh-cli/src/main.rs:8707-8750` (`cmd_grant_create`): `--to` is a free-form string, `--tools` is a
glob list, `--ttl` defaults to 3600 s, `--max-irreversible` may authorize irreversible tools, and the
audit actor is hardcoded `"user"`. No confirmation, no TTY check, no identity verification.
* `code/aiosh-cli/src/cli.ts:566-601` — the same verb in TypeScript, `issued_to: opts.to ?? "user"`.
* Both write the table the PEP reads: `$HOME/.aios/audit.db` (`aiosh-core/src/audit.rs:71-89`,
`code/aiosh-cli/src/audit.ts:224`). Same file, so a grant minted by either CLI is honoured by the MCP.
* `pep.rs:701-779` (the real check behind Gate #2) verifies revocation, expiry (fail-closed on a
malformed timestamp), `scope.tools` glob, `scope.networks`, and `scope.paths` — but **never compares
`g.issued_to` against the calling actor**. Any holder of the grant id can use it.

**Impact.** For the eight properly gated MCP tools and the `pentest.*` family, authorization reduces
to an audited formality: whatever can call the tool (the MCP server, the agent bridge, a script) can
equally run `aiosh grant create --tools 'pentest.*' --allow /` and satisfy it. The grant layer is a
traceability mechanism, not an access control, and nothing in the design distinguishes a human-issued
grant from an agent-issued one — the audit row simply says `user`.

**Fix.** Bind grants to an authenticated principal and compare it at dispatch; mint only after an
explicit out-of-band approval (TTY prompt, OS keychain signature, or a separate privileged daemon);
sign the scope so a grant row cannot be hand-inserted into the DB; and make the *requester* identity
part of the grant. Until then, treat every `require_grant=true` flag as advisory in the threat model.

## New: MEDIUM

### N-5 — Predictable, non-exclusive temp-file siblings in six store writers

Atomic-write implementations split into two camps, and the security-relevant one is the majority:

| Writer | Temp file | Exclusive? |
| --- | --- | --- |
| `handoff_service.rs:239` | `path.with_extension("tmp")` via `File::create` | no |
| `triage_service.rs:134-140` | `<path>.tmp` via `File::create` | no |
| `base_image_service.rs:207-219` | `<path>.tmp` via `File::create` | no |
| `kernel_module_service.rs:161` | `.tmp.<pid>.<name>` via `File::create` | no |
| `package_service.rs:477` | `<path>.tmp` via `std::fs::write` | no |
| `distro_service.rs:101` | `<path>.tmp.<pid>` via `fs::write` | no |
| `session.rs:416-424`, `ledger.rs:145`, `retention.rs:416` | `O_EXCL` + explicit `0600`/`0644` | **yes** |

`File::create`/`fs::write` follow an existing symlink and truncate without `O_EXCL`, so a pre-placed
symlink at the predictable temp name (`<store>.tmp` is guessable; the pid-scoped variants are only
marginally better) is written **through** to the link target before the `rename` replaces it. Combined
with N-1 the attacker controls both the temp name and the destination.

**Fix.** One `atomic_write_private(path, bytes)` helper (`OpenOptions::new().write(true).create_new(true)
.mode(0o600)`), used by all eleven writers. `ledger.rs`/`retention.rs` already show the correct pattern.

## New: LOW

### N-6 — `aiosh-sandbox` parses its header from anywhere in `argv`

`aiosh-sandbox/src/main.rs` locates its own flags with
`args.iter().position(|a| a == "--policy")`, scanning the *entire* argument vector — including the
arguments of the command it is about to wrap. `aiosh-sandbox --policy '{}' -- echo --policy '{"paths_rw":["/"]}' -- /bin/sh`
re-reads the second `--policy`, re-splits on the second `--`, and executes `/bin/sh` with a
different policy than the caller passed. Today's callers (`code/aiosh-cli/src/cli.ts:277`, and the
Rust equivalent) always emit `--policy` first, so this is not reachable from the shipped CLIs — but
the parser is ambiguous, and any future caller that forwards user argv inherits the confusion.

**Fix.** Require a strict positional header (`argv[0] == "--policy"`, then `--`); reject anything else.
Note also that a *missing* `--policy` falls back to defaults with `inherit_defaults: true` and the
17-syscall seccomp denylist still applied, which is acceptable fail-safe behaviour.

## Verified clean this pass (and one narrowing result)

* **No binaries are tracked in git.** `git ls-files` matches **zero** files for
`*.zip|*.iso|*.gguf|*.tar.gz|*.img|*.qcow2|*.bin`. The 58 backup ZIPs, the ISOs and the GGUF model in
the working tree are untracked ignored artifacts, not repository content — so they are a local
hygiene issue, not a supply-chain disclosure.
* **The shipped TS build matches its source.** `code/aiosh-cli/dist/*.js` is *newer* than `src/*.ts` and
consistent in the sampled regions (`AIOSH_SANDBOX_BIN` appears once on each side, same value), so the
`bin`/`main` targets in `package.json` are not stale with respect to the audited sources. (Fresh clones
have no `dist/`, so `npm start` fails until `npm run build` — DX, not security.)
* **`tools/*.py` (41 files) contain no shell execution.** No `shell=True`, `os.system`, `eval` or
`exec`. The single `subprocess.Popen` (`tools/ci_run.py:129`) passes an argv vector, sets
`start_new_session`, kills the process group on timeout, and is not exposed as an MCP/agent tool (only
`triage_service.rs:256` mentions `ci_run` as a record label).
* **`agent_bridge.py` is transport, not policy.** It allowlists 9 canonical tools, validates the op and
argument shape, and forwards to `aiosh_mcp.server` over MCP stdio; it mints nothing, checks nothing, and
correctly defers classifier/PEP/audit to the server. No grant logic to bypass there.
* **Gate #2 itself is sound once a grant is present.** `pep.rs:701-779` checks revocation, expiry
(fail-closed on malformed timestamps), anchored tool-glob matching, network scope, and *both*
`target` and the hidden `path_subjects` (the register-`spec` hole really is closed).
* **Module-name injection into `modprobe.d` is blocked** by `validate_module_name` (ASCII alphanumeric
+ `_`, ≤64 chars) for `blacklist`/`alias`/`softdep`/`options`/autoload — which narrows N-3 to exactly
two unconstrained fields, `Install.command` and `Remove.command`, and makes that fix small.
* **Store load paths are bounded** (`MAX_STORE_BYTES`/`MAX_MODULE_DOC_BYTES`, `metadata()` before read,
`take(cap+1)`), and `KernelModuleStore::save_to_path` calls `validate()` before serializing.

## Coverage note for this pass

Read closely: `kernel_module*.rs` (policy/config/service/data-model), `package_policy.rs` (+ the
`resolve`/`from_source` bodies), `package_service.rs`/`distro_service.rs`/`service_service.rs` save and
temp-file paths, `handoff_service.rs` persistence, `pep.rs` check body, `tools/*.py` (all 41, swept),
`tools/task_ledger.py` (state/lock/evidence-stub writes), `aiosh_mcp/agent_bridge.py`,
`aiosh-sandbox/src/main.rs`, plus the CLI's export/import arms (`main.rs:9935-10114`) and grant arms
(`main.rs:8707-8820`). Still not read line-by-line (next candidates): the verdict bodies of
`session_policy`/`distro_policy`/`base_image_policy`, `repo_health*`, `ci*`, `doc_index*`,
`code/aiosh-cli/src/{constitution,pentest,types}.ts`, and the Python `tools/ci_service.py` report layer.

---
