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
        Some("release") => cmd_release(&args[1..]),
        Some("backup") => cmd_backup(&args[1..]),
        Some("ci") => cmd_ci(&args[1..]),
        Some("task") => cmd_task(&args[1..]),
        Some("toolchain") => cmd_toolchain(&args[1..]),
        Some("doc") => cmd_doc(&args[1..]),
        Some("evidence") => cmd_evidence(&args[1..]),
        Some("repo") => cmd_repo(&args[1..]),
        Some("secrets") => cmd_secrets(&args[1..]),
        Some("triage") => cmd_triage(&args[1..]),
        Some("handoff") => cmd_handoff(&args[1..]),
        Some("distro") => cmd_distro(&args[1..]),
        Some("image") => cmd_image(&args[1..]),
        Some("--help") | Some("-h") | None => {
            println!("aiosh — AIOS shell CLI (Rust)\n\nUsage: aiosh <status|run|agent|audit|grant|pentest|classify|task|ci|release|backup|toolchain|doc|evidence|repo|secrets|triage|handoff|distro|image> ...\n\n  aiosh task <status|done|block|unblock|skip|rebuild|check>  Task ledger control\n  aiosh ci <show|failures|check|config|metrics> [--file PATH]  CI smoke reports\n  aiosh release generate  Create bootable ISO\n  aiosh backup create  Create system snapshot zip\n  aiosh toolchain check [--config <path>]  Verify host environment against ToolchainManifest\n  aiosh toolchain show [--config <path>]   Display the resolved ToolchainManifest\n  aiosh doc <show|check|search>  Documentation Index Control\n  aiosh evidence <verify|hash|scan>   Evidence & Audit Trail Control\n  aiosh repo <health|check>  Repository Health Diagnostics\n  aiosh secrets <scan|check> [--config <path>]  Secrets & Access Hygiene Scanner\n  aiosh triage <list|show|record|resolve|ingest|check>  Regression Triage Manager\n  aiosh handoff <list|show|initiate|accept|reject|complete|cancel>  Agent Handoff Protocol Manager\n  aiosh distro <list|show|evaluate|recommend|policy|stats|check>  Linux Distro Selection & Justification Manager\n  aiosh image <list|show|plan|filter>  Linux Base Image Build & Packaging Manager");
            0
        }
        Some(other) => {
            eprintln!("unknown command: {}", other);
            2
        }
    };
    exit(code);
}

fn cmd_distro(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    let is_json = has_flag(rest, "--json");

    let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
    let store = if let Some(path_str) = parse_flag(rest, "--store") {
        match aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(&path_str)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error loading distro store: {}", e);
                return 1;
            }
        }
    } else {
        match aiosh_core::distro_service::DistroStore::load_from_config(&cfg) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error loading distro store from config: {}", e);
                return 1;
            }
        }
    };

    match sub {
        Some("list") => {
            let profiles = store.list_profiles();
            classify_and_emit(
                &mut ctx,
                "distro",
                "list",
                json!({ "count": profiles.len() }),
                "success",
                None,
                Some("Listed distro profiles"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&profiles).unwrap_or_default());
            } else {
                println!("{:<30} {:<10} {:<10} {:<12} NAME", "ID", "FAMILY", "ARCH", "RECOMMENDED");
                println!("{}", "-".repeat(78));
                for p in &profiles {
                    println!("{:<30} {:<10?} {:<10?} {:<12} {}", p.id, p.family, p.arch, p.recommended, p.name);
                }
                println!("\nTotal distro profiles: {}", profiles.len());
            }
            0
        }
        Some("show") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh distro show <id> [--json] [--store <path>]");
                    return 2;
                }
            };
            match store.get_profile(id) {
                Some(p) => {
                    classify_and_emit(
                        &mut ctx,
                        "distro",
                        "show",
                        json!({ "id": p.id }),
                        "success",
                        Some(&p.id),
                        Some("Retrieved distro profile"),
                        "operator",
                        None,
                    );
                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
                    } else {
                        println!("Distribution Profile: {}", p.name);
                        println!("  ID:             {}", p.id);
                        println!("  Family:         {:?}", p.family);
                        println!("  Arch:           {:?}", p.arch);
                        println!("  Recommended:    {}", p.recommended);
                        println!("  Init System:    {:?}", p.init_system);
                        println!("  C Library:      {:?}", p.c_lib);
                        println!("  Min Kernel:     {}", p.min_kernel_version);
                        println!("  Packages:       {}", p.default_packages.join(", "));
                        println!("  Justification:  {}", p.justification);
                    }
                    0
                }
                None => {
                    eprintln!("Distro profile '{}' not found", id);
                    1
                }
            }
        }
        Some("evaluate") => {
            let id_opt = rest.first().filter(|s| !s.starts_with("--")).map(|s| s.as_str());
            if let Some(id) = id_opt {
                match store.evaluate_profile(id) {
                    Ok(ev) => {
                        classify_and_emit(
                            &mut ctx,
                            "distro",
                            "evaluate",
                            json!({ "id": ev.profile_id, "score": ev.overall_score }),
                            "success",
                            Some(&ev.profile_id),
                            Some("Evaluated distro profile"),
                            "operator",
                            None,
                        );
                        if is_json {
                            println!("{}", serde_json::to_string_pretty(&ev).unwrap_or_default());
                        } else {
                            println!("Evaluation for '{}' (Score: {:.2}):", ev.profile_id, ev.overall_score);
                            println!("  Binary Compatibility: {:.2}", ev.binary_compatibility_score);
                            println!("  Minimal Footprint:    {:.2}", ev.footprint_score);
                            println!("  Security Hardening:   {:.2}", ev.security_score);
                            println!("  Production Ready:     {}", ev.is_production_ready);
                        }
                        0
                    }
                    Err(e) => {
                        eprintln!("Evaluation error: {}", e);
                        1
                    }
                }
            } else {
                let evals = store.evaluate_all();
                classify_and_emit(
                    &mut ctx,
                    "distro",
                    "evaluate_all",
                    json!({ "count": evals.len() }),
                    "success",
                    None,
                    Some("Evaluated all distro profiles"),
                    "operator",
                    None,
                );
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&evals).unwrap_or_default());
                } else {
                    println!("{:<30} {:<14} EVALUATION SUMMARY", "PROFILE ID", "OVERALL SCORE");
                    println!("{}", "-".repeat(70));
                    for ev in &evals {
                        println!("{:<30} {:<14.2} compat={:.2} footprint={:.2} security={:.2} ready={}",
                            ev.profile_id, ev.overall_score, ev.binary_compatibility_score, ev.footprint_score, ev.security_score, ev.is_production_ready);
                    }
                }
                0
            }
        }
        Some("recommend") => {
            match store.get_recommended_profile() {
                Some(p) => {
                    classify_and_emit(
                        &mut ctx,
                        "distro",
                        "recommend",
                        json!({ "id": p.id }),
                        "success",
                        Some(&p.id),
                        Some("Retrieved recommended distro profile"),
                        "operator",
                        None,
                    );
                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
                    } else {
                        println!("Recommended Profile: {} ({})", p.name, p.id);
                        println!("Architecture: {:?} | Family: {:?}", p.arch, p.family);
                    }
                    0
                }
                None => {
                    eprintln!("No recommended distribution profile found");
                    1
                }
            }
        }
        Some("config") => {
            if is_json {
                println!("{}", serde_json::to_string_pretty(&cfg.to_json_with_sources()).unwrap_or_default());
            } else {
                println!("AIOS Distro Configuration:");
                println!("  Store Path:               {}", cfg.store_path);
                println!("  Pinned Reference ID:      {}", cfg.pinned_reference_id);
                println!("  Min Recommendation Score: {:.2}", cfg.min_recommendation_score);
                println!("  Auto Evaluate:            {}", cfg.auto_evaluate);
                println!("  Weights:                  binary={:.2}, security={:.2}, footprint={:.2}",
                    cfg.weights.binary_compatibility, cfg.weights.security, cfg.weights.footprint);
            }
            0
        }
        Some("policy") => {
            let policy = match aiosh_core::distro_policy::DistroSecurityPolicy::from_env() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error loading security policy: {}", e);
                    return 1;
                }
            };
            let id_opt = rest.first().filter(|s| !s.starts_with("--")).map(|s| s.as_str());
            if let Some(id) = id_opt {
                let profile = match store.get_profile(id) {
                    Some(p) => p,
                    None => {
                        eprintln!("Distro profile '{}' not found", id);
                        return 1;
                    }
                };
                let eval = match store.evaluate_profile(id) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Evaluation error: {}", e);
                        return 1;
                    }
                };
                let verdict = policy.check_profile(&profile, &eval);
                classify_and_emit(
                    &mut ctx,
                    "distro",
                    "policy",
                    json!({ "id": id, "allowed": verdict.allowed, "violations": verdict.violations }),
                    if verdict.allowed { "success" } else { "warning" },
                    Some(id),
                    Some("Checked distro profile security policy"),
                    "operator",
                    None,
                );
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&verdict).unwrap_or_default());
                } else {
                    println!("Policy Verdict for '{}': {}", verdict.profile_id, if verdict.allowed { "ALLOWED" } else { "REJECTED" });
                    if !verdict.violations.is_empty() {
                        println!("Violations:");
                        for v in &verdict.violations {
                            println!("  - {}", v);
                        }
                    }
                }
                if verdict.allowed { 0 } else { 1 }
            } else {
                let verdicts = store.check_security_policy(&policy);
                let allowed_count = verdicts.iter().filter(|v| v.allowed).count();
                classify_and_emit(
                    &mut ctx,
                    "distro",
                    "policy",
                    json!({ "total": verdicts.len(), "allowed": allowed_count }),
                    "success",
                    None,
                    Some("Checked security policy across all distro profiles"),
                    "operator",
                    None,
                );
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&verdicts).unwrap_or_default());
                } else {
                    println!("{:<30} {:<10} VIOLATIONS", "PROFILE ID", "STATUS");
                    println!("{}", "-".repeat(60));
                    for v in &verdicts {
                        let status = if v.allowed { "ALLOWED" } else { "REJECTED" };
                        let viol_summary = if v.violations.is_empty() {
                            "-".to_string()
                        } else {
                            v.violations.join("; ")
                        };
                        println!("{:<30} {:<10} {}", v.profile_id, status, viol_summary);
                    }
                    println!("\nCompliant profiles: {}/{}", allowed_count, verdicts.len());
                }
                0
            }
        }
        Some("stats") => {
            let policy_opt = aiosh_core::distro_policy::DistroSecurityPolicy::from_env().ok();
            let report = store.get_observability_report(policy_opt.as_ref());
            classify_and_emit(
                &mut ctx,
                "distro",
                "stats",
                json!({
                    "total": report.total_profiles,
                    "production_ready": report.production_ready_count,
                    "policy_compliant": report.policy_compliant_count,
                }),
                "success",
                None,
                Some("Retrieved distro observability report"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                println!("AIOS Distro Observability Report:");
                println!("  Total Profiles:            {}", report.total_profiles);
                println!("  Recommended Profile:       {}", report.recommended_profile_id.as_deref().unwrap_or("none"));
                println!("  Production Ready:          {}/{}", report.production_ready_count, report.total_profiles);
                println!("  Policy Compliant:          {}/{}", report.policy_compliant_count, report.total_profiles);
                println!("  Average Overall Score:     {:.2}", report.average_overall_score);
                println!("  Average Security Score:    {:.2}", report.average_security_score);
                println!("  Average Footprint Score:   {:.2}", report.average_footprint_score);
                println!("  Average Binary Compat:     {:.2}", report.average_binary_compatibility_score);
                println!("\nFamily Breakdown:");
                for (fam, count) in &report.family_breakdown {
                    println!("  {:<20} {}", fam, count);
                }
                println!("\nArchitecture Breakdown:");
                for (arch, count) in &report.architecture_breakdown {
                    println!("  {:<20} {}", arch, count);
                }
            }
            0
        }
        Some("check") => {
            let report = store.validate_health();
            classify_and_emit(
                &mut ctx,
                "distro",
                "check",
                json!({
                    "healthy": report.healthy,
                    "profile_count": report.profile_count,
                    "errors_count": report.errors.len(),
                }),
                if report.healthy { "success" } else { "failure" },
                None,
                Some("Validated distro store structural health"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                println!("AIOS Distro Store Health Check:");
                println!("  Status:                    {}", if report.healthy { "HEALTHY" } else { "UNHEALTHY" });
                println!("  Profile Count:             {}", report.profile_count);
                println!("  Recommended Profile Valid: {}", if report.recommended_profile_valid { "YES" } else { "NO" });
                if !report.errors.is_empty() {
                    println!("\nErrors Detected:");
                    for err in &report.errors {
                        println!("  - {}", err);
                    }
                }
            }
            if report.healthy { 0 } else { 1 }
        }
        Some("--help") | Some("-h") | None => {
            println!("aiosh distro — Linux Distro Selection & Justification Manager\n\nUsage:\n  aiosh distro list [--json] [--store <path>]\n  aiosh distro show <id> [--json] [--store <path>]\n  aiosh distro evaluate [<id>] [--json] [--store <path>]\n  aiosh distro recommend [--json] [--store <path>]\n  aiosh distro config [--json]\n  aiosh distro policy [<id>] [--json] [--store <path>]\n  aiosh distro stats [--json] [--store <path>]\n  aiosh distro check [--json] [--store <path>]");
            0
        }
        Some(other) => {
            eprintln!("unknown distro subcommand: {}", other);
            2
        }
    }
}

