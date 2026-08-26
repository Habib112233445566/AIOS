//! Sprint 2 — agent loop (Ollama backend + deterministic stub).
//!
//! Port of `agent.ts`. Loop contract (ADR-0035 D-1/D-2/D-4):
//!   observe → think (Ollama JSON plan) → classify → tool call
//!   → observe tool result → repeat.
//!
//! The agent never executes host tools directly: it dispatches through
//! the MCP surface (which performs its own classifier → PEP → audit
//! gate). The local classifier is a preflight, not the authority.

use crate::audit::{AuditRing, AuditRowInput};
use crate::classifier::{classify, ClassificationResult};
use crate::pep::PepStore;
use crate::types::CFlags;
use serde_json::Value;

pub const SUPPORTED_TOOLS: &[&str] = &[
    "aios.fs.read",
    "aios.process.list",
    "aios.audit.tail",
    "aios.audit.verify",
    "pentest.nmap",
    "pentest.nikto",
    "pentest.sqlmap",
    "pentest.tshark",
    "pentest.aircrack-ng",
];

const OLLAMA_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug, Clone)]
pub struct AgentToolCall {
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone)]
pub struct AgentPlan {
    pub reasoning: String,
    pub tool_calls: Vec<AgentToolCall>,
    pub stop_reason: String, // "end_turn" | "tool_use" | "max_steps"
}

#[derive(Debug, Clone)]
pub struct AgentToolResult {
    pub tool: String,
    pub args: Value,
    pub outcome: String, // "ok" | "refused" | "error"
    pub audit_id: i64,
    pub reason: Option<String>,
    pub result_preview: Option<String>,
    pub classifier_blocked: bool,
    pub via: String, // "mcp" | "classifier"
}

#[derive(Debug, Clone)]
pub struct AgentStep {
    pub step: usize,
    pub reasoning: String,
    pub tool_results: Vec<AgentToolResult>,
}

#[derive(Debug, Clone)]
pub struct AgentLoopResult {
    pub steps: Vec<AgentStep>,
    pub total_steps: usize,
    pub stop_reason: String, // "end_turn" | "max_steps" | "abort"
    pub total_tool_calls: usize,
    pub total_refused: usize,
    pub classifier_policy_revision: String,
    pub model_kind: String, // "ollama" | "stub"
    pub model_name: String,
    pub mcp_tools: Vec<String>,
}

