//! aiosh — the AIOS shell CLI (Rust rewrite).
//!
//! Subcommands (each emits exactly one audit row):
//!   aiosh status
//!   aiosh run <command...> [--target T]
//!   aiosh agent <prompt> [--grant G] [--max-steps N] [--ollama-url U] [--ollama-model M]
//!   aiosh audit tail [n]
//!   aiosh audit verify [--full]
//!   aiosh audit rotate [--keep N] [--dry-run]
//!   aiosh audit segments
//!   aiosh audit seen <hash> [--exact]
//!   aiosh grant create --to S --tools GLOBS [--networks CIDRS] [--allow P] [--deny P] [--ttl S]
//!   aiosh grant list
//!   aiosh grant revoke <id>
//!   aiosh pentest {nmap|nikto|sqlmap|tshark|aircrack-ng} <args...> [--grant G] [--timeout-s N]
//!   aiosh classify <tool> [--target T] [--json-args S]

use aiosh_core::audit::{AuditRing, AuditRowInput};
use aiosh_core::classifier::classify;
use aiosh_core::pentest;
use aiosh_core::pep::PepStore;
use aiosh_core::retention;
use aiosh_core::types::{AuditRow, CFlags, GrantScope, PathScope};
use serde_json::{json, Value};
use std::process::exit;

const IMPLICIT_REVISION: &str = "v0.0";

fn ai_home() -> String {
    std::env::var("AIOSH_HOME").unwrap_or_else(|_| {
        format!("{}/.aios", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
    })
}

fn db_path() -> String {
    format!("{}/audit.db", ai_home())
}

struct Ctx {
    ring: AuditRing,
    pep: PepStore,
    con_rev: String,
    con_title: String,
    con_source: String,
    actor_id: String,
}

fn open_context() -> Ctx {
    let path = db_path();
    let ring = AuditRing::open(aiosh_core::audit::OpenOptions {
        path: Some(path.clone()),
        home: None,
    })
    .expect("open audit db");
    ring.prepare_for_write().expect("prepare schemas");
    let pep_path = ring.path().to_string();
    let pep = if pep_path == ":memory:" {
        PepStore::new(rusqlite::Connection::open_in_memory().unwrap()).expect("open pep store")
    } else {
        PepStore::new(rusqlite::Connection::open(&pep_path).expect("open pep db")).expect("open pep store")
    };
    let con_path = std::env::var("AIOSH_CONSTITUTION").unwrap_or_else(|_| {
        "/content/AIOS_MERGED/mostimportanAIfolder/AI_CONSTITUTION.md".into()
    });
    let (con_rev, con_title) = match std::fs::read_to_string(&con_path) {
        Ok(content) => {
            let rev = aiosh_core::canonical::sha256_hex_bytes(content.as_bytes())[..12].to_string();
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_else(|| "untitled".into());
            (rev, title)
        }
        Err(_) => (IMPLICIT_REVISION.into(), "(no constitution file found)".into()),
    };
    let actor_id = format!(
        "user:{}@{}",
        std::env::var("USER").unwrap_or_else(|_| "anon".into()),
        std::env::var("HOSTNAME").unwrap_or_else(|_| "host".into())
    );
    Ctx { ring, pep, con_rev, con_title, con_source: con_path, actor_id }
}

/// Emit an audit row with optional classifier provenance.
#[allow(clippy::too_many_arguments)]
fn emit(
    ctx: &mut Ctx,
    tool: &str,
    command: &str,
    args: Value,
    outcome: &str,
    target: Option<&str>,
    outcome_detail: Option<&str>,
    actor: &str,
    grant_token: Option<&str>,
    c_flags: CFlags,
    cls: Option<&aiosh_core::classifier::ClassificationResult>,
) -> AuditRow {
    let input = AuditRowInput {
        ts: aiosh_core::canonical::utcnow_iso(),
        actor: actor.into(),
        actor_id: ctx.actor_id.clone(),
        tool: tool.into(),
        command: command.into(),
        args,
        target: target.map(|s| s.into()),
        outcome: outcome.into(),
        outcome_detail: outcome_detail.map(|s| s.into()),
        constitution_rev: Some(ctx.con_rev.clone()),
        grant_token: grant_token.map(|s| s.into()),
        c_flags,
        policy_revision: cls.map(|c| c.policy_revision.clone()),
        classify_rule_ids: cls.map(|c| c.rule_ids.clone()),
        classify_evidence: cls.map(|c| c.evidence_per_flag()),
        classify_overall_verdict: cls.map(|c| c.overall_verdict.clone()),
        classify_verdict_reason: cls.map(|c| c.verdict_reason.clone()),
    };
    ctx.ring.write(input).expect("audit write")
}

fn classify_and_emit(
    ctx: &mut Ctx,
    tool: &str,
    command: &str,
    args: Value,
    outcome: &str,
    target: Option<&str>,
    outcome_detail: Option<&str>,
    actor: &str,
    grant_token: Option<&str>,
) -> AuditRow {
    let cls = classify(tool, target, &args);
    let c = CFlags {
        c1: cls.c1.flag,
        c2: cls.c2.flag,
        c3: cls.c3.flag,
        c4: cls.c4.flag,
    };
    emit(ctx, tool, command, args, outcome, target, outcome_detail, actor, grant_token, c, Some(&cls))
}

fn ok_out(v: Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
}

fn err_out(v: Value) -> i32 {
    eprintln!("{}", serde_json::to_string_pretty(&v).unwrap());
    1
}

fn print_result(v: Value, ok: bool) -> i32 {
    if ok { ok_out(v); 0 } else { err_out(v) }
}

fn parse_flag(args: &[String], name: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
    }
    None
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn main() {
    // T-00038 hardening: never panic on hostile/accidental non-UTF-8
    // argv — lossy-convert instead (invalid bytes become U+FFFD), so
    // every invocation reaches the standard envelope + audit row.
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let code = match args.first().map(|s| s.as_str()) {
        Some("status") => cmd_status(),
        Some("run") => cmd_run(&args[1..]),
        Some("agent") => cmd_agent(&args[1..]),
        Some("audit") => cmd_audit(&args[1..]),
        Some("grant") => cmd_grant(&args[1..]),
        Some("pentest") => cmd_pentest(&args[1..]),
        Some("classify") => cmd_classify(&args[1..]),
        Some("ci") => cmd_ci(&args[1..]),
        Some("task") => cmd_task(&args[1..]),
        Some("--help") | Some("-h") | None => {
            println!("aiosh — AIOS shell CLI (Rust)\n\nUsage: aiosh <status|run|agent|audit|grant|pentest|classify|task|ci> ...\n\n  aiosh task <status|done|block|unblock|skip|rebuild|check>  Task ledger control\n  aiosh ci <show|failures|check|config|metrics> [--file PATH]  CI smoke reports");
            0
        }
        Some(other) => {
            eprintln!("unknown command: {}", other);
            2
        }
    };
    exit(code);
}

/// `aiosh task` — Task Ledger Control (T-00016; unified validation
/// T-00034 via `task_service::TaskCall`, spec T-00032). Every outcome —
/// including usage refusals — writes an honest audit row (ADR-0035 §F-2).
fn cmd_ci(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let action = args.first().map(|s| s.as_str());
    let file_arg = parse_flag(args, "--file").unwrap_or_else(|| {
        std::env::var("AIOSH_CI_RESULTS").unwrap_or_else(|_| "/tmp/aiosh-ci-results.json".to_string())
    });

    let path = std::path::Path::new(&file_arg);
    let summary = match aiosh_core::ci::load_summary_with_retry(path, 3) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ci-service: {}", e);
            emit(
                &mut ctx,
                "aiosh",
                "ci check",
                json!({"file": file_arg}),
                "error",
                None,
                Some(&e),
                "human",
                None,
                CFlags::empty(),
            );
            return 2;
        }
    };

    match action {
        Some("show") => {
            print!("{}", aiosh_core::ci::human_report(&summary));
            0
        }
                Some("metrics") => {
            match aiosh_core::ci_config::CiConfig::from_env() {
                Ok(cfg) => {
                    let snapshot = serde_json::json!({
                        "ok": true,
                        "action": "metrics",
                        "ci": &summary,
                        "config": cfg.to_json_with_sources()
                    });
                    println!("{}", serde_json::to_string_pretty(&snapshot).unwrap());
                    0
                }
                Err(e) => {
                    eprintln!("{}", e);
                    2
                }
            }
        }
        Some("config") => {
            match aiosh_core::ci_config::CiConfig::from_env() {
                Ok(cfg) => {
                    println!("{}", serde_json::to_string_pretty(&cfg.to_json_with_sources()).unwrap());
                    0
                }
                Err(e) => {
                    eprintln!("{}", e);
                    2
                }
            }
        }
        Some("failures") => {
            let failures: Vec<_> = summary.results.iter().filter(|r| r.status != "pass").collect();
            if failures.is_empty() {
                println!("no failed suites");
            } else {
                for r in failures {
                    let rc = r.exit_code.map_or("-".to_string(), |c| c.to_string());
                    println!("[FAIL] {} {} ({} ms) exit={} log={}", r.index, r.suite, r.duration_ms, rc, r.log_path);
                }
            }
            0
        }
        Some("check") => {
            let passed = summary.all_pass;
            let msg = if passed {
                format!("ci-check: PASS ({}/{} suites)", summary.passed, summary.total)
            } else {
                format!("ci-check: FAIL ({}/{} suites, {} failed)", summary.passed, summary.total, summary.failed)
            };
            println!("{}", msg);
            
            emit(
                &mut ctx,
                "aiosh",
                "ci check",
                json!({"file": file_arg}),
                if passed { "success" } else { "failure" },
                None,
                Some(&msg),
                "human",
                None,
                CFlags::empty(),
            );

            if passed { 0 } else { 1 }
        }
        _ => {
            eprintln!("usage: aiosh ci <show|failures|check|config|metrics> [--file PATH]

  config                       print resolved configuration and source
    metrics                      print consolidated JSON observability snapshot");
            2
        }
    }
}

