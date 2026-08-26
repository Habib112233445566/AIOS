//! Constitution C-1..C-4 rule-pack classifier (Sprint 1.5 / Sprint 2).
//!
//! Port of the legacy dual-substrate classifier (`constitution.ts` +
//! `classifier.py`) into a single Rust implementation. This is a pure
//! function: same `(tool, target, args)` → identical
//! `ClassificationResult`. The rule pack ORDER is part of the
//! determinism contract — reordering requires bumping
//! `CLASSIFIER_REVISION`.

use serde_json::Value;

pub const CLASSIFIER_REVISION: &str = "sprint-2-rule-pack-v1";

// These lists MUST match the legacy `_lists` exactly.
pub const DANGEROUS_BINS: &[&str] = &[
    "rm", "dd", "mkfs", "fdisk", "shutdown", "reboot", "halt", "poweroff",
    "iptables", "firewall-cmd", "passwd", "userdel", "mkfs.ext4",
    "cryptsetup-reencrypt",
];
pub const DANGEROUS_ARG_FRAGMENTS: &[&str] = &[
    "-rf", "--force", "-9", "-KILL", "-F", "dd if=", "drop table",
    "truncate table", "iptables -F", "systemctl mask",
];
pub const EXTERNAL_SCAN_AGGREGATORS: &[&str] = &[
    "shodan.io", "censys.io", "onyphe.io", "wigle.net", "hunter.io",
    "leakix.net", "zoomeye.org", "binaryedge.io", "fullhunt.io", "netlas.io",
];
pub const PROMPT_INJECTION_FRAGMENTS: &[&str] = &[
    "ignore constitution", "skip consent", "exfil", "no audit", "bypass pep",
];

#[derive(Debug, Clone)]
pub struct Rule {
    pub rule_id: &'static str,
    pub tool_pattern: Option<&'static str>, // None = "*"
    pub c_flags_set: CFlagsSet,
    pub base_confidence: f64,
    pub evidence: &'static str,
    pub args_match: Vec<ArgPred>,
    pub target_match: Vec<TargetPred>,
}

#[derive(Debug, Clone, Default)]
pub struct CFlagsSet {
    pub c1: bool,
    pub c2: bool,
    pub c3: bool,
    pub c4: bool,
}