impl AgentLoopResult {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "steps": self.steps.iter().map(|s| serde_json::json!({
                "step": s.step,
                "reasoning": s.reasoning,
                "tool_results": s.tool_results.iter().map(|r| serde_json::json!({
                    "tool": r.tool,
                    "args": r.args,
                    "outcome": r.outcome,
                    "audit_id": r.audit_id,
                    "reason": r.reason,
                    "result_preview": r.result_preview,
                    "classifier_blocked": r.classifier_blocked,
                    "via": r.via,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "total_steps": self.total_steps,
            "stop_reason": self.stop_reason,
            "total_tool_calls": self.total_tool_calls,
            "total_refused": self.total_refused,
            "classifier_policy_revision": self.classifier_policy_revision,
            "model": {"kind": self.model_kind, "name": self.model_name},
            "mcp_tools": self.mcp_tools,
        })
    }
}

pub struct AgentLoopOptions<'a> {
    pub prompt: &'a str,
    pub grant_id: Option<&'a str>,
    pub ring: &'a mut AuditRing,
    pub constitution_rev: &'a str,
    pub ollama_url: &'a str,
    pub ollama_model: &'a str,
    pub max_steps: usize,
    pub pep: &'a PepStore,
    /// Tool dispatcher — in the CLI this forwards to the MCP bridge.
    /// In tests, an in-process dispatcher is used.
    pub dispatcher: Option<&'a dyn Fn(&str, &Value) -> Result<Value, String>>,
}

fn target_for(tool: &str, input: &Value) -> Option<String> {
    if let Some(t) = input.get("target").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }
    if tool == "pentest.sqlmap" {
        if let Some(u) = input.get("url").and_then(|v| v.as_str()) {
            return Some(u.to_string());
        }
    }
    if tool == "pentest.tshark" {
        if let Some(p) = input.get("pcap_path").and_then(|v| v.as_str()) {
            return Some(p.to_string());
        }
    }
    if tool == "pentest.aircrack-ng" {
        if let Some(c) = input.get("capture_path").and_then(|v| v.as_str()) {
            return Some(c.to_string());
        }
    }
    if tool == "aios.fs.read" {
        if let Some(p) = input.get("path").and_then(|v| v.as_str()) {
            return Some(p.to_string());
        }
    }
    None
}

fn write_agent_audit_row(
    opts: &mut AgentLoopOptions,
    tool: &str,
    args: &Value,
    target: Option<&str>,
    cls: &ClassificationResult,
    outcome: &str,
    detail: Option<&str>,
) -> i64 {
    let c_flags = CFlags {
        c1: cls.c1.flag,
        c2: cls.c2.flag,
        c3: cls.c3.flag,
        c4: cls.c4.flag,
    };
    opts.ring.write(AuditRowInput {
        ts: crate::canonical::utcnow_iso(),
        actor: "agent".into(),
        actor_id: "agent:ollama@aiosh-cli".into(),
        tool: tool.into(),
        command: format!("agent MCP tools/call {}", tool),
        args: args.clone(),
        target: target.map(|s| s.into()),
        outcome: outcome.into(),
        outcome_detail: detail.map(|s| s.into()),
        constitution_rev: Some(opts.constitution_rev.into()),
        grant_token: opts.grant_id.map(|s| s.into()),
        c_flags,
        policy_revision: Some(cls.policy_revision.clone()),
        classify_rule_ids: Some(cls.rule_ids.clone()),
        classify_evidence: Some(cls.evidence_per_flag()),
        classify_overall_verdict: Some(cls.overall_verdict.clone()),
        classify_verdict_reason: Some(cls.verdict_reason.clone()),
    })
    .map(|row| row.id)
    .unwrap_or(-1)
}

fn dispatch_agent_tool_call(
    opts: &mut AgentLoopOptions,
    tool_call: &AgentToolCall,
) -> AgentToolResult {
    let target = target_for(&tool_call.name, &tool_call.input);
    // Local preflight (never performs the action).
    let cls = classify(&tool_call.name, target.as_deref(), &tool_call.input);
    if cls.overall_verdict == "refused" {
        let detail = format!(
            "classifier refused (policy={}, verdict={})",
            cls.policy_revision,
            if cls.verdict_reason.is_empty() { "refused" } else { &cls.verdict_reason }
        );
        let audit_id = write_agent_audit_row(
            opts, &tool_call.name, &tool_call.input, target.as_deref(), &cls,
            "refused", Some(&detail),
        );
        return AgentToolResult {
            tool: tool_call.name.clone(),
            args: tool_call.input.clone(),
            outcome: "refused".into(),
            audit_id,
            reason: Some(detail),
            result_preview: None,
            classifier_blocked: true,
            via: "classifier".into(),
        };
    }

    // Dispatch through the configured path.
    let result = match opts.dispatcher {
        Some(d) => d(&tool_call.name, &tool_call.input),
        None => {
            // Fallback: in-process local execution is not permitted for
            // host tools; report a refusal-grade error.
            Err("no dispatcher configured (agent requires MCP bridge)".into())
        }
    };

    match result {
        Ok(raw) => {
            let server_ok = raw.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            let outcome = if server_ok {
                "ok"
            } else if raw.get("gate").and_then(|v| v.as_str()).is_some()
                || raw.get("reason").and_then(|v| v.as_str()).unwrap_or("").contains("grant")
            {
                "refused"
            } else {
                "error"
            };
            let audit_id = raw.get("audit_id").and_then(|v| v.as_i64()).unwrap_or(-1);
            let reason = raw
                .get("reason")
                .and_then(|v| v.as_str())
                .or_else(|| raw.get("error").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
            let preview = crate::canonical::canonical(&raw)
                .chars()
                .take(2_048)
                .collect::<String>();
            if audit_id < 0 {
                // Server failed before auditing — write a local error row.
                let aid = write_agent_audit_row(
                    opts, &tool_call.name, &tool_call.input, target.as_deref(), &cls,
                    outcome, reason.as_deref(),
                );
                AgentToolResult {
                    tool: tool_call.name.clone(),
                    args: tool_call.input.clone(),
                    outcome: outcome.into(),
                    audit_id: aid,
                    reason,
                    result_preview: Some(preview),
                    classifier_blocked: false,
                    via: "mcp".into(),
                }
            } else {
                AgentToolResult {
                    tool: tool_call.name.clone(),
                    args: tool_call.input.clone(),
                    outcome: outcome.into(),
                    audit_id,
                    reason,
                    result_preview: Some(preview),
                    classifier_blocked: false,
                    via: "mcp".into(),
                }
            }
        }
        Err(detail) => {
            let audit_id = write_agent_audit_row(
                opts, &tool_call.name, &tool_call.input, target.as_deref(), &cls,
                "error", Some(&detail),
            );
            AgentToolResult {
                tool: tool_call.name.clone(),
                args: tool_call.input.clone(),
                outcome: "error".into(),
                audit_id,
                reason: Some(detail),
                result_preview: None,
                classifier_blocked: false,
                via: "mcp".into(),
            }
        }
    }
}

/// Normalize an Ollama JSON plan.
fn normalize_plan(value: &Value) -> Option<AgentPlan> {
    let obj = value.as_object()?;
    let calls_raw = obj.get("tool_calls")?.as_array()?;
    let mut tool_calls = Vec::new();
    for raw in calls_raw {
        let item = raw.as_object()?;
        let name = item.get("name")?.as_str()?;
        if !SUPPORTED_TOOLS.contains(&name) {
            return None;
        }
        let input = item.get("input")?.as_object()?;
        tool_calls.push(AgentToolCall {
            name: name.to_string(),
            input: Value::Object(input.clone()),
        });
    }
    let stop = obj.get("stop_reason").and_then(|v| v.as_str()).unwrap_or("");
    let stop_reason = if stop == "max_steps" {
        "max_steps"
    } else if !tool_calls.is_empty() {
        "tool_use"
    } else {
        "end_turn"
    };
    Some(AgentPlan {
        reasoning: obj
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("Ollama returned no reasoning field.")
            .to_string(),
        tool_calls,
        stop_reason: stop_reason.to_string(),
    })
}

fn plan_with_ollama(
    prompt: &str,
    observations: &[String],
    url: &str,
    model: &str,
) -> Option<AgentPlan> {
    let available = SUPPORTED_TOOLS.join(", ");
    let system = format!(
        "You are the AIOS S-rank agent. Use only these canonical MCP tools: {}. \
         Return strict JSON only: {{reasoning:string, tool_calls:[{{name:string,input:object}}], \
         stop_reason:'tool_use'|'end_turn'}}. Never invent a tool. Never put shell commands \
         in the response. The server owns authorization; request only the smallest action needed.",
        available
    );
    let mut messages: Vec<Value> = vec![
        serde_json::json!({"role": "system", "content": system}),
        serde_json::json!({"role": "user", "content": prompt}),
    ];
    if !observations.is_empty() {
        messages.push(serde_json::json!({
            "role": "user",
            "content": format!(
                "Observed MCP results from prior steps. Re-plan from these facts; do not repeat \
                 a refused action unless the user prompt clearly requires a different safe action:\n{}",
                observations.join("\n")
            ),
        }));
    }
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "format": "json",
    });
    let client = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(OLLAMA_TIMEOUT_MS))
        .build();
    let resp = match client.post(&format!("{}/api/chat", url)).send_json(body) {
        Ok(r) => r,
        Err(_) => return None,
    };
    let body: Value = match resp.into_json() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let text = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())?;
    let parsed: Value = serde_json::from_str(text).ok()?;
    normalize_plan(&parsed)
}