fn cmd_task(args: &[String]) -> i32 {
    if args.first().map(|s| s.as_str()) == Some("help") {
        println!("{}", task_usage_text(None));
        return 0;
    }
    let sub = args.first().cloned().unwrap_or_default();
    let label = if sub.is_empty() { "task".to_string() } else { format!("task {sub}") };
    let mut ctx = open_context();
    let run = || -> Result<Value, String> {
        if sub == "config" {
            let cfg = aiosh_core::ledger_config::LedgerConfig::from_env()?;
            return Ok(cfg.to_json_with_sources());
        }
        if sub == "metrics" {
            // T-00085: metrics is a no-operand action; a stray token is a
            // loud usage refusal (audited via the standard envelope below),
            // never silently ignored.
            if let Some(extra) = args.get(1) {
                return Err(format!(
                    "unexpected argument '{extra}' — 'metrics' takes no operands"
                ));
            }
            let cfg = aiosh_core::ledger_config::LedgerConfig::from_env()?;
            let tasks = aiosh_core::ledger::load_state(&aiosh_core::ledger::paths()?.state,
                                                       &aiosh_core::ledger::paths()?.events)?;
            let vr = ctx.ring.verify().map_err(|e| e.to_string())?;
            let head = ctx.ring.tail(1).unwrap_or_default();
            let prefix = head.first().map(|r| r.hash.chars().take(16).collect::<String>())
                               .unwrap_or_default();
            return aiosh_core::task_service::TaskCall::build_metrics_pub(
                tasks, vr.checked, vr.ok, &prefix, &cfg);
        }
        let parsed = parse_task_args(args)?;
        let call = parsed.call();
        call.validate()?;
        let p = aiosh_core::ledger::paths()?;
        if call.action == aiosh_core::task_service::TaskAction::Metrics {
            // T-00084: consolidated observability snapshot. The CLI
            // owns the ring handle, so it supplies audit facts here
            // (read-only surface; same envelope as other subs).
            let verify = ctx.ring.verify().map_err(|e| e.to_string())?;
            // T-00088 hardening: O(1) COUNT(*) instead of loading every
            // live row into memory via tail(i64::MAX).
            let rows = ctx.ring.count().unwrap_or(0) as usize;
            let head = ctx.ring.tail(1).unwrap_or_default();
            let head_prefix = head.first().map(|r| r.hash.chars().take(12).collect::<String>()).unwrap_or_default();
            let cfg = aiosh_core::ledger_config::LedgerConfig::from_env()?;
            let tasks = aiosh_core::ledger::load_state(&p.state, &p.events)?;
            return aiosh_core::task_service::TaskCall::build_metrics(tasks, rows, verify.ok, &head_prefix, &cfg);
        }
        call.execute_with(&p)
    };
    let (code, outcome, detail) = match run() {
        Ok(v) => {
            ok_out(json!({"ok": true, "subcommand": label, "data": v}));
            (0, "ok", None)
        }
        Err(e) => {
            err_out(json!({"ok": false, "subcommand": label, "error": e}));
            (1, "refused", Some(e))
        }
    };
    emit(
        &mut ctx,
        "task.ledger",
        &format!("aiosh {}", label),
        json!({"subcommand": sub, "args": &args[1.min(args.len())..]}),
        outcome,
        None,
        detail.as_deref(),
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    code
}

// ----------------------------------------------------------------------
// T-00034 IMPLEMENTATION — CLI surface unification (spec T-00032).
//
// Replaces the permissive `flag_after` parsing (research T-00031
// Q2/Q3/Q4/Q6). cmd_task now routes through these helpers.
//
// Contract highlights (spec §2):
//   - argv -> TaskArgsOwned mirroring `task_service::parse_args`
//     semantics for non-JSON input: decimal u64 >= 1 ids, non-empty
//     note/reason, 4096-byte text cap, <=16 evidence items, values
//     must not start with "--" unless after a `--` delimiter,
//     unknown subcommand/missing values are usage errors.
//   - `--` ends option parsing; later tokens are values even when
//     dash-prefixed.
//   - options-after-operands stays (documented POSIX-G9 deviation).

// ----------------------------------------------------------------------
// T-00034 IMPLEMENTATION — CLI surface unification (spec T-00032).
//
// argv -> TaskArgsOwned mirroring `task_service::parse_args` semantics
// for non-JSON input. Grammar (spec §2):
//   - decimal u64 >= 1 operand where required; exactly one.
//   - options AFTER operands allowed (documented POSIX-G9 deviation).
//   - option values must be separate arguments, must not start with
//     "--" unless after a `--` delimiter (G7/G14), and must exist (no
//     silent-empty).
//   - `--` ends option parsing; later tokens are values even when
//     dash-prefixed (G10).
//   - unknown dash-tokens are errors naming the token.
// Semantic rules (non-empty, caps, conditional ids) stay in
// `TaskCall::validate` — the SAME code the MCP surface runs.

fn missing_value(opt: &str) -> String {
    format!("missing value for '{opt}'\n{}", task_usage_text(None))
}

/// Consume the value token for `opt` starting at `*i` (pointing at the
/// option). A bare `--` in value position becomes the delimiter and the
/// FOLLOWING token is taken as the value (spec §2 / POSIX G10+G14).
fn take_value(
    args: &[String],
    i: &mut usize,
    opt: &str,
    past_dd: &mut bool,
) -> Result<String, String> {
    *i += 1;
    loop {
        let v = args.get(*i).ok_or_else(|| missing_value(opt))?;
        if v == "--" && !*past_dd {
            *past_dd = true;
            *i += 1;
            continue;
        }
        if v.starts_with("--") && !*past_dd {
            return Err(format!(
                "value for '{opt}' must not start with \"--\" (use \"--\" to pass literals)"
            ));
        }
        return Ok(v.clone());
    }
}

fn parse_task_args(args: &[String]) -> Result<aiosh_core::task_service::TaskArgsOwned, String> {
    use aiosh_core::task_service::{TaskAction, TaskArgsOwned, MAX_EVIDENCE_ITEMS};
    let overview = task_usage_text(None);
    let sub = args
        .first()
        .ok_or_else(|| overview.clone())?;
    let action =
        TaskAction::parse(sub).ok_or_else(|| format!("unknown subcommand '{sub}'\n{overview}"))?;
    let needs_id = !matches!(
        action,
        TaskAction::Status | TaskAction::Check | TaskAction::Validate | TaskAction::Rebuild
    );
    let mut task_id: Option<u64> = None;
    let mut note: Option<String> = None;
    let mut reason: Option<String> = None;
    let mut evidence: Vec<String> = Vec::new();
    let mut past_dd = false;
    let mut i = 1;
    while i < args.len() {
        let tok = &args[i];
        if !past_dd && tok == "--" {
            past_dd = true;
            i += 1;
            continue;
        }
        if !past_dd && tok.starts_with('-') && tok.len() > 1 {
            let (slot, opt): (&mut Option<String>, &str) = match tok.as_str() {
                "--note" => (&mut note, "--note"),
                "--reason" => (&mut reason, "--reason"),
                "--evidence" => {
                    let v = take_value(args, &mut i, "--evidence", &mut past_dd)?;
                    if evidence.len() >= MAX_EVIDENCE_ITEMS {
                        return Err(format!("'evidence' exceeds {MAX_EVIDENCE_ITEMS} items"));
                    }
                    evidence.push(v);
                    i += 1; // take_value left i on the value token
                    continue;
                }
                other => {
                    return Err(format!(
                        "unknown option '{other}'\n{}",
                        task_usage_text(Some(sub))
                    ))
                }
            };
            *slot = Some(take_value(args, &mut i, opt, &mut past_dd)?);
            i += 1;
            continue;
        }
        // Bare token: the single required operand, or an error.
        if needs_id && task_id.is_none() {
            let id: u64 = tok
                .parse()
                .map_err(|_| format!("invalid task_id '{tok}' (decimal integer >= 1)\n{}", task_usage_text(Some(sub))))?;
            if id == 0 {
                return Err(format!(
                    "invalid task_id '0' (must be >= 1)\n{}",
                    task_usage_text(Some(sub))
                ));
            }
            task_id = Some(id);
        } else {
            return Err(format!(
                "unexpected argument '{tok}'\n{}",
                task_usage_text(Some(sub))
            ));
        }
        i += 1;
    }
    Ok(TaskArgsOwned {
        action,
        task_id,
        note,
        reason,
        evidence,
        grant_id: None,
    })
}

fn task_usage_text(sub: Option<&str>) -> String {
    let overview = "\
usage: aiosh task <status|check|validate|config|done|block|unblock|skip|rebuild|help> [args]

  status                       print TASK_STATE.json
  check                        validate ledger invariants
  validate                     read-only integrity report (state vs events)
  done     <id> --note <text> [--evidence <path>]...
  block    <id> --reason <text>
  unblock  <id> --reason <text>
  skip     <id> --reason <text>
  rebuild                      recompute pointer from COMPLETIONS.jsonl
  config                       print effective AIOSH_LEDGER_* settings
  help                         this help

notes:
  * done/block/unblock/skip act ONLY on the current next_task (no-skip law)
  * text values: non-empty, <= 4096 bytes; evidence: <= 16 paths
  * a value may not start with \"--\" unless preceded by a lone \"--\"
  * options may appear after the operand (intentional GNU-style deviation)";
    match sub {
        None => overview.to_string(),
        Some("done") => "\
usage: aiosh task done <task_id> --note <text> [--evidence <path>]...
  complete the current next_task; --note is required and non-empty."
            .into(),
        Some("block") => "\
usage: aiosh task block <task_id> --reason <text>
  refuse to proceed; pointer does not advance.".into(),
        Some("unblock") => "\
usage: aiosh task unblock <task_id> --reason <text>
  clear the blocked marker; pointer returns to <task_id>.".into(),
        Some("skip") => "\
usage: aiosh task skip <task_id> --reason <text>
  human override: record pointer_reset and advance past <task_id>.".into(),
        Some("status") => "usage: aiosh task status\n  print the live TASK_STATE.json pointer.".into(),
        Some("check") => "usage: aiosh task check\n  validate ledger invariants (ids contiguous, linear deps).".into(),
        Some("validate") => "usage: aiosh task validate\n  read-only integrity report: live state vs event-log replay (report-only; `task rebuild` remains the only repair path).".into(),
        Some("rebuild") => "usage: aiosh task rebuild\n  recompute TASK_STATE.json from the append-only event log.".into(),
        Some(other) => format!("no such subcommand '{other}'\n{overview}"),
    }
}

fn cmd_status() -> i32 {
    let mut ctx = open_context();
    let verify = ctx.ring.verify().expect("verify");
    let head = ctx.ring.tail(1).unwrap_or_default();
    emit(
        &mut ctx,
        "system.status",
        "aiosh status",
        json!({}),
        "ok",
        None,
        None,
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    let data = json!({
        "aiosh_version": "0.1.0",
        "ai_home": ai_home(),
        "audit_db": db_path(),
        "constitution_rev": ctx.con_rev,
        "constitution_title": ctx.con_title,
        "constitution_source": ctx.con_source,
        "audit_ring": {
            "verify_ok": verify.ok,
            "rows": verify.checked,
            "head_hash": head.first().map(|r| r.hash.clone()).unwrap_or_else(|| "null".into()),
        },
        "rust": format!("{}", rustc_version()),
    });
    ok_out(json!({"ok": true, "subcommand": "status", "outcome": "ok",
                  "audit_id": -1, "data": data}));
    0
}

fn rustc_version() -> String {
    let v = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    format!("aiosh-rust/{}", v)
}

fn cmd_run(args: &[String]) -> i32 {
    let mut ctx = open_context();
    // Single-pass option parsing: `--target VALUE` / `--target=VALUE` is
    // consumed together with its value; EVERY other token — including
    // dash-prefixed ones — belongs to the spawned command's argv.
    let mut target: Option<String> = None;
    let mut command_args: Vec<String> = Vec::new();
    {
        let mut it = args.iter();
        while let Some(a) = it.next() {
            if a == "--target" {
                target = it.next().cloned();
                continue;
            }
            if let Some(v) = a.strip_prefix("--target=") {
                target = Some(v.to_string());
                continue;
            }
            command_args.push(a.clone());
        }
    }
    if command_args.is_empty() {
        return err_out(json!({"ok": false, "subcommand": "run", "outcome": "error",
                              "audit_id": -1, "error": "usage: aiosh run <command...>"}));
    }
    let command = command_args.join(" ");
    let bin = command_args[0].clone();
    let rest = command_args[1..].to_vec();

    // Sandbox policy (conservative defaults).
    let policy = json!({
        "paths_ro": ["/usr", "/lib", "/lib64", "/etc/ld.so.cache",
                    "/etc/ld.so.conf", "/etc/ld.so.conf.d", "/dev", "/proc/self"],
        "paths_rw": ["/tmp"],
        "paths_execute": ["/usr/bin", "/usr/local/bin", "/bin"],
        "no_new_privs": true,
        "seccomp_denylist": ["ptrace", "mount", "umount2", "reboot", "kexec_load",
                             "kexec_file_load", "init_module", "finit_module",
                             "delete_module", "setuid", "setgid", "setreuid", "setregid",
                             "setresuid", "setresgid", "chroot", "pivot_root"],
        "inherit_defaults": true,
    });
    let argv: Vec<String> = std::iter::once(bin.clone())
        .chain(rest.clone())
        .collect();
    let result = aiosh_core::sandbox::sandbox_exec(&argv, &parse_policy(&policy));
    let mut full_argv = vec![bin.clone()];
    full_argv.extend(rest.clone());
    let outcome = if result == 0 { "ok" } else { "error" };
    let outcome_detail: Option<String> = if result == 0 {
        None
    } else {
        Some(format!("exit code {}", result))
    };
    classify_and_emit(
        &mut ctx,
        "process.run",
        &format!("aiosh run {}", command),
        json!({"bin": bin, "args": rest, "target": target, "policy": policy}),
        outcome,
        target.as_deref().or(Some(bin.as_str())),
        outcome_detail.as_deref(),
        "user",
        None,
    );
    ok_out(json!({"ok": result == 0, "subcommand": "run",
                  "outcome": if result == 0 { "ok" } else { "error" },
                  "audit_id": -1,
                  "data": {"bin": bin, "args": rest, "exit_code": result}}));
    if result == 0 { 0 } else { 1 }
}

fn parse_policy(v: &Value) -> aiosh_core::sandbox::SandboxPolicy {
    aiosh_core::sandbox::SandboxPolicy::from_json(&v.to_string()).unwrap_or_default()
}

fn cmd_agent(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let grant = parse_flag(args, "--grant");
    let max_steps: usize = parse_flag(args, "--max-steps")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
        .clamp(1, 32);
    let ollama_url = parse_flag(args, "--ollama-url").unwrap_or_else(|| "http://localhost:11434".into());
    let ollama_model =
        parse_flag(args, "--ollama-model").unwrap_or_else(|| "qwen2.5:7b-instruct".into());

    // First positional token(s) = the prompt. Known value-taking flags
    // are skipped together with their values so option values never
    // leak into the prompt text.
    const VALUE_FLAGS: [&str; 4] =
        ["--grant", "--max-steps", "--ollama-url", "--ollama-model"];
    let mut prompt_args: Vec<String> = Vec::new();
    {
        let mut it = args.iter();
        while let Some(a) = it.next() {
            if VALUE_FLAGS.contains(&a.as_str()) {
                let _ = it.next(); // consume the flag's value
                continue;
            }
            if a.starts_with("--") {
                continue;
            }
            prompt_args.push(a.clone());
        }
    }
    let prompt = prompt_args.join(" ");
    if prompt.is_empty() {
        return err_out(json!({"ok": false, "subcommand": "agent", "outcome": "error",
                              "audit_id": -1, "error": "usage: aiosh agent <prompt>"}));
    }

    if let Some(g) = &grant {
        if ctx.pep.get(g).expect("grant get").is_none() {
            let out = classify_and_emit(
                &mut ctx,
                "agent.invoke",
                "aiosh agent",
                json!({"prompt": prompt.chars().take(256).collect::<String>(), "grant": g}),
                "refused",
                None,
                Some(&format!("unknown grant {}", g)),
                "user",
                Some(g),
            );
            return err_out(json!({"ok": false, "subcommand": "agent", "outcome": "refused",
                                  "audit_id": out.id, "error": format!("unknown grant: {}", g),
                                  "data": {}}));
        }
    }

    classify_and_emit(
        &mut ctx,
        "agent.invoke",
        "aiosh agent",
        json!({"prompt": prompt.chars().take(256).collect::<String>(),
               "grant": grant, "max_steps": max_steps}),
        "ok",
        None,
        None,
        "user",
        grant.as_deref(),
    );

    let loop_opts = aiosh_core::agent::AgentLoopOptions {
        prompt: &prompt,
        grant_id: grant.as_deref(),
        ring: &mut ctx.ring,
        constitution_rev: &ctx.con_rev,
        ollama_url: &ollama_url,
        ollama_model: &ollama_model,
        max_steps,
        pep: &ctx.pep,
        dispatcher: None,
    };
    let result = aiosh_core::agent::run_agent_loop(loop_opts);
    ok_out(json!({"ok": true, "subcommand": "agent", "outcome": "ok",
                  "audit_id": -1, "data": result.to_json()}));
    0
}

fn cmd_audit(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str());
    match sub {
        Some("tail") => cmd_audit_tail(&args[1..]),
        Some("verify") => cmd_audit_verify(&args[1..]),
        Some("rotate") => cmd_audit_rotate(&args[1..]),
        Some("segments") => cmd_audit_segments(),
        Some("seen") => cmd_audit_seen(&args[1..]),
        _ => {
            eprintln!("usage: aiosh audit <tail|verify|rotate|segments|seen>");
            2
        }
    }
}

fn cmd_audit_tail(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let n: i64 = args
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
        .clamp(1, 1024);
    let rows = ctx.ring.tail(n).expect("tail");
    let rows_json: Vec<Value> = rows.iter().map(|r| row_to_json(r)).collect();
    emit(
        &mut ctx,
        "audit.tail",
        &format!("aiosh audit tail {}", n),
        json!({"n": n}),
        "ok",
        None,
        None,
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    ok_out(json!({"ok": true, "subcommand": "audit tail", "outcome": "ok",
                  "audit_id": -1, "data": {"count": rows_json.len(), "rows": rows_json}}));
    0
}

fn row_to_json(r: &AuditRow) -> Value {
    // Same shape as the legacy to_dict: base fields + conditional classifier.
    let mut m = serde_json::Map::new();
    m.insert("id".into(), json!(r.id));
    m.insert("ts".into(), json!(r.ts));
    m.insert("actor".into(), json!(r.actor));
    m.insert("actor_id".into(), json!(r.actor_id));
    m.insert("tool".into(), json!(r.tool));
    m.insert("command".into(), json!(r.command));
    m.insert("args".into(), r.args.clone());
    m.insert("target".into(), json!(r.target));
    m.insert("outcome".into(), json!(r.outcome));
    m.insert("outcome_detail".into(), json!(r.outcome_detail));
    m.insert("constitution_rev".into(), json!(r.constitution_rev));
    m.insert("grant_token".into(), json!(r.grant_token));
    m.insert("c_flags".into(), r.c_flags.to_json());
    if let Some(p) = &r.policy_revision {
        m.insert("policy_revision".into(), json!(p));
    }
    if let Some(ids) = &r.classify_rule_ids {
        m.insert("classify_rule_ids".into(), json!(ids));
    }
    if let Some(ev) = &r.classify_evidence {
        m.insert("classify_evidence".into(), ev.clone());
    }
    if let Some(v) = &r.classify_overall_verdict {
        m.insert("classify_overall_verdict".into(), json!(v));
    }
    if let Some(v) = &r.classify_verdict_reason {
        m.insert("classify_verdict_reason".into(), json!(v));
    }
    m.insert("prev_hash".into(), json!(r.prev_hash));
    m.insert("hash".into(), json!(r.hash));
    Value::Object(m)
}

fn cmd_audit_verify(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let full = has_flag(args, "--full");
    let conn = ctx.ring.conn();
    let v = if full {
        let res = retention::verify_full(&conn, None).expect("verify_full");
        json!({
            "ok": res.ok,
            "checked": res.checked,
            "broken_at": res.broken_at,
            "anchor": res.anchor,
            "segments": res.segments,
            "archive_checked": res.archive_checked,
            "live_checked": res.live_checked,
            "broken_segment": res.broken_segment,
            "error": res.error,
            "mode": "full",
        })
    } else {
        let res = ctx.ring.verify().expect("verify");
        json!({
            "ok": res.ok,
            "checked": res.checked,
            "broken_at": res.broken_at,
            "anchor": res.anchor,
            "segments": res.segments,
            "mode": "live",
        })
    };
    let vok = v["ok"].as_bool().unwrap_or(false);
    emit(
        &mut ctx,
        "audit.verify",
        &format!("aiosh audit verify{}", if full { " --full" } else { "" }),
        json!({"full": full}),
        if vok { "ok" } else { "refused" },
        None,
        if vok {
            None
        } else {
            v.get("error").and_then(|e| e.as_str())
        },
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    print_result(
        json!({"ok": vok, "subcommand": "audit verify",
               "outcome": if vok { "ok" } else { "refused" },
               "audit_id": -1, "data": v}),
        vok,
    )
}

fn cmd_audit_rotate(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let keep: i64 = parse_flag(args, "--keep").and_then(|s| s.parse().ok()).unwrap_or(0);
    let dry_run = has_flag(args, "--dry-run");
    let db_path = ctx.ring.path().to_string();
    let conn = if db_path == ":memory:" {
        rusqlite::Connection::open_in_memory().unwrap()
    } else {
        rusqlite::Connection::open(&db_path).unwrap()
    };
    let res = retention::rotate(
        &conn,
        &mut ctx.ring,
        retention::RotateOptions {
            keep_rows: keep,
            dry_run,
            actor: "user".into(),
            actor_id: ctx.actor_id.clone(),
            constitution_rev: Some(ctx.con_rev.clone()),
            ..Default::default()
        },
    )
    .expect("rotate");
    let out = json!({
        "ok": res.ok, "subcommand": "audit rotate",
        "outcome": if res.ok { "ok" } else { "refused" },
        "audit_id": res.audit_id.unwrap_or(-1),
        "data": res.to_json(),
        "error": res.error,
    });
    print_result(out, res.ok)
}

fn cmd_audit_segments() -> i32 {
    let mut ctx = open_context();
    let conn = ctx.ring.conn();
    let segs = retention::list_segments(&conn).expect("segments");
    let segs_json: Vec<Value> = segs
        .iter()
        .map(|s| {
            json!({
                "segment_id": s.segment_id,
                "closed_at": s.closed_at,
                "first_row_id": s.first_row_id,
                "last_row_id": s.last_row_id,
                "row_count": s.row_count,
                "genesis_prev_hash": s.genesis_prev_hash,
                "head_hash": s.head_hash,
                "archive_path": s.archive_path,
                "archive_sha256": s.archive_sha256,
                "bloom_m_bits": s.bloom_m_bits,
                "bloom_k": s.bloom_k,
                "bloom_hex": s.bloom_hex,
            })
        })
        .collect();
    emit(
        &mut ctx,
        "audit.segments",
        "aiosh audit segments",
        json!({}),
        "ok",
        None,
        None,
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    ok_out(json!({"ok": true, "subcommand": "audit segments", "outcome": "ok",
                  "audit_id": -1, "data": {"count": segs_json.len(), "segments": segs_json}}));
    0
}

fn cmd_audit_seen(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let hash = args
        .first()
        .cloned()
        .unwrap_or_default();
    let exact = has_flag(args, "--exact");
    let conn = ctx.ring.conn();
    let res = retention::seen(&conn, &hash, exact, None).expect("seen");
    emit(
        &mut ctx,
        "audit.seen",
        &format!("aiosh audit seen {}", hash),
        json!({"hash": hash, "exact": exact}),
        "ok",
        None,
        None,
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    ok_out(json!({"ok": true, "subcommand": "audit seen", "outcome": "ok",
                  "audit_id": -1, "data": res.to_json()}));
    0
}

fn cmd_grant(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str());
    match sub {
        Some("create") => cmd_grant_create(&args[1..]),
        Some("list") => cmd_grant_list(),
        Some("revoke") => cmd_grant_revoke(&args[1..]),
        _ => {
            eprintln!("usage: aiosh grant <create|list|revoke>");
            2
        }
    }
}

fn cmd_grant_create(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let to = parse_flag(args, "--to").unwrap_or_default();
    let tools_str = parse_flag(args, "--tools").unwrap_or_default();
    let networks = parse_flag(args, "--networks");
    let allow = parse_flag(args, "--allow");
    let deny = parse_flag(args, "--deny");
    let ttl: i64 = parse_flag(args, "--ttl").and_then(|s| s.parse().ok()).unwrap_or(3600);
    let max_irreversible = parse_flag(args, "--max-irreversible").and_then(|s| s.parse().ok());

    if to.is_empty() || tools_str.is_empty() {
        return err_out(json!({"ok": false, "subcommand": "grant create", "outcome": "error",
                              "audit_id": -1,
                              "error": "--to and --tools are required"}));
    }
    let scope = GrantScope {
        tools: tools_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        networks: networks
            .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
            .unwrap_or_default(),
        paths: PathScope {
            allow: allow
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default(),
            deny: deny
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default(),
        },
        max_irreversible,
    };
    let grant = ctx
        .pep
        .create(&scope, ttl, &to, &ctx.con_rev)
        .expect("grant create");
    emit(
        &mut ctx,
        "pep.grant.create",
        "aiosh grant create",
        json!({"scope": scope_to_json(&scope), "ttl": ttl, "issued_to": to}),
        "ok",
        Some(&grant.grant_id),
        None,
        "user",
        Some(&grant.grant_id),
        CFlags { c4: true, ..Default::default() },
        None,
    );
    ok_out(json!({"ok": true, "subcommand": "grant create", "outcome": "ok",
                  "audit_id": -1,
                  "data": {
                      "grant_id": grant.grant_id,
                      "issued_at": grant.issued_at,
                      "expires_at": grant.expires_at,
                      "issued_to": grant.issued_to,
                      "constitution_rev": grant.constitution_rev,
                      "scope": scope_to_json(&grant.scope),
                  }}));
    0
}

fn scope_to_json(scope: &GrantScope) -> Value {
    json!({
        "tools": scope.tools,
        "networks": scope.networks,
        "paths": {"allow": scope.paths.allow, "deny": scope.paths.deny},
        "max_irreversible": scope.max_irreversible,
    })
}

fn cmd_grant_list() -> i32 {
    let mut ctx = open_context();
    let grants = ctx.pep.list(true).expect("grant list");
    let grants_json: Vec<Value> = grants
        .iter()
        .map(|g| {
            json!({
                "grant_id": g.grant_id,
                "issued_at": g.issued_at,
                "expires_at": g.expires_at,
                "issued_to": g.issued_to,
                "constitution_rev": g.constitution_rev,
                "scope": scope_to_json(&g.scope),
            })
        })
        .collect();
    emit(
        &mut ctx,
        "pep.grant.list",
        "aiosh grant list",
        json!({}),
        "ok",
        None,
        None,
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    ok_out(json!({"ok": true, "subcommand": "grant list", "outcome": "ok",
                  "audit_id": -1, "data": {"count": grants_json.len(), "grants": grants_json}}));
    0
}

fn cmd_grant_revoke(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let grant_id = args.first().cloned().unwrap_or_default();
    let ok = ctx.pep.revoke(&grant_id).expect("revoke");
    emit(
        &mut ctx,
        "pep.grant.revoke",
        &format!("aiosh grant revoke {}", grant_id),
        json!({"grant_id": grant_id}),
        if ok { "ok" } else { "refused" },
        Some(&grant_id),
        if ok { None } else { Some("grant already revoked") },
        "user",
        None,
        CFlags { c4: true, ..Default::default() },
        None,
    );
    print_result(
        json!({"ok": ok, "subcommand": "grant revoke",
               "outcome": if ok { "ok" } else { "refused" },
               "audit_id": -1,
               "data": {"grant_id": grant_id, "revoked": ok}}),
        ok,
    )
}

fn cmd_pentest(args: &[String]) -> i32 {
    let tool = args.first().map(|s| s.as_str()).unwrap_or("");
    let rest = &args[1..];
    let grant = parse_flag(rest, "--grant");
    let timeout: u64 = parse_flag(rest, "--timeout-s").and_then(|s| s.parse().ok()).unwrap_or(60);
    // Positional args (drop --grant/--timeout-s and their values).
    let positional = strip_flags(rest, &["--grant", "--timeout-s"]);

    let mut ctx = open_context();
    let result = match tool {
        "nmap" => {
            let target = positional.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return err_out(json!({"ok": false, "error": "nmap requires <target>"}));
            }
            pentest::pentest_nmap(&mut pentest_ctx(&mut ctx), &target, grant.as_deref(), timeout)
        }
        "nikto" => {
            let target = positional.first().cloned().unwrap_or_default();
            if target.is_empty() {
                return err_out(json!({"ok": false, "error": "nikto requires <target>"}));
            }
            pentest::pentest_nikto(&mut pentest_ctx(&mut ctx), &target, grant.as_deref(), timeout)
        }
        "sqlmap" => {
            let url = positional.first().cloned().unwrap_or_default();
            if url.is_empty() {
                return err_out(json!({"ok": false, "error": "sqlmap requires <url>"}));
            }
            let level: i64 = parse_flag(rest, "--level").and_then(|s| s.parse().ok()).unwrap_or(1);
            let risk: i64 = parse_flag(rest, "--risk").and_then(|s| s.parse().ok()).unwrap_or(1);
            pentest::pentest_sqlmap(&mut pentest_ctx(&mut ctx), &url, grant.as_deref(), level, risk, timeout)
        }
        "tshark" => {
            let pcap = positional.first().cloned().unwrap_or_default();
            if pcap.is_empty() {
                return err_out(json!({"ok": false, "error": "tshark requires <pcap_path>"}));
            }
            let filter = parse_flag(rest, "--display-filter");
            pentest::pentest_tshark(&mut pentest_ctx(&mut ctx), &pcap, filter.as_deref(), grant.as_deref(), timeout)
        }
        "aircrack-ng" => {
            let capture = positional.first().cloned().unwrap_or_default();
            let wordlist = positional.get(1).cloned().unwrap_or_default();
            if capture.is_empty() || wordlist.is_empty() {
                return err_out(json!({"ok": false, "error": "aircrack-ng requires <capture> <wordlist>"}));
            }
            pentest::pentest_aircrack_ng(&mut pentest_ctx(&mut ctx), &capture, &wordlist, grant.as_deref(), timeout)
        }
        _ => {
            return err_out(json!({"ok": false, "error": format!(
                "unknown pentest tool: {} (nmap|nikto|sqlmap|tshark|aircrack-ng)", tool)}));
        }
    };
    let r_ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    print_result(
        json!({"ok": r_ok, "subcommand": format!("pentest {}", tool),
               "outcome": if r_ok { "ok" } else { "refused" },
               "audit_id": result.get("audit_id").cloned().unwrap_or(json!(-1)),
               "data": result}),
        r_ok,
    )
}

fn pentest_ctx(ctx: &mut Ctx) -> pentest::RunToolCtx<'_> {
    pentest::RunToolCtx {
        ring: &mut ctx.ring,
        pep: &ctx.pep,
        constitution_rev: &ctx.con_rev,
        actor_id: &ctx.actor_id,
    }
}

fn strip_flags(args: &[String], flags: &[&str]) -> Vec<String> {
    let mut out = vec![];
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        if flags.contains(&a.as_str()) {
            let _ = it.next();
            continue;
        }
        out.push(a.clone());
    }
    out
}

fn cmd_classify(args: &[String]) -> i32 {
    let tool = args.first().cloned().unwrap_or_default();
    if tool.is_empty() {
        eprintln!("usage: aiosh classify <tool> [--target T] [--json-args S]");
        return 2;
    }
    let target = parse_flag(args, "--target");
    let json_args = parse_flag(args, "--json-args").unwrap_or_else(|| "{}".into());
    let parsed: Value = match serde_json::from_str(&json_args) {
        Ok(v) => v,
        Err(e) => {
            return err_out(json!({"ok": false, "subcommand": "classify", "outcome": "error",
                                  "audit_id": 0,
                                  "data": {"error": format!("invalid --json-args: {}", e)}}));
        }
    };
    let result = classify(&tool, target.as_deref(), &parsed);
    println!(
        "{}",
        serde_json::to_string_pretty(&result.to_dict()).unwrap()
    );
    0
}

#[cfg(test)]
mod task_cli_tests {
    use super::*;
    use aiosh_core::task_service::TaskAction;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_done_with_note_and_repeatable_evidence() {
        let a = parse_task_args(&s(&[
            "done", "2", "--note", "n", "--evidence", "a.md", "--evidence", "b.md",
        ]))
        .unwrap();
        assert_eq!(a.action, TaskAction::Done);
        assert_eq!(a.task_id, Some(2));
        assert_eq!(a.note.as_deref(), Some("n"));
        assert_eq!(a.evidence, ["a.md", "b.md"]);
        assert!(a.call().validate().is_ok());
    }

    #[test]
    fn parses_status_without_operand() {
        let a = parse_task_args(&s(&["status"])).unwrap();
        assert_eq!(a.action, TaskAction::Status);
        assert!(a.call().validate().is_ok());
    }

    #[test]
    fn rejects_empty_note() {
        let e = parse_task_args(&s(&["done", "1", "--note", ""]))
            .unwrap()
            .call()
            .validate()
            .unwrap_err();
        assert!(e.contains("'note'"), "{e}");
    }

    #[test]
    fn rejects_missing_note() {
        let e = parse_task_args(&s(&["done", "1"]))
            .unwrap()
            .call()
            .validate()
            .unwrap_err();
        assert!(e.contains("'note'"), "{e}");
    }

    #[test]
    fn rejects_oversized_text_at_validate() {
        // Caps live in TaskCall::validate (single source with MCP).
        let long = "x".repeat(4097);
        assert!(parse_task_args(&s(&["done", "1", "--note", &long]))
            .unwrap()
            .call()
            .validate()
            .unwrap_err()
            .contains("exceeds"));
        assert!(parse_task_args(&s(&["block", "1", "--reason", &long]))
            .unwrap()
            .call()
            .validate()
            .unwrap_err()
            .contains("exceeds"));
    }

    #[test]
    fn rejects_dash_leading_option_value() {
        let e = parse_task_args(&s(&["block", "1", "--reason", "--force"])).unwrap_err();
        assert!(e.contains("must not start with"), "{e}");
    }

    #[test]
    fn rejects_missing_value_at_end() {
        let e = parse_task_args(&s(&["skip", "1", "--reason"])).unwrap_err();
        assert!(e.contains("missing value for '--reason'"), "{e}");
    }

    #[test]
    fn rejects_unknown_option_token() {
        let e = parse_task_args(&s(&["status", "--wat"])).unwrap_err();
        assert!(e.contains("unknown option '--wat'"), "{e}");
    }

    #[test]
    fn double_dash_allows_dash_leading_values() {
        let a =
            parse_task_args(&s(&["done", "2", "--note", "--", "-not-a-flag"])).unwrap();
        assert_eq!(a.note.as_deref(), Some("-not-a-flag"));
        assert!(
            parse_task_args(&s(&["done", "2", "--note", "n", "--", "extra"])).is_err(),
            "bare token after consumed value must still error"
        );
    }

    #[test]
    fn id_must_be_decimal_gte_one() {
        assert!(parse_task_args(&s(&["done", "abc", "--note", "n"])).is_err());
        assert!(parse_task_args(&s(&["done", "-2", "--note", "n"])).is_err());
        assert!(parse_task_args(&s(&["done", "0", "--note", "n"])).is_err());
    }

    #[test]
    fn extra_operand_rejected_for_read_only_actions() {
        // Stricter-but-earlier: parse refuses before validate runs.
        let e = parse_task_args(&s(&["status", "5"])).unwrap_err();
        assert!(e.contains("unexpected argument '5'"), "{e}");
    }

    #[test]
    fn evidence_item_cap_enforced_by_validate() {
        let long = "y".repeat(4097);
        let a = parse_task_args(&s(&["done", "1", "--note", "n", "--evidence", &long]))
            .unwrap();
        assert!(a.call().validate().unwrap_err().contains("'evidence' item"));
    }

    #[test]
    fn usage_text_lists_contract() {
        let t = task_usage_text(None);
        for want in ["done", "--note", "help", "4096", "\"--\""] {
            assert!(t.contains(want), "missing {want}");
        }
        assert!(task_usage_text(Some("done")).contains("--note <text>"));
        assert!(task_usage_text(Some("nope")).contains("no such subcommand"));
    }
}