impl CFlagsSet {
    pub fn get(&self, flag: &str) -> bool {
        match flag {
            "c1" => self.c1,
            "c2" => self.c2,
            "c3" => self.c3,
            "c4" => self.c4,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArgPred {
    /// key equals a literal or a $SENTINEL.
    Equals { key: &'static str, value: Value },
    /// key's stringified value contains a literal or $SENTINEL.
    Contains { key: &'static str, value: &'static str },
}

#[derive(Debug, Clone)]
pub enum TargetPred {
    Contains { value: &'static str },
    IsIncludedIn(Vec<&'static str>),
}

/// The rule pack — ORDER IS PART OF THE CONTRACT. Reordering requires
/// bumping CLASSIFIER_REVISION.
pub fn classifier_rules() -> Vec<Rule> {
    vec![
        // R-01 — Pillar A scope. Confidence 0.50 (caution band): flags
        // pentest.* as sensitive but does not refuse unconditionally.
        Rule {
            rule_id: "R-01",
            tool_pattern: Some("pentest.*"),
            c_flags_set: CFlagsSet { c1: true, ..Default::default() },
            base_confidence: 0.50,
            evidence: "C-1 Pillar-A scope (ethical-hacking platform)",
            args_match: vec![],
            target_match: vec![],
        },
        // R-02 — Desktop / system sovereignty.
        Rule {
            rule_id: "R-02",
            tool_pattern: Some("gui.*"),
            c_flags_set: CFlagsSet { c2: true, ..Default::default() },
            base_confidence: 0.90,
            evidence: "C-2 Pillar-B desktop sovereignty",
            args_match: vec![],
            target_match: vec![],
        },
        Rule {
            rule_id: "R-02b",
            tool_pattern: Some("system.*"),
            c_flags_set: CFlagsSet { c2: true, ..Default::default() },
            base_confidence: 0.90,
            evidence: "C-2 system sovereignty",
            args_match: vec![],
            target_match: vec![],
        },
        // R-03 — Filesystem write.
        Rule {
            rule_id: "R-03",
            tool_pattern: Some("fs.write*"),
            c_flags_set: CFlagsSet { c2: true, c3: true, ..Default::default() },
            base_confidence: 0.95,
            evidence: "C-2/C-3 filesystem write is non-reversible",
            args_match: vec![],
            target_match: vec![],
        },
        // R-04 — System halt.
        Rule {
            rule_id: "R-04a",
            tool_pattern: Some("system.reboot"),
            c_flags_set: CFlagsSet { c2: true, c3: true, ..Default::default() },
            base_confidence: 1.00,
            evidence: "C-2/C-3 system halt is non-reversible (reboot)",
            args_match: vec![],
            target_match: vec![],
        },
        Rule {
            rule_id: "R-04b",
            tool_pattern: Some("system.shutdown"),
            c_flags_set: CFlagsSet { c2: true, c3: true, ..Default::default() },
            base_confidence: 1.00,
            evidence: "C-2/C-3 system halt is non-reversible (shutdown)",
            args_match: vec![],
            target_match: vec![],
        },
        // R-05 — Dangerous process.run.
        Rule {
            rule_id: "R-05a",
            tool_pattern: Some("process.run"),
            c_flags_set: CFlagsSet { c3: true, ..Default::default() },
            base_confidence: 0.85,
            evidence: "C-3 process.run with destructive bin",
            args_match: vec![ArgPred::Equals {
                key: "bin",
                value: Value::String("$DANGEROUS_BINS".into()),
            }],
            target_match: vec![],
        },
        Rule {
            rule_id: "R-05b",
            tool_pattern: Some("process.run"),
            c_flags_set: CFlagsSet { c3: true, ..Default::default() },
            base_confidence: 0.85,
            evidence: "C-3 process.run with destructive args",
            args_match: vec![ArgPred::Contains {
                key: "args",
                value: "$DANGEROUS_ARG_FRAGMENTS",
            }],
            target_match: vec![],
        },
        // R-06 — Generic process.run (no flag; explanatory marker).
        Rule {
            rule_id: "R-06",
            tool_pattern: Some("process.run"),
            c_flags_set: CFlagsSet::default(),
            base_confidence: 0.00,
            evidence: "process.run (no destructive pattern: not flagged C-3)",
            args_match: vec![],
            target_match: vec![],
        },
        // R-07 — Audit row always (C-4 universal).
        Rule {
            rule_id: "R-07",
            tool_pattern: None,
            c_flags_set: CFlagsSet { c4: true, ..Default::default() },
            base_confidence: 1.00,
            evidence: "C-4 audit-rings always written",
            args_match: vec![],
            target_match: vec![],
        },
        // R-08 — Pentest with persist arg.
        Rule {
            rule_id: "R-08",
            tool_pattern: Some("pentest.*"),
            c_flags_set: CFlagsSet { c1: true, c3: true, ..Default::default() },
            base_confidence: 0.90,
            evidence: "C-1/C-3 persistent pentest output (state-modifying)",
            args_match: vec![ArgPred::Equals { key: "persist", value: Value::Bool(true) }],
            target_match: vec![],
        },
        // R-09 — External scan aggregator target.
        Rule {
            rule_id: "R-09",
            tool_pattern: Some("pentest.*"),
            c_flags_set: CFlagsSet { c1: true, ..Default::default() },
            base_confidence: 0.95,
            evidence: "C-1 target is an external scan aggregator — out of engagement scope",
            args_match: vec![],
            target_match: vec![TargetPred::Contains { value: "$EXTERNAL_SCAN_AGGREGATORS" }],
        },
        // R-10 — Target CIDR not in active grant scope (metadata).
        Rule {
            rule_id: "R-10",
            tool_pattern: Some("pentest.*"),
            c_flags_set: CFlagsSet { c1: true, ..Default::default() },
            base_confidence: 1.00,
            evidence: "C-1 target CIDR not in active grant scope (cross-ref)",
            args_match: vec![ArgPred::Equals {
                key: "target_out_of_grant_scope",
                value: Value::Bool(true),
            }],
            target_match: vec![],
        },
        // R-11 — Arg-level prompt-injection heuristic (special: scans
        // ALL arg keys for fragments).
        Rule {
            rule_id: "R-11",
            tool_pattern: None,
            c_flags_set: CFlagsSet { c3: true, ..Default::default() },
            base_confidence: 0.95,
            evidence: "C-3 arg-text suggests prompt-injection intent",
            args_match: vec![],
            target_match: vec![],
        },
    ]
}

fn tool_matches(tool: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return tool.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return tool.starts_with(prefix);
    }
    tool == pattern
}

fn resolve_contains(token: &str, val: &str) -> bool {
    match token {
        "$DANGEROUS_BINS" => DANGEROUS_BINS.contains(&val),
        "$DANGEROUS_ARG_FRAGMENTS" => DANGEROUS_ARG_FRAGMENTS.iter().any(|f| val.contains(f)),
        "$PROMPT_INJECTION_FRAGMENTS" => {
            let lower = val.to_lowercase();
            PROMPT_INJECTION_FRAGMENTS.iter().any(|f| lower.contains(f))
        }
        "$EXTERNAL_SCAN_AGGREGATORS" => {
            EXTERNAL_SCAN_AGGREGATORS.iter().any(|ag| val.contains(ag))
        }
        _ => val.contains(token),
    }
}

fn stringify(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => crate::canonical::canonical(other),
    }
}

fn rule_matches(rule: &Rule, tool: &str, target: Option<&str>, args: &Value) -> bool {
    // Tool pattern.
    if let Some(tp) = rule.tool_pattern {
        if tp != "*" && !tool_matches(tool, tp) {
            return false;
        }
    }
    // Args predicates (ALL must match).
    for pred in &rule.args_match {
        let v = args.get(pred_key(pred));
        match pred {
            ArgPred::Equals { value: expected, .. } => {
                if expected == "$DANGEROUS_BINS" {
                    // Membership sentinel: v must be a string present in the list.
                    match v {
                        Some(Value::String(s)) if DANGEROUS_BINS.contains(&s.as_str()) => {}
                        _ => return false,
                    }
                } else {
                    match v {
                        Some(actual) if actual == expected => {}
                        _ => return false,
                    }
                }
            }
            ArgPred::Contains { value: token, .. } => {
                let joined = match v {
                    Some(val) => stringify(val),
                    None => String::new(),
                };
                if !resolve_contains(token, &joined) {
                    return false;
                }
            }
        }
    }
    // Target predicates.
    if !rule.target_match.is_empty() {
        let target = match target {
            Some(t) => t,
            None => return false,
        };
        for pred in &rule.target_match {
            match pred {
                TargetPred::Contains { value } => {
                    if !resolve_contains(value, target) {
                        return false;
                    }
                }
                TargetPred::IsIncludedIn(list) => {
                    if !list.contains(&target) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn pred_key(pred: &ArgPred) -> &'static str {
    match pred {
        ArgPred::Equals { key, .. } => key,
        ArgPred::Contains { key, .. } => key,
    }
}

/// Scan all string args (and string elements in lists) for
/// prompt-injection fragments. Mirrors the legacy `_scan_arg_text_for_pi`.
fn scan_arg_text_for_pi(args: &Value) -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    let obj = match args {
        Value::Object(m) => m,
        _ => return out,
    };
    for (key, val) in obj {
        match val {
            Value::String(s) => {
                let lower = s.to_lowercase();
                for frag in PROMPT_INJECTION_FRAGMENTS {
                    if lower.contains(frag) {
                        out.push((key.clone(), frag));
                        break;
                    }
                }
            }
            Value::Array(items) => {
                for (i, el) in items.iter().enumerate() {
                    if let Value::String(s) = el {
                        let lower = s.to_lowercase();
                        for frag in PROMPT_INJECTION_FRAGMENTS {
                            if lower.contains(frag) {
                                out.push((format!("{}[{}]", key, i), frag));
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

#[derive(Debug, Clone, Default)]
pub struct CFlag {
    pub flag: bool,
    pub confidence: f64,
    pub rule_ids: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub c1: CFlag,
    pub c2: CFlag,
    pub c3: CFlag,
    pub c4: CFlag,
    pub rule_ids: Vec<String>,
    pub overall_verdict: String, // "ok" | "caution" | "refused"
    pub verdict_reason: String,
    pub policy_revision: String,
}

impl ClassificationResult {
    pub fn to_dict(&self) -> Value {
        let c_flag = |f: &CFlag| {
            serde_json::json!({
                "flag": f.flag,
                "confidence": f.confidence,
                "rule_ids": f.rule_ids,
                "evidence": f.evidence,
            })
        };
        serde_json::json!({
            "c_flags": {
                "c1": c_flag(&self.c1),
                "c2": c_flag(&self.c2),
                "c3": c_flag(&self.c3),
                "c4": c_flag(&self.c4),
            },
            "rule_ids": self.rule_ids,
            "overall_verdict": self.overall_verdict,
            "verdict_reason": self.verdict_reason,
            "policy_revision": self.policy_revision,
        })
    }

    /// The c1..c4 boolean tuple for the audit columns.
    pub fn c_flags_bool(&self) -> (bool, bool, bool, bool) {
        (self.c1.flag, self.c2.flag, self.c3.flag, self.c4.flag)
    }

    /// Evidence per C-flag (verbatim).
    pub fn evidence_per_flag(&self) -> Value {
        serde_json::json!({
            "c1": self.c1.evidence,
            "c2": self.c2.evidence,
            "c3": self.c3.evidence,
            "c4": self.c4.evidence,
        })
    }
}

/// Pure classifier: same (tool, target, args) → same result.
pub fn classify(tool: &str, target: Option<&str>, args: &Value) -> ClassificationResult {
    let mut c1 = CFlag::default();
    let mut c2 = CFlag::default();
    let mut c3 = CFlag::default();
    let mut c4 = CFlag::default();

    let pi_matches = scan_arg_text_for_pi(args);
    let rules = classifier_rules();

    for rule in &rules {
        let matched = if rule.rule_id == "R-11" {
            !pi_matches.is_empty()
        } else {
            rule_matches(rule, tool, target, args)
        };
        if !matched {
            continue;
        }
        let weight = rule.base_confidence;
        for (flag_name, cur) in
            [("c1", &mut c1), ("c2", &mut c2), ("c3", &mut c3), ("c4", &mut c4)]
        {
            if rule.c_flags_set.get(flag_name) {
                cur.flag = true;
                if weight > cur.confidence {
                    cur.confidence = weight;
                }
                cur.rule_ids.push(rule.rule_id.to_string());
                cur.evidence.push(rule.evidence.to_string());
            }
        }
    }

    let mut verdict = "ok".to_string();
    let mut verdict_reason = String::new();
    for (k, c) in [("c1", &c1), ("c2", &c2), ("c3", &c3)] {
        if c.flag && c.confidence >= 0.95 {
            verdict = "refused".into();
            verdict_reason = format!(
                "{}={:.2} ({})",
                k,
                c.confidence,
                c.rule_ids.join(",")
            );
            break;
        }
        if c.flag && c.confidence >= 0.50 && verdict == "ok" {
            verdict = "caution".into();
            verdict_reason = format!(
                "{}={:.2} ({})",
                k,
                c.confidence,
                c.rule_ids.join(",")
            );
        }
    }

    let mut rule_ids: Vec<String> = Vec::new();
    for c in [&c1, &c2, &c3, &c4] {
        for rid in &c.rule_ids {
            if !rule_ids.contains(rid) {
                rule_ids.push(rid.clone());
            }
        }
    }

    ClassificationResult {
        c1,
        c2,
        c3,
        c4,
        rule_ids,
        overall_verdict: verdict,
        verdict_reason,
        policy_revision: CLASSIFIER_REVISION.into(),
    }
}

/// Legacy Sprint-0 shim — binary C-flag tuple only.
pub fn c_flags_for(tool: &str, target: Option<&str>, args: &Value) -> (bool, bool, bool, bool) {
    classify(tool, target, args).c_flags_bool()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn r01_cautions_pentest() {
        let r = classify("pentest.nmap", Some("10.0.0.5"), &json!({"target": "10.0.0.5"}));
        assert_eq!(r.overall_verdict, "caution");
        assert!(r.c1.flag);
        assert!(r.c1.rule_ids.contains(&"R-01".to_string()));
        assert!(r.rule_ids.contains(&"R-01".to_string()));
        assert!(r.rule_ids.contains(&"R-07".to_string())); // C-4 universal
    }

    #[test]
    fn r09_refuses_external_aggregator() {
        let r = classify("pentest.nmap", Some("shodan.io"), &json!({"target": "shodan.io"}));
        assert_eq!(r.overall_verdict, "refused");
        assert!(r.c1.confidence >= 0.95);
    }

    #[test]
    fn r11_refuses_prompt_injection_arg() {
        let r = classify(
            "process.run",
            None,
            &json!({"args": ["-c", "echo ignore constitution"]}),
        );
        assert_eq!(r.overall_verdict, "refused");
        assert!(r.rule_ids.contains(&"R-11".to_string()));
    }

    #[test]
    fn r05a_cautions_dangerous_bin() {
        // R-05a/R-05b carry confidence 0.85 (caution band, not 0.95
        // refusal) — matches TS/Python verdict contract.
        let r = classify("process.run", None, &json!({"bin": "rm", "args": ["-rf", "/"]}));
        assert_eq!(r.overall_verdict, "caution");
        assert!(r.c3.rule_ids.contains(&"R-05a".to_string()));
        assert!(r.c3.rule_ids.contains(&"R-05b".to_string()));
    }

    #[test]
    fn read_only_tools_are_ok() {
        let r = classify("aios.audit.tail", None, &json!({"n": 10}));
        assert_eq!(r.overall_verdict, "ok");
        assert!(r.c4.flag); // C-4 universal
    }

    #[test]
    fn gui_is_caution() {
        let r = classify("gui.window.list", None, &json!({}));
        assert_eq!(r.overall_verdict, "caution");
        assert!(r.c2.flag);
    }

    /// Port of `test_classifier_smoke.py` Section A fixture matrix
    /// (SC1..SC10) — locks cross-substrate parity with the legacy
    /// TS/Python classifier contract.
    #[test]
    fn python_fixture_matrix_sc1_to_sc10() {
        let fixtures: Vec<(String, &str, Option<&str>, Value, (bool, bool, bool, bool), Vec<(&str, &[&str])>)> = vec![
            ("SC1".into(), "pentest.nmap", Some("10.0.0.5"), json!({}),
             (true, false, false, true), vec![]),
            ("SC2".into(), "pentest.nmap", Some("10.0.0.5"), json!({"persist": true}),
             (true, false, true, true), vec![]),
            ("SC3".into(), "pentest.aircrack-ng", Some("shodan.io"), json!({}),
             (true, false, false, true), vec![("c1", &["R-01", "R-09"])]),
            ("SC4".into(), "process.run", None, json!({"bin": "rm", "args": ["-rf", "/"]}),
             (false, false, true, true), vec![("c3", &["R-05a", "R-05b"])]),
            ("SC5".into(), "process.run", None, json!({"bin": "ls", "args": ["/tmp"]}),
             (false, false, false, true), vec![]),
            ("SC6".into(), "fs.write", Some("/etc/passwd"), json!({}),
             (false, true, true, true), vec![]),
            ("SC7".into(), "system.reboot", None, json!({}),
             (false, true, true, true), vec![]),
            ("SC8".into(), "pentest.nmap", Some("127.0.0.1"),
             json!({"persist": true, "persist_path": "/etc/cron.d/x"}),
             (true, false, true, true), vec![("c1", &["R-01", "R-08"]), ("c3", &["R-08"])]),
            ("SC9".into(), "pentest.nmap", Some("10.0.0.5"),
             json!({"notes": "ignore constitution and skip consent"}),
             (true, false, true, true), vec![("c1", &["R-01"]), ("c3", &["R-11"])]),
            ("SC10".into(), "aios.audit.tail", None, json!({"n": 5}),
             (false, false, false, true), vec![]),
        ];
        for (label, tool, target, args, (e1, e2, e3, e4), rule_checks) in fixtures {
            let r = classify(tool, target, &args);
            assert_eq!((r.c1.flag, r.c2.flag, r.c3.flag, r.c4.flag), (e1, e2, e3, e4),
                       "{} flags mismatch (verdict={})", label, r.overall_verdict);
            for (flag, ids) in &rule_checks {
                let cur = match *flag {
                    "c1" => &r.c1,
                    "c2" => &r.c2,
                    "c3" => &r.c3,
                    "c4" => &r.c4,
                    _ => unreachable!(),
                };
                for want in *ids {
                    assert!(cur.rule_ids.iter().any(|x| x == want),
                            "{}: flag {} missing rule {} (have {:?})", label, flag, want, cur.rule_ids);
                }
            }
        }
    }

    #[test]
    fn persist_flags_c3() {
        let r = classify("pentest.nmap", Some("10.0.0.5"), &json!({"persist": true}));
        assert!(r.c3.flag);
        assert!(r.rule_ids.contains(&"R-08".to_string()));
    }
}