fn plan_with_stub(prompt: &str, observations: &[String]) -> AgentPlan {
    let lc = prompt.to_lowercase();
    let mut calls: Vec<AgentToolCall> = vec![];
    let mut reasoning = "Deterministic fallback: Ollama unavailable; selected a bounded MCP read/action from prompt keywords.".to_string();

    let re_verify = ["audit verify", "verify chain", "chain verify"];
    if re_verify.iter().any(|r| lc.contains(r)) {
        calls.push(AgentToolCall { name: "aios.audit.verify".into(), input: serde_json::json!({}) });
        reasoning = "Fallback plan: verify the audit chain through MCP.".into();
    } else if ["audit tail", "tail audit", "recent rows"].iter().any(|r| lc.contains(r)) {
        let n = extract_number(&lc).unwrap_or(10);
        calls.push(AgentToolCall { name: "aios.audit.tail".into(), input: serde_json::json!({"n": n}) });
        reasoning = "Fallback plan: inspect recent audit rows through MCP.".into();
    } else if ["list process", "process list", "ps "].iter().any(|r| lc.contains(r)) {
        calls.push(AgentToolCall { name: "aios.process.list".into(), input: serde_json::json!({}) });
        reasoning = "Fallback plan: list processes through MCP.".into();
    } else if ["read file", "fs.read", "cat "].iter().any(|r| lc.contains(r)) {
        if let Some(m) = extract_target(&lc, &["read", "fs.read", "cat"]) {
            calls.push(AgentToolCall {
                name: "aios.fs.read".into(),
                input: serde_json::json!({"path": m}),
            });
            reasoning = format!("Fallback plan: read {} through MCP.", m);
        }
    } else if ["scan ", "nmap", "ports", "recon"].iter().any(|r| lc.contains(r)) {
        if let Some(m) = extract_target(&lc, &["scan", "nmap", "recon"]) {
            calls.push(AgentToolCall {
                name: "pentest.nmap".into(),
                input: serde_json::json!({"target": m}),
            });
            reasoning = format!("Fallback plan: nmap {} through MCP; PEP grant required.", m);
        }
    } else if ["nikto", "web scan", "webserver"].iter().any(|r| lc.contains(r)) {
        if let Some(m) = extract_target(&lc, &["nikto", "web"]) {
            calls.push(AgentToolCall {
                name: "pentest.nikto".into(),
                input: serde_json::json!({"target": m}),
            });
        }
    }

    if !observations.is_empty()
        && observations.last().map(|o| o.contains("refused")).unwrap_or(false)
    {
        return AgentPlan {
            reasoning: "Fallback stopped after a refused MCP action.".into(),
            tool_calls: vec![],
            stop_reason: "end_turn".into(),
        };
    }
    AgentPlan {
        reasoning,
        tool_calls: calls.clone(),
        stop_reason: if calls.is_empty() { "end_turn" } else { "tool_use" }.into(),
    }
}