fn cmd_image(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    let is_json = has_flag(rest, "--json");

    let store = if let Some(path_str) = parse_flag(rest, "--store") {
        match aiosh_core::base_image_service::ImageStore::load_from_path(std::path::Path::new(&path_str)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to load image store from '{}': {}", path_str, e);
                return 1;
            }
        }
    } else {
        aiosh_core::base_image_service::ImageStore::new()
    };

    match sub {
        Some("list") => {
            let images = store.list_images();
            classify_and_emit(
                &mut ctx,
                "image",
                "list",
                json!({ "count": images.len() }),
                "success",
                None,
                Some("Enumerated registered base image manifests"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&images).unwrap_or_default());
            } else {
                println!("{:<32} {:<10} {:<10} {:<12} {}", "IMAGE ID", "FORMAT", "ARCH", "FILESYSTEM", "VERSION");
                for img in &images {
                    println!("{:<32} {:<10} {:<10} {:<12} {}", img.id, img.format, img.rootfs.architecture, img.rootfs.filesystem_type, img.version);
                }
            }
            0
        }
        Some("show") => {
            let id = match rest.first() {
                Some(s) if !s.starts_with("--") => s.as_str(),
                _ => {
                    eprintln!("missing required image id: aiosh image show <id>");
                    return 2;
                }
            };
            if !id.chars().all(|c| c.is_ascii_graphic()) || id.is_empty() {
                eprintln!("invalid image id: contains non-printable or control characters");
                return 2;
            }
            let manifest = match store.get_image(id) {
                Some(m) => m,
                None => {
                    eprintln!("image '{}' not found in store", id);
                    return 1;
                }
            };
            classify_and_emit(
                &mut ctx,
                "image",
                "show",
                json!({ "id": id, "format": manifest.format.to_string() }),
                "success",
                None,
                Some("Retrieved base image manifest"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(manifest).unwrap_or_default());
            } else {
                println!("AIOS Base Image Manifest: {}", manifest.id);
                println!("  Version:         {}", manifest.version);
                println!("  Format:          {}", manifest.format);
                println!("  Target Distro:   {}", manifest.rootfs.distro_id);
                println!("  Architecture:    {}", manifest.rootfs.architecture);
                println!("  Filesystem:      {}", manifest.rootfs.filesystem_type);
                println!("  Hostname:        {}", manifest.rootfs.hostname);
                println!("  Size Budget:     {} MB", manifest.rootfs.size_budget_bytes / (1024 * 1024));
                println!("  Kernel Version:  {}", manifest.kernel.version);
                println!("  Kernel Cmdline:  {}", manifest.kernel.cmdline);
                println!("  Initramfs Gen:   {}", manifest.kernel.initramfs_generator);
                println!("  Packages Count:  {}", manifest.rootfs.packages.len());
                println!("  Packages Sample: {}", manifest.rootfs.packages.iter().take(8).cloned().collect::<Vec<_>>().join(", "));
            }
            0
        }
        Some("plan") => {
            let id = match rest.first() {
                Some(s) if !s.starts_with("--") => s.as_str(),
                _ => {
                    eprintln!("missing required image id: aiosh image plan <id>");
                    return 2;
                }
            };
            if !id.chars().all(|c| c.is_ascii_graphic()) || id.is_empty() {
                eprintln!("invalid image id: contains non-printable or control characters");
                return 2;
            }
            let plan = match store.generate_build_plan(id) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("failed to generate build plan for '{}': {}", id, e);
                    return 1;
                }
            };
            classify_and_emit(
                &mut ctx,
                "image",
                "plan",
                json!({ "id": id, "stages_count": plan.stages.len(), "estimated_duration_secs": plan.estimated_total_duration_secs }),
                "success",
                None,
                Some("Generated base image build plan"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&plan).unwrap_or_default());
            } else {
                println!("AIOS Build Execution Plan for '{}':", plan.image_id);
                println!("  Target Format:      {}", plan.target_format);
                println!("  Estimated Duration: {}s", plan.estimated_total_duration_secs);
                println!("  Estimated Size:     {} MB", plan.estimated_artifact_size_bytes / (1024 * 1024));
                println!("\nDiscrete Stages ({}):", plan.stages.len());
                for (i, stage) in plan.stages.iter().enumerate() {
                    println!("  [{}] {} (approx {}s):", i + 1, stage.name, stage.estimated_duration_secs);
                    println!("      Description: {}", stage.description);
                    println!("      Command:     {}", stage.command_template);
                }
            }
            0
        }
        Some("filter") => {
            let mut matches = store.list_images();
            if let Some(fmt_str) = parse_flag(rest, "--format") {
                let fmt = match fmt_str.to_lowercase().as_str() {
                    "raw" => aiosh_core::base_image::ImageFormat::Raw,
                    "qcow2" => aiosh_core::base_image::ImageFormat::Qcow2,
                    "iso" => aiosh_core::base_image::ImageFormat::Iso,
                    "tarball" | "tar" => aiosh_core::base_image::ImageFormat::Tarball,
                    other => {
                        eprintln!("unknown image format: {}", other);
                        return 2;
                    }
                };
                matches.retain(|m| m.format == fmt);
            }
            if let Some(distro) = parse_flag(rest, "--distro") {
                matches.retain(|m| m.rootfs.distro_id == distro);
            }
            classify_and_emit(
                &mut ctx,
                "image",
                "filter",
                json!({ "matched_count": matches.len() }),
                "success",
                None,
                Some("Filtered registered base image manifests"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&matches).unwrap_or_default());
            } else {
                println!("Matched Base Images ({}):", matches.len());
                for img in &matches {
                    println!("  {:<32} {:<10} {:<10} {}", img.id, img.format, img.rootfs.architecture, img.rootfs.distro_id);
                }
            }
            0
        }
        Some("config") => {
            let config = if let Some(cfg_path_str) = parse_flag(rest, "--config") {
                match aiosh_core::base_image_config::ImageBuildConfig::from_file(std::path::Path::new(&cfg_path_str)) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("failed to load image config from '{}': {}", cfg_path_str, e);
                        return 1;
                    }
                }
            } else {
                match aiosh_core::base_image_config::ImageBuildConfig::from_env() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("failed to load image config from environment: {}", e);
                        return 1;
                    }
                }
            };
            classify_and_emit(
                &mut ctx,
                "image",
                "config",
                json!({ "default_target": config.default_target, "timeout": config.max_build_duration_secs }),
                "success",
                None,
                Some("Inspected base image configuration"),
                "operator",
                None,
            );
            if is_json {
                println!("{}", serde_json::to_string_pretty(&config).unwrap_or_default());
            } else {
                println!("AIOS Base Image Build Configuration:");
                println!("  Build Directory:    {}", config.build_dir.display());
                println!("  Output Directory:   {}", config.output_dir.display());
                println!("  Default Target:     {}", config.default_target);
                println!("  Max Duration:       {}s", config.max_build_duration_secs);
                println!("  Max Artifact Size:  {} MB", config.max_artifact_size_bytes / (1024 * 1024));
                println!("  Compression Level:  {}", config.compression_level);
            }
            0
        }
        Some("policy") => {
            let policy = match aiosh_core::base_image_policy::BaseImageSecurityPolicy::from_env() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("failed to load image security policy: {}", e);
                    return 1;
                }
            };
            let id_opt = rest.first().filter(|s| !s.starts_with("--")).map(|s| s.as_str());
            if let Some(id) = id_opt {
                if !id.chars().all(|c| c.is_ascii_graphic()) || id.is_empty() {
                    eprintln!("invalid image id: contains non-printable or control characters");
                    return 2;
                }
                let manifest = match store.get_image(id) {
                    Some(m) => m,
                    None => {
                        eprintln!("image '{}' not found in store", id);
                        return 1;
                    }
                };
                let verdict = policy.evaluate(manifest);
                classify_and_emit(
                    &mut ctx,
                    "image",
                    "policy",
                    json!({ "id": id, "allowed": verdict.allowed, "violations_count": verdict.violations.len() }),
                    if verdict.allowed { "success" } else { "failure" },
                    None,
                    Some("Evaluated base image security policy"),
                    "operator",
                    None,
                );
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&verdict).unwrap_or_default());
                } else {
                    println!("Base Image Policy Verdict for '{}':", id);
                    println!("  Allowed:    {}", verdict.allowed);
                    println!("  Mode:       {:?}", verdict.mode);
                    println!("  Violations: {}", verdict.violations.len());
                    for v in &verdict.violations {
                        println!("    - [{}] {} (fatal: {})", v.rule_id, v.description, v.fatal);
                    }
                }
                if verdict.allowed { 0 } else { 1 }
            } else {
                let verdicts = policy.check_all(&store);
                classify_and_emit(
                    &mut ctx,
                    "image",
                    "policy",
                    json!({ "evaluated_count": verdicts.len() }),
                    "success",
                    None,
                    Some("Evaluated base image security policy across store"),
                    "operator",
                    None,
                );
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&verdicts).unwrap_or_default());
                } else {
                    println!("Base Image Security Policy (mode: {:?}):", policy.mode);
                    for v in &verdicts {
                        let status_str = if v.allowed { "PASS" } else { "FAIL" };
                        println!("  [{}] {:<32} (violations: {})", status_str, v.manifest_id, v.violations.len());
                    }
                }
                0
            }
        }
        Some("--help") | Some("-h") | None => {
            println!("aiosh image — Linux Base Image Build & Packaging Manager\n\nUsage:\n  aiosh image list [--json] [--store <path>]\n  aiosh image show <id> [--json] [--store <path>]\n  aiosh image plan <id> [--json] [--store <path>]\n  aiosh image filter [--format <format>] [--distro <id>] [--json] [--store <path>]\n  aiosh image config [--json] [--config <path>]\n  aiosh image policy [<id>] [--json] [--store <path>]");
            0
        }
        Some(other) => {
            eprintln!("unknown image subcommand: {}", other);
            2
        }
    }
}

fn cmd_handoff(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let config = if let Some(cfg_path_str) = parse_flag(rest, "--config") {
        match aiosh_core::handoff_config::HandoffConfig::from_file(std::path::Path::new(&cfg_path_str)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading handoff config: {}", e);
                return 1;
            }
        }
    } else {
        aiosh_core::handoff_config::HandoffConfig::from_env_or_default()
    };

    let store_path_str = parse_flag(rest, "--store")
        .or_else(|| config.store_path.clone())
        .unwrap_or_else(|| format!("{}/handoff_store.json", ai_home()));
    let store_path = std::path::Path::new(&store_path_str);

    let (mut store, _recovery_warning) = aiosh_core::handoff_service::HandoffStore::load_or_recover_with_config(store_path, &config);

    match sub {
        Some("list") => {
            let active_only = has_flag(rest, "--active");
            let status_filter = parse_flag(rest, "--status");

            let records = if active_only {
                store.list_active()
            } else {
                store.list_all()
            };

            let filtered: Vec<_> = records
                .into_iter()
                .filter(|r| {
                    if let Some(ref st) = status_filter {
                        let st_str = format!("{:?}", r.status).to_lowercase();
                        st_str == st.to_lowercase()
                    } else {
                        true
                    }
                })
                .collect();

            if is_json {
                println!("{}", serde_json::to_string_pretty(&filtered).unwrap_or_default());
            } else {
                println!("{:<14} {:<12} {:<16} {:<16} {:<10} SUMMARY", "ID", "STATUS", "SENDER", "RECEIVER", "PRIORITY");
                println!("{}", "-".repeat(80));
                for r in &filtered {
                    println!(
                        "{:<14} {:<12?} {:<16} {:<16} {:<10?} {}",
                        r.id, r.status, r.sender_agent_id, r.receiver_agent_id, r.priority, r.context_summary
                    );
                }
                println!("\nTotal handoffs listed: {}", filtered.len());
            }
            0
        }
        Some("show") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh handoff show <id> [--json] [--store <path>]");
                    return 2;
                }
            };

            match store.get_by_id(id) {
                Some(r) => {
                    if is_json {
                        println!("{}", serde_json::to_string_pretty(r).unwrap_or_default());
                    } else {
                        println!("Handoff Record: {}", r.id);
                        println!("Signature:      {}", r.signature);
                        println!("Status:         {:?}", r.status);
                        println!("Priority:       {:?}", r.priority);
                        println!("Sender:         {}", r.sender_agent_id);
                        println!("Receiver:       {}", r.receiver_agent_id);
                        if let Some(t) = r.task_id {
                            println!("Task ID:        {}", t);
                        }
                        println!("Created At:     {}", r.created_at);
                        if let Some(ref exp) = r.expires_at {
                            println!("Expires At:     {}", exp);
                        }
                        if let Some(ref notes) = r.resolution_notes {
                            println!("Notes:          {}", notes);
                        }
                        println!("Summary:        {}", r.context_summary);
                        println!("Payload:        {}", r.payload_json);
                    }
                    0
                }
                None => {
                    eprintln!("Error: Handoff record '{}' not found", id);
                    1
                }
            }
        }
        Some("initiate") => {
            let sender = match parse_flag(rest, "--sender") {
                Some(s) => s,
                None => {
                    eprintln!("Error: --sender is required for aiosh handoff initiate");
                    return 2;
                }
            };
            let receiver = match parse_flag(rest, "--receiver") {
                Some(r) => r,
                None => {
                    eprintln!("Error: --receiver is required for aiosh handoff initiate");
                    return 2;
                }
            };
            let summary = match parse_flag(rest, "--summary") {
                Some(s) => s,
                None => {
                    eprintln!("Error: --summary is required for aiosh handoff initiate");
                    return 2;
                }
            };
            let task_id = parse_flag(rest, "--task").and_then(|t| t.parse::<u32>().ok());
            let payload = parse_flag(rest, "--payload").unwrap_or_else(|| "{}".into());
            let priority = match parse_flag(rest, "--priority").as_deref() {
                Some("low") => aiosh_core::handoff::HandoffPriority::Low,
                Some("high") => aiosh_core::handoff::HandoffPriority::High,
                Some("urgent") => aiosh_core::handoff::HandoffPriority::Urgent,
                _ => aiosh_core::handoff::HandoffPriority::Normal,
            };

            let rec = store.initiate_handoff(&sender, &receiver, task_id, &summary, &payload, priority);

            if let Err(e) = store.save_to_path(store_path) {
                eprintln!("Error saving store: {}", e);
                return 1;
            }

            classify_and_emit(
                &mut ctx,
                "handoff",
                "initiate",
                json!({ "id": rec.id, "sender": sender, "receiver": receiver }),
                "success",
                Some(&rec.id),
                Some("Handoff initiated"),
                "operator",
                None,
            );

            if is_json {
                println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());
            } else {
                println!("Initiated handoff: {} (status: {:?})", rec.id, rec.status);
            }
            0
        }
        Some("accept") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh handoff accept <id> [--notes <notes>] [--store <path>]");
                    return 2;
                }
            };
            let notes = parse_flag(rest, "--notes");

            match store.accept_handoff(id, notes.as_deref()) {
                Ok(rec) => {
                    if let Err(e) = store.save_to_path(store_path) {
                        eprintln!("Error saving store: {}", e);
                        return 1;
                    }

                    classify_and_emit(
                        &mut ctx,
                        "handoff",
                        "accept",
                        json!({ "id": rec.id, "receiver": rec.receiver_agent_id }),
                        "success",
                        Some(&rec.id),
                        Some("Handoff accepted"),
                        "operator",
                        None,
                    );

                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());
                    } else {
                        println!("Accepted handoff: {} (status: {:?})", rec.id, rec.status);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error accepting handoff: {}", e);
                    1
                }
            }
        }
        Some("reject") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh handoff reject <id> [--notes <notes>] [--store <path>]");
                    return 2;
                }
            };
            let notes = parse_flag(rest, "--notes");

            match store.reject_handoff(id, notes.as_deref()) {
                Ok(rec) => {
                    if let Err(e) = store.save_to_path(store_path) {
                        eprintln!("Error saving store: {}", e);
                        return 1;
                    }

                    classify_and_emit(
                        &mut ctx,
                        "handoff",
                        "reject",
                        json!({ "id": rec.id, "receiver": rec.receiver_agent_id }),
                        "success",
                        Some(&rec.id),
                        Some("Handoff rejected"),
                        "operator",
                        None,
                    );

                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());
                    } else {
                        println!("Rejected handoff: {} (status: {:?})", rec.id, rec.status);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error rejecting handoff: {}", e);
                    1
                }
            }
        }
        Some("complete") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh handoff complete <id> [--notes <notes>] [--store <path>]");
                    return 2;
                }
            };
            let notes = parse_flag(rest, "--notes");

            match store.complete_handoff(id, notes.as_deref()) {
                Ok(rec) => {
                    if let Err(e) = store.save_to_path(store_path) {
                        eprintln!("Error saving store: {}", e);
                        return 1;
                    }

                    classify_and_emit(
                        &mut ctx,
                        "handoff",
                        "complete",
                        json!({ "id": rec.id, "receiver": rec.receiver_agent_id }),
                        "success",
                        Some(&rec.id),
                        Some("Handoff completed"),
                        "operator",
                        None,
                    );

                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());
                    } else {
                        println!("Completed handoff: {} (status: {:?})", rec.id, rec.status);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error completing handoff: {}", e);
                    1
                }
            }
        }
        Some("cancel") => {
            let id = match rest.first() {
                Some(id_str) if !id_str.starts_with("--") => id_str.as_str(),
                _ => {
                    eprintln!("Usage: aiosh handoff cancel <id> [--notes <notes>] [--store <path>]");
                    return 2;
                }
            };
            let notes = parse_flag(rest, "--notes");

            match store.cancel_handoff(id, notes.as_deref()) {
                Ok(rec) => {
                    if let Err(e) = store.save_to_path(store_path) {
                        eprintln!("Error saving store: {}", e);
                        return 1;
                    }

                    classify_and_emit(
                        &mut ctx,
                        "handoff",
                        "cancel",
                        json!({ "id": rec.id, "sender": rec.sender_agent_id }),
                        "success",
                        Some(&rec.id),
                        Some("Handoff cancelled"),
                        "operator",
                        None,
                    );

                    if is_json {
                        println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());
                    } else {
                        println!("Cancelled handoff: {} (status: {:?})", rec.id, rec.status);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error cancelling handoff: {}", e);
                    1
                }
            }
        }
        Some("--help") | Some("-h") | None => {
            println!("aiosh handoff — Agent Handoff Protocol Manager\n\nUsage:\n  aiosh handoff list [--active] [--status <status>] [--json] [--store <path>]\n  aiosh handoff show <id> [--json] [--store <path>]\n  aiosh handoff initiate --sender <S> --receiver <R> [--task <T>] --summary <CTX> [--payload <JSON>] [--priority <P>] [--store <path>]\n  aiosh handoff accept <id> [--notes <notes>] [--store <path>]\n  aiosh handoff reject <id> [--notes <notes>] [--store <path>]\n  aiosh handoff complete <id> [--notes <notes>] [--store <path>]\n  aiosh handoff cancel <id> [--notes <notes>] [--store <path>]");
            0
        }
        Some(other) => {
            eprintln!("unknown handoff subcommand: {}", other);
            2
        }
    }
}