fn extract_number(s: &str) -> Option<i64> {
    // First standalone numeric token ("audit tail 5" → 5), not a
    // concatenation of every digit in the prompt.
    s.split_whitespace().find_map(|w| {
        let digits: String =
            w.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            None
        } else {
            digits.parse().ok()
        }
    })
}

fn extract_target(lc: &str, keywords: &[&str]) -> Option<String> {
    let words: Vec<&str> = lc.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        if keywords.iter().any(|k| w.contains(k)) {
            if let Some(next) = words.get(i + 1) {
                let clean = next
                    .trim_matches(|c: char| !(c.is_alphanumeric() || c == '.' || c == ':' || c == '/' || c == '-'));
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Run the agent loop. If `dispatcher` is None and Ollama is
/// unreachable, the loop uses the stub planner and reports
/// "no dispatcher" refusals — tests inject a dispatcher.
pub fn run_agent_loop(mut opts: AgentLoopOptions) -> AgentLoopResult {
    let mut steps: Vec<AgentStep> = vec![];
    let mut observations: Vec<String> = vec![];
    let mut total_tool_calls = 0usize;
    let mut total_refused = 0usize;
    let (mut model_kind, mut model_name) = ("stub".to_string(), "deterministic-stub".to_string());
    let mut policy_revision = crate::classifier::CLASSIFIER_REVISION.to_string();
    let mut stop_reason = "end_turn".to_string();
    let mcp_tools: Vec<String> = SUPPORTED_TOOLS.iter().map(|s| s.to_string()).collect();

    let ollama_available = detect_ollama(opts.ollama_url);
    if ollama_available {
        model_kind = "ollama".into();
        model_name = opts.ollama_model.to_string();
    }

    for i in 0..opts.max_steps {
        let ollama_plan = if model_kind == "ollama" {
            plan_with_ollama(opts.prompt, &observations, opts.ollama_url, opts.ollama_model)
        } else {
            None
        };
        let plan = ollama_plan.unwrap_or_else(|| plan_with_stub(opts.prompt, &observations));
        // char-safe truncation (byte-slicing can panic mid-codepoint)
        let prompt_head: String = opts.prompt.chars().take(256).collect();
        policy_revision = classify("agent.step", None, &serde_json::json!({"prompt": prompt_head, "step": i}))
            .policy_revision;

        if plan.tool_calls.is_empty() || plan.stop_reason == "end_turn" {
            steps.push(AgentStep { step: i, reasoning: plan.reasoning, tool_results: vec![] });
            stop_reason = "end_turn".into();
            break;
        }

        let mut results: Vec<AgentToolResult> = vec![];
        for tool_call in &plan.tool_calls {
            total_tool_calls += 1;
            let result = dispatch_agent_tool_call(&mut opts, tool_call);
            if result.outcome == "refused" {
                total_refused += 1;
            }
            observations.push(serde_json::json!({
                "tool": result.tool,
                "outcome": result.outcome,
                "audit_id": result.audit_id,
                "reason": result.reason,
                "result_preview": result.result_preview,
            })
            .to_string());
            results.push(result);
        }
        steps.push(AgentStep { step: i, reasoning: plan.reasoning.clone(), tool_results: results.clone() });
        if results.iter().all(|r| r.outcome == "refused") {
            stop_reason = "abort".into();
            break;
        }
        if i == opts.max_steps - 1 {
            stop_reason = "max_steps".into();
        }
    }

    AgentLoopResult {
        steps: steps.clone(),
        total_steps: steps.len(),
        stop_reason,
        total_tool_calls,
        total_refused,
        classifier_policy_revision: policy_revision,
        model_kind,
        model_name,
        mcp_tools,
    }
}

fn detect_ollama(url: &str) -> bool {
    let client = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(1_500))
        .build();
    match client.get(&format!("{}/api/tags", url)).call() {
        Ok(r) => r.status() == 200,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_plan_accepts_valid_json() {
        let v: Value = serde_json::from_str(
            r#"{"reasoning":"scan it","tool_calls":[{"name":"pentest.nmap","input":{"target":"10.0.0.5"}}],"stop_reason":"tool_use"}"#,
        )
        .unwrap();
        let p = normalize_plan(&v).unwrap();
        assert_eq!(p.tool_calls.len(), 1);
        assert_eq!(p.tool_calls[0].name, "pentest.nmap");
        assert_eq!(p.stop_reason, "tool_use");
    }

    #[test]
    fn normalize_plan_rejects_unknown_tool() {
        let v: Value = serde_json::from_str(
            r#"{"reasoning":"x","tool_calls":[{"name":"evil.tool","input":{}}],"stop_reason":"tool_use"}"#,
        )
        .unwrap();
        assert!(normalize_plan(&v).is_none());
    }

    #[test]
    fn stub_plan_scan() {
        let p = plan_with_stub("please scan 10.0.0.5", &[]);
        assert_eq!(p.tool_calls.len(), 1);
        assert_eq!(p.tool_calls[0].name, "pentest.nmap");
        assert_eq!(p.tool_calls[0].input["target"], "10.0.0.5");
    }

    #[test]
    fn stub_plan_audit_tail() {
        let p = plan_with_stub("show me audit tail 5", &[]);
        assert_eq!(p.tool_calls[0].name, "aios.audit.tail");
        assert_eq!(p.tool_calls[0].input["n"], 5);
    }

    #[test]
    fn stub_stops_after_refusal() {
        let p = plan_with_stub("scan 10.0.0.5", &["{\"outcome\":\"refused\"}".to_string()]);
        assert!(p.tool_calls.is_empty());
        assert_eq!(p.stop_reason, "end_turn");
    }
}