fn cmd_triage(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let config = if let Some(cfg_path_str) = parse_flag(rest, "--config") {
        match aiosh_core::triage_config::TriageConfig::from_file(std::path::Path::new(&cfg_path_str)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading triage config: {}", e);
                return 1;
            }
        }
    } else {
        aiosh_core::triage_config::TriageConfig::from_env_or_default()
    };

    let store_path_str = parse_flag(rest, "--store")
        .or_else(|| config.store_path.clone())
        .unwrap_or_else(|| format!("{}/triage_store.json", ai_home()));
    let store_path = std::path::Path::new(&store_path_str);

    let mut store = match aiosh_core::triage_service::TriageStore::load_from_path_with_config(store_path, &config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading triage store: {}", e);
            return 1;
        }
    };

    match sub {
        Some("list") => {
            let status_filter = parse_flag(rest, "--status");
            let severity_filter = parse_flag(rest, "--severity");

            let report = store.to_report();
            let filtered: Vec<_> = report.records.into_iter().filter(|r| {
                if let Some(ref st) = status_filter {
                    let st_str = match r.status {
                        aiosh_core::triage::TriageStatus::Untriaged => "untriaged",
                        aiosh_core::triage::TriageStatus::Triaged => "triaged",
                        aiosh_core::triage::TriageStatus::FixPending => "fix_pending",
                        aiosh_core::triage::TriageStatus::Resolved => "resolved",
                        aiosh_core::triage::TriageStatus::WontFix => "wont_fix",
                    };
                    if st_str != st.to_lowercase() {
                        return false;
                    }
                }
                if let Some(ref sv) = severity_filter {
                    let sv_str = match r.severity {
                        aiosh_core::triage::TriageSeverity::Blocker => "blocker",
                        aiosh_core::triage::TriageSeverity::Critical => "critical",
                        aiosh_core::triage::TriageSeverity::Major => "major",
                        aiosh_core::triage::TriageSeverity::Minor => "minor",
                    };
                    if sv_str != sv.to_lowercase() {
                        return false;
                    }
                }
                true
            }).collect();

            if is_json {
                println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
            } else {
                println!("=== AIOS Regression Triage Records ({}) ===", filtered.len());
                for r in &filtered {
                    println!("[{}] {:?} {:?} - {} ({}) occ:{}", r.id, r.severity, r.status, r.test_target, r.suite_name, r.occurrences);
                    println!("    error: {}", r.error_message.lines().next().unwrap_or(""));
                    if let Some(ref n) = r.resolution_notes {
                        println!("    notes: {}", n);
                    }
                }
            }
            0
        }
        Some("show") => {
            let id = match rest.first() {
                Some(i) => i,
                None => {
                    eprintln!("Usage: aiosh triage show <id> [--store <path>] [--json]");
                    return 2;
                }
            };
            if let Some(rec) = store.get_by_id(id) {
                if is_json {
                    println!("{}", serde_json::to_string_pretty(rec).unwrap());
                } else {
                    println!("Triage ID:       {}", rec.id);
                    println!("Signature:       {}", rec.signature);
                    println!("Status:          {:?}", rec.status);
                    println!("Severity:        {:?}", rec.severity);
                    println!("Test Target:     {}", rec.test_target);
                    println!("Suite Name:      {}", rec.suite_name);
                    println!("Occurrences:     {}", rec.occurrences);
                    println!("First Observed:  {}", rec.first_observed_at);
                    println!("Last Observed:   {}", rec.last_observed_at);
                    println!("Repro Command:   {}", rec.repro_command);
                    println!("Error Message:\n{}", rec.error_message);
                    if let Some(ref n) = rec.resolution_notes {
                        println!("Resolution Notes: {}", n);
                    }
                }
                0
            } else {
                eprintln!("Record {} not found", id);
                1
            }
        }
        Some("record") => {
            let target = match parse_flag(rest, "--target") {
                Some(t) => t,
                None => {
                    eprintln!("Missing required option --target");
                    return 2;
                }
            };
            let suite = parse_flag(rest, "--suite").unwrap_or_else(|| "manual".into());
            let error_msg = match parse_flag(rest, "--error") {
                Some(e) => e,
                None => {
                    eprintln!("Missing required option --error");
                    return 2;
                }
            };
            let repro = parse_flag(rest, "--repro").unwrap_or_else(|| "".into());
            let sev = match parse_flag(rest, "--severity").as_deref() {
                Some("blocker") => aiosh_core::triage::TriageSeverity::Blocker,
                Some("major") => aiosh_core::triage::TriageSeverity::Major,
                Some("minor") => aiosh_core::triage::TriageSeverity::Minor,
                _ => aiosh_core::triage::TriageSeverity::Critical,
            };

            let rec = store.record_failure(&target, &suite, &error_msg, &repro, sev);
            if let Err(e) = store.save_to_path(store_path) {
                eprintln!("Failed to save triage store: {}", e);
                return 1;
            }

            classify_and_emit(
                &mut ctx,
                "triage",
                "record",
                json!({ "id": rec.id, "target": target, "suite": suite }),
                "success",
                Some(&rec.id),
                Some("Regression recorded"),
                "operator",
                None,
            );

            if is_json {
                println!("{}", serde_json::to_string_pretty(&rec).unwrap());
            } else {
                println!("Recorded triage item {} (occurrences: {})", rec.id, rec.occurrences);
            }
            0
        }
        Some("resolve") => {
            let id = match rest.first() {
                Some(i) => i,
                None => {
                    eprintln!("Usage: aiosh triage resolve <id> --notes <text> [--store <path>]");
                    return 2;
                }
            };
            let notes = match parse_flag(rest, "--notes") {
                Some(n) => n,
                None => {
                    eprintln!("Missing required option --notes");
                    return 2;
                }
            };

            let rec = match store.resolve(id, &notes) {
                Ok(r) => r.clone(),
                Err(e) => {
                    eprintln!("Resolve error: {}", e);
                    return 1;
                }
            };

            if let Err(e) = store.save_to_path(store_path) {
                eprintln!("Failed to save triage store: {}", e);
                return 1;
            }

            classify_and_emit(
                &mut ctx,
                "triage",
                "resolve",
                json!({ "id": id, "notes": notes }),
                "success",
                Some(id),
                Some("Regression resolved"),
                "operator",
                None,
            );

            if is_json {
                println!("{}", serde_json::to_string_pretty(&rec).unwrap());
            } else {
                println!("Resolved triage item {}", rec.id);
            }
            0
        }
        Some("ingest") => {
            let file_path_str = match rest.first() {
                Some(f) => f,
                None => {
                    eprintln!("Usage: aiosh triage ingest <summary_json_file> [--store <path>]");
                    return 2;
                }
            };
            let summary_path = std::path::Path::new(file_path_str);
            let summary = match aiosh_core::ci::load_summary_with_retry(summary_path, 2) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to load CI summary: {}", e);
                    return 1;
                }
            };

            let count = store.ingest_ci_summary_with_config(&summary, &config);
            if let Err(e) = store.save_to_path(store_path) {
                eprintln!("Failed to save triage store: {}", e);
                return 1;
            }

            classify_and_emit(
                &mut ctx,
                "triage",
                "ingest",
                json!({ "source": file_path_str, "processed": count }),
                "success",
                Some(file_path_str),
                Some(&format!("Ingested {} regression candidates", count)),
                "operator",
                None,
            );

            if is_json {
                println!("{}", json!({ "processed": count, "store": store_path_str }));
            } else {
                println!("Ingested {} regression candidates into {}", count, store_path_str);
            }
            0
        }
        Some("check") => {
            let report = store.to_report();
            let mut blocker_count = 0;
            let mut critical_count = 0;

            for r in &report.records {
                if r.status != aiosh_core::triage::TriageStatus::Resolved && r.status != aiosh_core::triage::TriageStatus::WontFix {
                    match r.severity {
                        aiosh_core::triage::TriageSeverity::Blocker => blocker_count += 1,
                        aiosh_core::triage::TriageSeverity::Critical => critical_count += 1,
                        _ => {}
                    }
                }
            }

            let ok = blocker_count == 0 && critical_count == 0;
            if is_json {
                println!("{}", json!({
                    "clean": ok,
                    "total_records": report.total_records,
                    "open_records": report.open_records,
                    "blocker_open": blocker_count,
                    "critical_open": critical_count
                }));
            } else {
                println!("=== AIOS Triage Check ===");
                println!("Total records:   {}", report.total_records);
                println!("Open records:    {}", report.open_records);
                println!("Blocker open:    {}", blocker_count);
                println!("Critical open:   {}", critical_count);
                if ok {
                    println!("\nPASS: No open blocker or critical regressions.");
                } else {
                    println!("\nFAIL: Found {} blocker and {} critical open regressions.", blocker_count, critical_count);
                }
            }
            if ok { 0 } else { 1 }
        }
        Some("--help") | Some("-h") | None => {
            println!("Usage: aiosh triage <list|show|record|resolve|ingest|check> [options]\n\nSubcommands:\n  list [--status <st>] [--severity <sev>] [--json] [--store <path>]\n  show <id> [--json] [--store <path>]\n  record --target <target> --suite <suite> --error <msg> [--repro <cmd>] [--severity <sev>] [--store <path>]\n  resolve <id> --notes <notes> [--store <path>]\n  ingest <summary_file> [--store <path>]\n  check [--store <path>] [--json]");
            0
        }
        Some(other) => {
            eprintln!("unknown triage subcommand: {}", other);
            2
        }
    }
}

fn cmd_secrets(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let repo_flag = parse_flag(rest, "--repo");
    let file_flag = parse_flag(rest, "--file");
    let config_flag = parse_flag(rest, "--config");

    let config = match config_flag {
        Some(ref cp) => match aiosh_core::secrets_config::SecretsConfig::from_path(std::path::Path::new(cp)) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to read secrets config: {}", e);
                if is_json {
                    return err_out(json!({"ok": false, "subcommand": "secrets", "error": msg}));
                } else {
                    eprintln!("[-] {}", msg);
                    return 1;
                }
            }
        },
        None => aiosh_core::secrets_config::SecretsConfig::from_env().unwrap_or_default(),
    };

    let max_bytes: u64 = parse_flag(rest, "--max-bytes")
        .and_then(|s| s.parse().ok())
        .unwrap_or(config.max_file_bytes);

    let repo_root = repo_flag
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));

    match sub {
        Some("scan") | Some("check") => {
            let is_check = sub == Some("check");
            let report = if let Some(ref file_path_str) = file_flag {
                let target_file = std::path::Path::new(file_path_str);
                let findings = match aiosh_core::secrets_service::scan_file_for_secrets(target_file, repo_root, max_bytes) {
                    Ok(f) => f,
                    Err(e) => {
                        let msg = format!("Failed to scan file for secrets: {}", e);
                        emit(
                            &mut ctx,
                            "secrets.scan",
                            &format!("aiosh secrets scan --file {}", file_path_str),
                            json!({"file": file_path_str}),
                            "error",
                            Some(file_path_str),
                            Some(&msg),
                            "user",
                            None,
                            CFlags::default(),
                            None,
                        );
                        if is_json {
                            return err_out(json!({"ok": false, "subcommand": "secrets scan", "error": msg}));
                        } else {
                            eprintln!("[-] {}", msg);
                            return 1;
                        }
                    }
                };
                aiosh_core::secrets::SecretScanReport::new(
                    target_file.to_string_lossy().to_string(),
                    findings,
                    1,
                )
            } else {
                let ignored_slice: Vec<&str> = config.ignored_dirs.iter().map(|s| s.as_str()).collect();
                match aiosh_core::secrets_service::scan_workspace_for_secrets(
                    repo_root,
                    max_bytes,
                    &ignored_slice,
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        let msg = format!("Failed to scan workspace for secrets: {}", e);
                        emit(
                            &mut ctx,
                            "secrets.scan",
                            "aiosh secrets scan",
                            json!({"repo": repo_root.to_string_lossy()}),
                            "error",
                            None,
                            Some(&msg),
                            "user",
                            None,
                            CFlags::default(),
                            None,
                        );
                        if is_json {
                            return err_out(json!({"ok": false, "subcommand": "secrets scan", "error": msg}));
                        } else {
                            eprintln!("[-] {}", msg);
                            return 1;
                        }
                    }
                }
            };

            let outcome = if report.is_clean { "ok" } else { "failure" };
            emit(
                &mut ctx,
                if is_check { "secrets.check" } else { "secrets.scan" },
                &format!("aiosh secrets {}", if is_check { "check" } else { "scan" }),
                json!({
                    "repo": repo_root.to_string_lossy(),
                    "is_clean": report.is_clean,
                    "total_findings": report.total_findings,
                    "scanned_files_count": report.scanned_files_count,
                }),
                outcome,
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                if report.is_clean {
                    ok_out(json!({"ok": true, "subcommand": format!("secrets {}", if is_check { "check" } else { "scan" }), "data": report}));
                    0
                } else {
                    err_out(json!({"ok": false, "subcommand": format!("secrets {}", if is_check { "check" } else { "scan" }), "data": report}));
                    1
                }
            } else if is_check {
                if report.is_clean {
                    println!("[+] Secrets check passed (0 findings in {} files).", report.scanned_files_count);
                    0
                } else {
                    eprintln!("[-] Secrets check failed: {} findings detected across {} files in {}.", report.total_findings, report.scanned_files_count, report.repo_path);
                    1
                }
            } else {
                println!("=== Secrets & Access Hygiene Scan: {} ===", report.repo_path);
                println!("Timestamp: {}", report.timestamp_utc);
                println!(
                    "Status: {} ({} files scanned, {} findings: {} critical, {} high, {} medium, {} low)",
                    if report.is_clean { "CLEAN" } else { "FINDINGS DETECTED" },
                    report.scanned_files_count,
                    report.total_findings,
                    report.critical_findings,
                    report.high_findings,
                    report.medium_findings,
                    report.low_findings
                );
                println!();

                if !report.is_clean {
                    println!("Findings:");
                    for f in &report.findings {
                        println!(
                            "  - [!] {} ({:?}) {}:{} - {}",
                            f.rule_id, f.severity, f.path, f.line_number, f.description
                        );
                        let fp_short = if f.fingerprint.len() >= 8 { &f.fingerprint[..8] } else { &f.fingerprint };
                        println!("      Snippet: {} [fp: {}]", f.redacted_snippet, fp_short);
                    }
                }

                if report.is_clean { 0 } else { 1 }
            }
        }
        Some(other) => {
            eprintln!("unknown secrets subcommand: {} (usage: aiosh secrets <scan|check> [--repo <path>] [--file <path>] [--json])", other);
            2
        }
        None => {
            eprintln!("missing secrets subcommand (usage: aiosh secrets <scan|check> [--repo <path>] [--file <path>] [--json])");
            2
        }
    }
}

fn cmd_repo(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let repo_flag = parse_flag(rest, "--repo");
    let config_flag = parse_flag(rest, "--config");
    let repo_root = repo_flag
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));

    let _config = match config_flag {
        Some(ref p) => match aiosh_core::repo_health_config::RepoHealthConfig::from_path(std::path::Path::new(p)) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to load repo health config: {}", e);
                if is_json {
                    return err_out(json!({"ok": false, "subcommand": "repo health", "error": msg}));
                } else {
                    eprintln!("[-] {}", msg);
                    return 1;
                }
            }
        },
        None => aiosh_core::repo_health_config::RepoHealthConfig::from_env().unwrap_or_default(),
    };

    match sub {
        Some("health") | Some("check") => {
            let report = match aiosh_core::repo_health_service::check_repo_health(repo_root) {
                Ok(r) => r,
                Err(e) => {
                    let msg = format!("Failed to assess repository health: {}", e);
                    emit(
                        &mut ctx,
                        "repo.health",
                        "aiosh repo health",
                        json!({"repo": repo_root.to_string_lossy()}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        return err_out(json!({"ok": false, "subcommand": "repo health", "error": msg}));
                    } else {
                        eprintln!("[-] {}", msg);
                        return 1;
                    }
                }
            };

            let is_fail = report.overall_status == aiosh_core::repo_health::HealthStatus::Fail;
            let outcome = if is_fail { "error" } else { "ok" };

            emit(
                &mut ctx,
                "repo.health",
                "aiosh repo health",
                json!({
                    "repo": repo_root.to_string_lossy(),
                    "overall_status": report.overall_status,
                    "total_checks": report.total_checks,
                    "failed_checks": report.failed_checks,
                }),
                outcome,
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                if is_fail {
                    err_out(json!({"ok": false, "subcommand": "repo health", "data": report}));
                    return 1;
                } else {
                    ok_out(json!({"ok": true, "subcommand": "repo health", "data": report}));
                    return 0;
                }
            } else {
                println!("=== Repository Health Assessment: {} ===", report.repo_path);
                println!("Timestamp: {}", report.timestamp_utc);
                println!(
                    "Overall Status: {:?} ({} checks: {} pass, {} warn, {} fail, {} skip)",
                    report.overall_status,
                    report.total_checks,
                    report.passed_checks,
                    report.warn_checks,
                    report.failed_checks,
                    report.skipped_checks
                );
                println!();

                for c in &report.checks {
                    let symbol = match c.status {
                        aiosh_core::repo_health::HealthStatus::Pass => "[+]",
                        aiosh_core::repo_health::HealthStatus::Warn => "[!]",
                        aiosh_core::repo_health::HealthStatus::Fail => "[-]",
                        aiosh_core::repo_health::HealthStatus::Skip => "[*]",
                    };
                    println!("{} {} ({}, {:?}) - {}ms", symbol, c.name, c.check_id, c.category, c.duration_ms);
                    println!("    {}", c.message);
                    if let Some(ref details) = c.details {
                        for d in details {
                            println!("      * {}", d);
                        }
                    }
                }

                if is_fail { 1 } else { 0 }
            }
        }
        Some(other) => {
            eprintln!("unknown repo subcommand: {} (usage: aiosh repo <health|check> [--repo <path>] [--json])", other);
            2
        }
        None => {
            eprintln!("missing repo subcommand (usage: aiosh repo <health|check> [--repo <path>] [--json])");
            2
        }
    }
}

fn cmd_doc(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let repo_flag = parse_flag(rest, "--repo");
    let config_flag = parse_flag(rest, "--config");
    let repo_root = repo_flag
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));

    let _config = match config_flag {
        Some(ref p) => match aiosh_core::doc_index_config::DocIndexConfig::from_path(std::path::Path::new(p)) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to load doc index config: {}", e);
                if is_json {
                    return err_out(json!({"ok": false, "subcommand": "doc", "error": msg}));
                } else {
                    eprintln!("[-] {}", msg);
                    return 1;
                }
            }
        },
        None => aiosh_core::doc_index_config::DocIndexConfig::from_env().unwrap_or_default(),
    };

    let default_docs = &["docs/README.md", "docs/SPEC-TASK-LEDGER.md", "docs/tasks/GOALS.md"];

    match sub {
        Some("show") => {
            let manifest = match aiosh_core::doc_index_service::build_doc_index_from_paths(repo_root, default_docs) {
                Ok(m) => m,
                Err(e) => {
                    let msg = format!("Failed to build doc index: {}", e);
                    emit(
                        &mut ctx,
                        "doc.show",
                        "aiosh doc show",
                        json!({"repo": repo_root.to_string_lossy()}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        return err_out(json!({"ok": false, "subcommand": "doc show", "error": msg}));
                    } else {
                        eprintln!("[-] {}", msg);
                        return 1;
                    }
                }
            };

            emit(
                &mut ctx,
                "doc.show",
                "aiosh doc show",
                json!({"repo": repo_root.to_string_lossy(), "count": manifest.entries.len()}),
                "ok",
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                ok_out(json!({"ok": true, "subcommand": "doc show", "data": manifest}));
            } else {
                println!("{}", aiosh_core::doc_index_service::format_doc_index_summary(&manifest));
            }
            0
        }
        Some("check") => {
            let (_manifest, report, telemetry) = match aiosh_core::doc_index_service::reconcile_doc_index(repo_root, default_docs) {
                Ok(t) => t,
                Err(e) => {
                    let msg = format!("Failed to read doc files: {}", e);
                    emit(
                        &mut ctx,
                        "doc.check",
                        "aiosh doc check",
                        json!({"repo": repo_root.to_string_lossy()}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        return err_out(json!({"ok": false, "subcommand": "doc check", "error": msg}));
                    } else {
                        eprintln!("[-] {}", msg);
                        return 1;
                    }
                }
            };

            let outcome = if report.is_valid { "ok" } else { "failure" };
            emit(
                &mut ctx,
                "doc.check",
                "aiosh doc check",
                json!({"repo": repo_root.to_string_lossy(), "report": report, "telemetry": telemetry}),
                outcome,
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                if report.is_valid {
                    ok_out(json!({"ok": true, "subcommand": "doc check", "data": report}));
                    0
                } else {
                    err_out(json!({"ok": false, "subcommand": "doc check", "data": report}));
                    1
                }
            } else {
                if report.is_valid {
                    println!("[+] Documentation link verification passed ({} links checked)", report.total_links_checked);
                    0
                } else {
                    eprintln!("[-] Broken links detected ({} links checked, {} broken):", report.total_links_checked, report.broken_links.len());
                    for b in &report.broken_links {
                        eprintln!("    - {} -> {} ({})", b.source_path, b.target_link, b.reason);
                    }
                    1
                }
            }
        }
        Some("search") => {
            let positional = strip_flags(rest, &["--json", "--repo"]);
            let query = match positional.first() {
                Some(q) => q.to_lowercase(),
                None => {
                    eprintln!("usage: aiosh doc search <query> [--json] [--repo <path>]");
                    return 2;
                }
            };

            let manifest = match aiosh_core::doc_index_service::build_doc_index_from_paths(repo_root, default_docs) {
                Ok(m) => m,
                Err(e) => {
                    let msg = format!("Failed to read doc files: {}", e);
                    if is_json {
                        return err_out(json!({"ok": false, "subcommand": "doc search", "error": msg}));
                    } else {
                        eprintln!("[-] {}", msg);
                        return 1;
                    }
                }
            };

            let matches: Vec<_> = manifest.entries.into_iter().filter(|e| {
                e.title.to_lowercase().contains(&query) ||
                e.path.to_lowercase().contains(&query) ||
                e.section.to_lowercase().contains(&query)
            }).collect();

            emit(
                &mut ctx,
                "doc.search",
                "aiosh doc search",
                json!({"query": query, "matches_count": matches.len()}),
                "ok",
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                ok_out(json!({"ok": true, "subcommand": "doc search", "data": matches}));
            } else {
                println!("Documentation search results for '{}':", query);
                for entry in &matches {
                    println!("  [{}] {} ({})", entry.section, entry.title, entry.path);
                }
            }
            0
        }
        _ => {
            eprintln!("usage: aiosh doc <show|check|search> [--json] [--repo <path>]");
            2
        }
    }
}

fn cmd_evidence(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    let rest = if args.len() > 1 { &args[1..] } else { &[] };

    let is_json = has_flag(rest, "--json");
    let repo_flag = parse_flag(rest, "--repo");
    let manifest_flag = parse_flag(rest, "--manifest");
    let repo_root = repo_flag
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));

    match sub {
        Some("verify") => {
            let manifest = match manifest_flag {
                Some(ref p) => {
                    let content = match std::fs::read_to_string(p) {
                        Ok(c) => c,
                        Err(e) => {
                            let msg = format!("Failed to read manifest file {}: {}", p, e);
                            if is_json {
                                return err_out(json!({"ok": false, "subcommand": "evidence verify", "error": msg}));
                            } else {
                                eprintln!("[-] {}", msg);
                                return 1;
                            }
                        }
                    };
                    match aiosh_core::evidence::TaskEvidenceManifest::from_json(&content) {
                        Ok(m) => m,
                        Err(e) => {
                            let msg = format!("Failed to parse evidence manifest: {}", e);
                            if is_json {
                                return err_out(json!({"ok": false, "subcommand": "evidence verify", "error": msg}));
                            } else {
                                eprintln!("[-] {}", msg);
                                return 1;
                            }
                        }
                    }
                }
                None => aiosh_core::evidence::TaskEvidenceManifest::default(),
            };

            let report = match aiosh_core::evidence_service::verify_evidence_manifest(repo_root, &manifest) {
                Ok(r) => r,
                Err(e) => {
                    let msg = format!("Failed to verify evidence manifest: {}", e);
                    emit(
                        &mut ctx,
                        "evidence.verify",
                        "aiosh evidence verify",
                        json!({"repo": repo_root.to_string_lossy()}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        return err_out(json!({"ok": false, "subcommand": "evidence verify", "error": msg}));
                    } else {
                        eprintln!("[-] {}", msg);
                        return 1;
                    }
                }
            };

            let outcome = if report.is_valid { "ok" } else { "failure" };
            emit(
                &mut ctx,
                "evidence.verify",
                "aiosh evidence verify",
                json!({"repo": repo_root.to_string_lossy(), "report": report}),
                outcome,
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                if report.is_valid {
                    ok_out(json!({"ok": true, "subcommand": "evidence verify", "data": report}));
                } else {
                    err_out(json!({"ok": false, "subcommand": "evidence verify", "error": "Evidence verification failed", "report": report}));
                }
            } else {
                if report.is_valid {
                    println!("[+] All {} evidence records verified successfully (SHA-256 match).", report.total_records);
                } else {
                    eprintln!("[-] Evidence verification failed: {}/{} valid.", report.valid_records, report.total_records);
                    for m in &report.missing_files {
                        eprintln!("    - Missing: {}", m);
                    }
                    for h in &report.hash_mismatches {
                        eprintln!("    - Mismatch: {}", h);
                    }
                    return 1;
                }
            }
            0
        }
        Some("hash") => {
            let positional = strip_flags(rest, &["--json"]);
            let path_str = match positional.first() {
                Some(p) => p,
                None => {
                    eprintln!("usage: aiosh evidence hash <path> [--json]");
                    return 2;
                }
            };
            let target_path = std::path::Path::new(path_str);
            match aiosh_core::evidence_service::compute_file_sha256(target_path) {
                Ok(hash) => {
                    emit(
                        &mut ctx,
                        "evidence.hash",
                        "aiosh evidence hash",
                        json!({"path": path_str, "sha256": hash}),
                        "ok",
                        None,
                        None,
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        ok_out(json!({"ok": true, "subcommand": "evidence hash", "path": path_str, "sha256": hash}));
                    } else {
                        println!("[+] {} -> {}", path_str, hash);
                    }
                    0
                }
                Err(e) => {
                    let msg = format!("Failed to compute SHA-256 for {}: {}", path_str, e);
                    emit(
                        &mut ctx,
                        "evidence.hash",
                        "aiosh evidence hash",
                        json!({"path": path_str}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    if is_json {
                        err_out(json!({"ok": false, "subcommand": "evidence hash", "error": msg}))
                    } else {
                        eprintln!("[-] {}", msg);
                        1
                    }
                }
            }
        }
        Some("scan") => {
            let task_filter = parse_flag(rest, "--task").and_then(|s| s.parse::<u32>().ok());
            let evidence_dir = repo_root.join("docs/tasks/evidence");
            if !evidence_dir.exists() {
                let msg = format!("Evidence directory not found: {}", evidence_dir.display());
                if is_json {
                    return err_out(json!({"ok": false, "subcommand": "evidence scan", "error": msg}));
                } else {
                    eprintln!("[-] {}", msg);
                    return 1;
                }
            }

            let mut records = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&evidence_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if file_name.starts_with('T') && file_name.contains('-') {
                            let parts: Vec<&str> = file_name.split('-').collect();
                            if parts.len() >= 2 {
                                if let Ok(tid) = parts[1].parse::<u32>() {
                                    if task_filter.map_or(true, |target| target == tid) {
                                        let rel_path = format!("docs/tasks/evidence/{}", file_name);
                                        if let Ok(hash) = aiosh_core::evidence_service::compute_file_sha256(&path) {
                                            records.push(json!({
                                                "task_id": tid,
                                                "file_path": rel_path,
                                                "sha256": hash
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            emit(
                &mut ctx,
                "evidence.scan",
                "aiosh evidence scan",
                json!({"repo": repo_root.to_string_lossy(), "count": records.len()}),
                "ok",
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );

            if is_json {
                ok_out(json!({"ok": true, "subcommand": "evidence scan", "data": records}));
            } else {
                println!("Scanned {} evidence files in {}", records.len(), evidence_dir.display());
                for r in &records {
                    println!("  [T-{:05}] {} ({})", r["task_id"].as_u64().unwrap_or(0), r["file_path"].as_str().unwrap_or(""), &r["sha256"].as_str().unwrap_or("")[..8]);
                }
            }
            0
        }
        _ => {
            eprintln!("usage: aiosh evidence <verify|hash|scan> [--json] [--repo <path>] [--manifest <path>] [--task <id>]");
            2
        }
    }
}

fn cmd_release(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    match sub {
        Some("generate") => {
            let os = match parse_flag(args, "--os") {
                Some(v) => v,
                None => {
                    eprintln!("usage: aiosh release generate --os <target_os> --version <version> [--components <c1,c2...>]");
                    return 2;
                }
            };
            let version = match parse_flag(args, "--version") {
                Some(v) => v,
                None => {
                    eprintln!("usage: aiosh release generate --os <target_os> --version <version> [--components <c1,c2...>]");
                    return 2;
                }
            };
            let comp_str = parse_flag(args, "--components").unwrap_or_else(|| "core".into());
            let components: Vec<String> = comp_str.split(',').map(|s| s.trim().to_string()).collect();
            
            let manifest = aiosh_core::release::PackageManifest {
                target_os: os,
                version,
                components,
            };
            
            let mut rel_ctx = aiosh_core::release::ReleaseCtx {
                ring: &mut ctx.ring,
                actor_id: &ctx.actor_id,
                constitution_rev: &ctx.con_rev,
            };
            
            match aiosh_core::release::generate_release(&mut rel_ctx, &manifest) {
                Ok((path, hash)) => {
                    ok_out(json!({"ok": true, "subcommand": "release generate", "data": {"artifact_path": path, "hash": hash}}));
                    0
                }
                Err(e) => {
                    err_out(json!({"ok": false, "subcommand": "release generate", "error": e}));
                    1
                }
            }
        }
        _ => {
            eprintln!("usage: aiosh release generate --os <target_os> --version <version> [--components <c1,c2...>]");
            2
        }
    }
}

fn cmd_toolchain(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());

    // Parse optional --config <path> from remaining args
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    let mut config_path: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--config" {
            if i + 1 >= rest.len() {
                eprintln!("usage: aiosh toolchain <check|show> [--config <path>]");
                return 2;
            }
            config_path = Some(rest[i + 1].clone());
            i += 2;
        } else {
            eprintln!("unknown flag: {}", rest[i]);
            eprintln!("usage: aiosh toolchain <check|show> [--config <path>]");
            return 2;
        }
    }

    // Resolve manifest using --config flag, env var, or default path
    let load_manifest = || -> Result<aiosh_core::toolchain_config::ToolchainManifest, String> {
        match &config_path {
            Some(p) => aiosh_core::toolchain_config::ToolchainManifest::from_path(p),
            None => aiosh_core::toolchain_config::ToolchainManifest::from_env(),
        }
    };

    match sub {
        Some("check") => {
            let manifest = match load_manifest() {
                Ok(m) => m,
                Err(e) => {
                    let msg = format!("Failed to load toolchain config: {}", e);
                    err_out(json!({"ok": false, "subcommand": "toolchain check", "error": msg.clone()}));
                    emit(
                        &mut ctx,
                        "toolchain.check",
                        "aiosh toolchain check",
                        json!({}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    return 1;
                }
            };

            match aiosh_core::toolchain_service::enforce_toolchain(&manifest) {
                Ok(_) => {
                    let data = manifest.to_json_with_sources();
                    ok_out(json!({"ok": true, "subcommand": "toolchain check", "data": data}));
                    emit(
                        &mut ctx,
                        "toolchain.check",
                        "aiosh toolchain check",
                        json!({"manifest": data}),
                        "success",
                        None,
                        None,
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    0
                }
                Err(e) => {
                    err_out(json!({"ok": false, "subcommand": "toolchain check", "error": e}));
                    emit(
                        &mut ctx,
                        "toolchain.check",
                        "aiosh toolchain check",
                        json!({}),
                        "error",
                        None,
                        Some(&e),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    1
                }
            }
        }
        Some("show") => {
            let manifest = match load_manifest() {
                Ok(m) => m,
                Err(e) => {
                    let msg = format!("Failed to load toolchain config: {}", e);
                    err_out(json!({"ok": false, "subcommand": "toolchain show", "error": msg.clone()}));
                    emit(
                        &mut ctx,
                        "toolchain.show",
                        "aiosh toolchain show",
                        json!({}),
                        "error",
                        None,
                        Some(&msg),
                        "user",
                        None,
                        CFlags::default(),
                        None,
                    );
                    return 1;
                }
            };

            let data = manifest.to_json_with_sources();
            ok_out(json!({"ok": true, "subcommand": "toolchain show", "data": data}));
            emit(
                &mut ctx,
                "toolchain.show",
                "aiosh toolchain show",
                json!({"manifest": data}),
                "success",
                None,
                None,
                "user",
                None,
                CFlags::default(),
                None,
            );
            0
        }
        _ => {
            eprintln!("usage: aiosh toolchain <check|show> [--config <path>]");
            2
        }
    }
}

fn cmd_backup(args: &[String]) -> i32 {
    let mut ctx = open_context();
    let sub = args.first().map(|s| s.as_str());
    match sub {
        Some("create") => {
            let target_path = match parse_flag(args, "--target-path") {
                Some(t) => t,
                None => {
                    eprintln!("usage: aiosh backup create --target-path <path> [--include-audit <true|false>] [--include-memory <true|false>]");
                    return 2;
                }
            };
            let include_audit = match parse_flag(args, "--include-audit").as_deref() {
                Some("false") => false,
                _ => true,
            };
            let include_memory = match parse_flag(args, "--include-memory").as_deref() {
                Some("true") => true,
                _ => false,
            };
            
            let snapshot = aiosh_core::release::BackupSnapshot {
                target_path,
                include_audit,
                include_memory,
            };
            
            let mut rel_ctx = aiosh_core::release::ReleaseCtx {
                ring: &mut ctx.ring,
                actor_id: &ctx.actor_id,
                constitution_rev: &ctx.con_rev,
            };
            
            match aiosh_core::release::create_backup(&mut rel_ctx, &snapshot) {
                Ok(path) => {
                    ok_out(json!({"ok": true, "subcommand": "backup create", "data": {"backup_path": path}}));
                    0
                }
                Err(e) => {
                    err_out(json!({"ok": false, "subcommand": "backup create", "error": e}));
                    1
                }
            }
        }
        _ => {
            eprintln!("usage: aiosh backup create --target-path <path> [--include-audit <true|false>] [--include-memory <true|false>]");
            2
        }
    }
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
                CFlags::default(),
                None,
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
                CFlags::default(),
                None,
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
    let conn = match if db_path == ":memory:" {
        rusqlite::Connection::open_in_memory()
    } else {
        rusqlite::Connection::open(&db_path)
    } {
        Ok(c) => c,
        Err(e) => {
            eprintln!("audit rotate failed to open database: {e}");
            return 1;
        }
    };
    let res = match retention::rotate(
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
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("audit rotate operation failed: {e}");
            return 1;
        }
    };
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

    #[test]
    fn test_cmd_doc_show_check_and_search() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let code_show = cmd_doc(&["show".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert_eq!(code_show, 0);

        let code_show_json = cmd_doc(&["show".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert_eq!(code_show_json, 0);

        let code_check = cmd_doc(&["check".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert_eq!(code_check, 0);

        let code_check_json = cmd_doc(&["check".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert_eq!(code_check_json, 0);

        let code_search = cmd_doc(&["search".to_string(), "task".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert_eq!(code_search, 0);

        let code_invalid = cmd_doc(&["invalid_subcommand".to_string()]);
        assert_eq!(code_invalid, 2);
    }

    #[test]
    fn test_cmd_repo_health_and_check() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let code_health = cmd_repo(&["health".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert!(code_health == 0 || code_health == 1);

        let code_health_json = cmd_repo(&["health".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert!(code_health_json == 0 || code_health_json == 1);

        let code_check = cmd_repo(&["check".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert!(code_check == 0 || code_check == 1);

        let code_check_json = cmd_repo(&["check".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert!(code_check_json == 0 || code_check_json == 1);

        let code_invalid = cmd_repo(&["invalid_subcommand".to_string()]);
        assert_eq!(code_invalid, 2);
    }

    #[test]
    fn test_cmd_secrets_scan_and_check() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let code_scan = cmd_secrets(&["scan".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert!(code_scan == 0 || code_scan == 1);

        let code_scan_json = cmd_secrets(&["scan".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert!(code_scan_json == 0 || code_scan_json == 1);

        let code_check = cmd_secrets(&["check".to_string(), "--repo".to_string(), repo_str.clone()]);
        assert!(code_check == 0 || code_check == 1);

        let code_check_json = cmd_secrets(&["check".to_string(), "--repo".to_string(), repo_str.clone(), "--json".to_string()]);
        assert!(code_check_json == 0 || code_check_json == 1);

        let config_path = repo_root.join("docs/secrets_config.json").to_string_lossy().to_string();
        let code_config_scan = cmd_secrets(&["scan".to_string(), "--repo".to_string(), repo_str.clone(), "--config".to_string(), config_path]);
        assert!(code_config_scan == 0 || code_config_scan == 1);

        let code_invalid = cmd_secrets(&["invalid_subcommand".to_string()]);
        assert_eq!(code_invalid, 2);
    }

    #[test]
    fn test_cmd_triage_flow() {
        let store_file = std::env::temp_dir().join(format!("aios_triage_test_{}.json", std::process::id()));
        let store_str = store_file.to_string_lossy().to_string();

        let _ = std::fs::remove_file(&store_file);

        let code_list_empty = cmd_triage(&["list".to_string(), "--store".to_string(), store_str.clone()]);
        assert_eq!(code_list_empty, 0);

        let code_record = cmd_triage(&[
            "record".to_string(),
            "--target".to_string(),
            "secrets::tests::test_scan".to_string(),
            "--suite".to_string(),
            "secrets_suite".to_string(),
            "--error".to_string(),
            "panicked at assertion".to_string(),
            "--store".to_string(),
            store_str.clone(),
        ]);
        assert_eq!(code_record, 0);

        let code_check_fail = cmd_triage(&["check".to_string(), "--store".to_string(), store_str.clone()]);
        assert_eq!(code_check_fail, 1);

        let code_list = cmd_triage(&["list".to_string(), "--store".to_string(), store_str.clone(), "--json".to_string()]);
        assert_eq!(code_list, 0);

        let config_file = std::env::temp_dir().join(format!("aios_triage_cfg_test_{}.json", std::process::id()));
        let cfg = aiosh_core::triage_config::TriageConfig::default();
        cfg.save_to_file(&config_file).unwrap();

        let code_config_list = cmd_triage(&["list".to_string(), "--config".to_string(), config_file.to_string_lossy().to_string(), "--store".to_string(), store_str.clone()]);
        assert_eq!(code_config_list, 0);

        let code_invalid = cmd_triage(&["invalid_subcommand".to_string()]);
        assert_eq!(code_invalid, 2);

        let _ = std::fs::remove_file(&config_file);
        let _ = std::fs::remove_file(&store_file);
    }

    #[test]
    fn test_cmd_handoff_flow() {
        let store_file = std::env::temp_dir().join(format!("aios_handoff_test_{}.json", std::process::id()));
        let store_str = store_file.to_string_lossy().to_string();

        let _ = std::fs::remove_file(&store_file);

        // List empty
        let code_list_empty = cmd_handoff(&["list".to_string(), "--store".to_string(), store_str.clone()]);
        assert_eq!(code_list_empty, 0);

        // Initiate
        let code_initiate = cmd_handoff(&[
            "initiate".to_string(),
            "--sender".to_string(),
            "operator".to_string(),
            "--receiver".to_string(),
            "subagent-1".to_string(),
            "--summary".to_string(),
            "Review task execution".to_string(),
            "--priority".to_string(),
            "high".to_string(),
            "--store".to_string(),
            store_str.clone(),
        ]);
        assert_eq!(code_initiate, 0);

        // List JSON
        let code_list_json = cmd_handoff(&["list".to_string(), "--store".to_string(), store_str.clone(), "--json".to_string()]);
        assert_eq!(code_list_json, 0);

        // Accept
        let code_accept = cmd_handoff(&[
            "accept".to_string(),
            "HND-".to_string(),
            "--notes".to_string(),
            "Accepted".to_string(),
            "--store".to_string(),
            store_str.clone(),
        ]);
        // Invalid ID returns 1, valid sub returns 0/1
        assert!(code_accept == 0 || code_accept == 1);

        // Help
        let code_help = cmd_handoff(&["--help".to_string()]);
        assert_eq!(code_help, 0);

        // Invalid
        let code_invalid = cmd_handoff(&["invalid_sub".to_string()]);
        assert_eq!(code_invalid, 2);

        let _ = std::fs::remove_file(&store_file);
    }

    #[test]
    fn test_cmd_distro_flow() {
        let code_list = cmd_distro(&["list".to_string()]);
        assert_eq!(code_list, 0);

        let code_list_json = cmd_distro(&["list".to_string(), "--json".to_string()]);
        assert_eq!(code_list_json, 0);

        let code_show = cmd_distro(&["show".to_string(), "debian-12-minimal-x86_64".to_string()]);
        assert_eq!(code_show, 0);

        let code_show_missing = cmd_distro(&["show".to_string(), "nonexistent".to_string()]);
        assert_eq!(code_show_missing, 1);

        let code_eval = cmd_distro(&["evaluate".to_string()]);
        assert_eq!(code_eval, 0);

        let code_eval_single = cmd_distro(&["evaluate".to_string(), "alpine-319-container-x86_64".to_string()]);
        assert_eq!(code_eval_single, 0);

        let code_rec = cmd_distro(&["recommend".to_string()]);
        assert_eq!(code_rec, 0);

        let code_help = cmd_distro(&["--help".to_string()]);
        assert_eq!(code_help, 0);

        let code_invalid = cmd_distro(&["invalid_sub".to_string()]);
        assert_eq!(code_invalid, 2);
    }

    #[test]
    fn test_cmd_image_flow() {
        let code_list = cmd_image(&["list".to_string()]);
        assert_eq!(code_list, 0);

        let code_list_json = cmd_image(&["list".to_string(), "--json".to_string()]);
        assert_eq!(code_list_json, 0);

        let code_show = cmd_image(&["show".to_string(), "debian-12-minimal-raw".to_string()]);
        assert_eq!(code_show, 0);

        let code_show_missing = cmd_image(&["show".to_string(), "nonexistent".to_string()]);
        assert_eq!(code_show_missing, 1);

        let code_show_no_arg = cmd_image(&["show".to_string()]);
        assert_eq!(code_show_no_arg, 2);

        let code_show_control_char = cmd_image(&["show".to_string(), "bad\x07id".to_string()]);
        assert_eq!(code_show_control_char, 2);

        let code_plan = cmd_image(&["plan".to_string(), "debian-12-minimal-raw".to_string()]);
        assert_eq!(code_plan, 0);

        let code_plan_missing = cmd_image(&["plan".to_string(), "nonexistent".to_string()]);
        assert_eq!(code_plan_missing, 1);

        let code_plan_no_arg = cmd_image(&["plan".to_string()]);
        assert_eq!(code_plan_no_arg, 2);

        let code_plan_control_char = cmd_image(&["plan".to_string(), "bad\x07id".to_string()]);
        assert_eq!(code_plan_control_char, 2);

        let code_filter = cmd_image(&["filter".to_string(), "--format".to_string(), "iso".to_string()]);
        assert_eq!(code_filter, 0);

        let code_filter_bad_format = cmd_image(&["filter".to_string(), "--format".to_string(), "bad_fmt".to_string()]);
        assert_eq!(code_filter_bad_format, 2);

        let code_config = cmd_image(&["config".to_string()]);
        assert_eq!(code_config, 0);

        let code_config_json = cmd_image(&["config".to_string(), "--json".to_string()]);
        assert_eq!(code_config_json, 0);

        let code_policy_all = cmd_image(&["policy".to_string()]);
        assert_eq!(code_policy_all, 0);

        let code_policy_all_json = cmd_image(&["policy".to_string(), "--json".to_string()]);
        assert_eq!(code_policy_all_json, 0);

        let code_policy_single = cmd_image(&["policy".to_string(), "debian-12-minimal-raw".to_string()]);
        assert_eq!(code_policy_single, 0);

        let code_policy_missing = cmd_image(&["policy".to_string(), "nonexistent-img".to_string()]);
        assert_eq!(code_policy_missing, 1);

        let code_policy_bad_id = cmd_image(&["policy".to_string(), "bad\x07id".to_string()]);
        assert_eq!(code_policy_bad_id, 2);

        let code_help = cmd_image(&["--help".to_string()]);
        assert_eq!(code_help, 0);

        let code_invalid = cmd_image(&["invalid_sub".to_string()]);
        assert_eq!(code_invalid, 2);
    }
}
