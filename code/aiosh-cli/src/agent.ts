/**
 * Sprint 2 — Ollama 0.22.1 agent loop over the real MCP server.
 *
 * Loop contract (ADR-0035 D-1/D-2/D-4):
 *
 *   observe → think (Ollama JSON plan) → classify → MCP tools/call
 *   → observe tool result → repeat
 *
 * This module deliberately contains NO direct host-tool execution. The
 * only action path is `McpBridge.call()`, which speaks to the real
 * `aiosh_mcp.server` over MCP stdio. The Python server performs its own
 * classifier → PEP → audit gate, so the local TypeScript classifier is
 * a preflight and not a bypassable authority.
 */

import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createInterface, type Interface as ReadlineInterface } from "node:readline";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { classify, type ClassificationResult } from "./constitution.js";
import { AuditRing } from "./audit.js";
import { PepStore } from "./pep.js";
import { type AuditRow } from "./types.js";


export interface AgentToolCall {
  name: string;
  input: Record<string, unknown>;
}

export interface AgentPlan {
  reasoning: string;
  tool_calls: AgentToolCall[];
  stop_reason: "end_turn" | "tool_use" | "max_steps";
}

export interface AgentLoopOptions {
  prompt: string;
  grant_id: string | undefined;
  ring: AuditRing;
  constitution_rev: string;
  ollama_url: string;
  ollama_model: string;
  max_steps: number;
  open_context_pep: PepStore;
}

export interface AgentToolResult {
  tool: string;
  args: Record<string, unknown>;
  outcome: "ok" | "refused" | "error";
  audit_id: number;
  reason?: string;
  result_preview?: string;
  classifier: ClassificationResult;
  classifier_blocked: boolean;
  via: "mcp" | "classifier";
}

export interface AgentStep {
  step: number;
  plan: AgentPlan;
  tool_results: AgentToolResult[];
}

export interface AgentLoopResult {
  steps: AgentStep[];
  total_steps: number;
  stop_reason: "end_turn" | "max_steps" | "abort";
  total_tool_calls: number;
  total_refused: number;
  classifier_policy_revision: string;
  model: { kind: "ollama" | "stub"; name: string };
  mcp_tools: string[];
}

interface BridgeResponse {
  id?: number;
  ok: boolean;
  error?: string;
  tool?: string;
  result?: unknown;
  is_error?: boolean;
}

interface BridgeReady {
  event: "ready";
  tools: string[];
}

const OLLAMA_TIMEOUT_MS = 5_000;
const MCP_TIMEOUT_MS = 30_000;
const TOOL_RESULT_PREVIEW_CHARS = 2_048;
const MCP_ROOT = process.env["AIOSH_MCP_ROOT"] ??
  resolve(dirname(fileURLToPath(import.meta.url)), "../../aiosh-mcp");

const SUPPORTED_TOOLS = new Set([
  "aios.fs.read", "aios.process.list", "aios.audit.tail",
  "aios.audit.verify", "pentest.nmap", "pentest.nikto",
  "pentest.sqlmap", "pentest.tshark", "pentest.aircrack-ng",
]);


/** Persistent JSONL client whose peer is the actual MCP stdio server.
 * JSONL is only the internal process boundary between TS and Python;
 * Python then speaks MCP JSON-RPC to `aiosh_mcp.server`. */
class McpBridge {
  private child?: ChildProcessWithoutNullStreams;
  private lines?: ReadlineInterface;
  private nextId = 1;
  private readyPromise?: Promise<BridgeReady>;
  private readyResolve?: (ready: BridgeReady) => void;
  private readyReject?: (error: Error) => void;
  private pending = new Map<number, {
    resolve: (value: BridgeResponse) => void;
    reject: (error: Error) => void;
  }>();
  private ready?: BridgeReady;

  async start(): Promise<BridgeReady> {
    if (this.ready) return this.ready;
    if (this.readyPromise) return this.readyPromise;

    this.readyPromise = new Promise<BridgeReady>((resolveReady, rejectReady) => {
      this.readyResolve = resolveReady;
      this.readyReject = rejectReady;
    });
    const env = {
      ...process.env,
      AIOSH_MCP_ROOT: MCP_ROOT,
      PYTHONPATH: `${MCP_ROOT}${process.env["PYTHONPATH"]
        ? `:${process.env["PYTHONPATH"]}` : ""}`,
    };
    this.child = spawn("python3", ["-m", "aiosh_mcp.agent_bridge"], {
      cwd: MCP_ROOT,
      env,
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.child.stderr.on("data", (chunk: Buffer) => {
      // MCP/server diagnostics stay out of the agent protocol. Keep a
      // bounded diagnostic stream available when AIOSH_AGENT_DEBUG=1.
      if (process.env["AIOSH_AGENT_DEBUG"] === "1")
        process.stderr.write(`[mcp] ${chunk.toString()}`);
    });
    this.child.on("error", (error) => this.fail(error));
    this.child.on("exit", (code, signal) => {
      if (!this.ready) {
        this.fail(new Error(`MCP bridge exited before ready: ${code ?? signal}`));
      }
      for (const waiter of this.pending.values()) {
        waiter.reject(new Error(`MCP bridge exited: ${code ?? signal}`));
      }
      this.pending.clear();
    });
    this.lines = createInterface({ input: this.child.stdout });
    this.lines.on("line", (line) => this.handleLine(line));

    const timer = setTimeout(() => {
      this.fail(new Error("MCP bridge ready timeout"));
    }, MCP_TIMEOUT_MS);
    try {
      const ready = await this.readyPromise;
      this.ready = ready;
      return ready;
    } finally {
      clearTimeout(timer);
    }
  }

  async call(tool: string, args: Record<string, unknown>): Promise<BridgeResponse> {
    const ready = await this.start();
    if (!ready.tools.includes(tool)) {
      return { ok: false, error: `MCP server did not advertise ${tool}` };
    }
    const id = this.nextId++;
    const request = JSON.stringify({ id, op: "call", tool, arguments: args });
    return new Promise<BridgeResponse>((resolveResponse, rejectResponse) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        rejectResponse(new Error(`MCP call timeout: ${tool}`));
      }, MCP_TIMEOUT_MS);
      this.pending.set(id, {
        resolve: (value) => {
          clearTimeout(timeout);
          resolveResponse(value);
        },
        reject: (error) => {
          clearTimeout(timeout);
          rejectResponse(error);
        },
      });
      this.child?.stdin.write(`${request}\n`);
    });
  }

  close(): void {
    this.lines?.close();
    this.child?.stdin.end();
    this.child?.kill();
    this.child = undefined;
    this.ready = undefined;
    this.readyPromise = undefined;
  }

  private handleLine(line: string): void {
    try {
      const value = JSON.parse(line) as BridgeReady | BridgeResponse;
      if ((value as BridgeReady).event === "ready") {
        this.readyResolve?.(value as BridgeReady);
        this.readyResolve = undefined;
        this.readyReject = undefined;
        return;
      }
      const response = value as BridgeResponse;
      if (typeof response.id !== "number") return;
      const waiter = this.pending.get(response.id);
      if (!waiter) return;
      this.pending.delete(response.id);
      waiter.resolve(response);
    } catch {
      // Ignore non-JSON stdout lines; the bridge contract emits JSONL.
    }
  }

  private fail(error: Error): void {
    this.readyReject?.(error);
    this.readyResolve = undefined;
    this.readyReject = undefined;
  }
}


function targetFor(tool: string, input: Record<string, unknown>): string | undefined {
  if (typeof input["target"] === "string") return input["target"];
  if (tool === "pentest.sqlmap" && typeof input["url"] === "string")
    return input["url"];
  if (tool === "pentest.tshark" && typeof input["pcap_path"] === "string")
    return input["pcap_path"];
  if (tool === "pentest.aircrack-ng" && typeof input["capture_path"] === "string")
    return input["capture_path"];
  if (tool === "aios.fs.read" && typeof input["path"] === "string")
    return input["path"];
  return undefined;
}

function classifierEvidence(cls: ClassificationResult): Record<"c1" | "c2" | "c3" | "c4", string[]> {
  return {
    c1: cls.c_flags.c1.evidence,
    c2: cls.c_flags.c2.evidence,
    c3: cls.c_flags.c3.evidence,
    c4: cls.c_flags.c4.evidence,
  };
}

function writeAgentAuditRow(
  opts: AgentLoopOptions,
  tool: string,
  args: Record<string, unknown>,
  target: string | undefined,
  cls: ClassificationResult,
  outcome: "ok" | "refused" | "error",
  detail: string | undefined,
): AuditRow {
  return opts.ring.write({
    ts: new Date().toISOString(),
    actor: "agent",
    actor_id: "agent:ollama@aiosh-cli",
    tool,
    command: `agent MCP tools/call ${tool}`,
    args,
    target,
    outcome,
    outcome_detail: detail,
    constitution_rev: opts.constitution_rev,
    grant_token: opts.grant_id ?? undefined,
    c_flags: {
      c1: cls.c_flags.c1.flag,
      c2: cls.c_flags.c2.flag,
      c3: cls.c_flags.c3.flag,
      c4: cls.c_flags.c4.flag,
    },
    policy_revision: cls.policy_revision,
    classify_rule_ids: cls.rule_ids,
    classify_evidence: classifierEvidence(cls),
    classify_overall_verdict: cls.overall_verdict,
    classify_verdict_reason: cls.verdict_reason,
  });
}


async function dispatchAgentToolCall(
  bridge: McpBridge,
  toolCall: AgentToolCall,
  opts: AgentLoopOptions,
): Promise<AgentToolResult> {
  const target = targetFor(toolCall.name, toolCall.input);
  // Local preflight: this can stop a refused request before MCP, but it
  // never performs the action. The MCP server repeats the same decision.
  const cls = classify(toolCall.name, target, toolCall.input);
  if (cls.overall_verdict === "refused") {
    const detail = `classifier refused (policy=${cls.policy_revision}, ` +
      `verdict=${cls.verdict_reason})`;
    const row = writeAgentAuditRow(
      opts, toolCall.name, toolCall.input, target, cls, "refused", detail,
    );
    return {
      tool: toolCall.name, args: toolCall.input, outcome: "refused",
      audit_id: row.id, reason: detail, classifier: cls,
      classifier_blocked: true, via: "classifier",
    };
  }

  try {
    // The actual tool action crosses MCP here. PEP and server-side
    // classifier/audit enforcement happen inside aiosh_mcp.server.
    const response = await bridge.call(toolCall.name, {
      ...toolCall.input,
      ...(opts.grant_id ? { grant_id: opts.grant_id } : {}),
    });
    if (!response.ok) {
      const detail = response.error ?? "MCP bridge rejected call";
      const row = writeAgentAuditRow(
        opts, toolCall.name, toolCall.input, target, cls, "error", detail,
      );
      return {
        tool: toolCall.name, args: toolCall.input, outcome: "error",
        audit_id: row.id, reason: detail, classifier: cls,
        classifier_blocked: false, via: "mcp",
      };
    }
    const raw = response.result;
    const resultObject = raw && typeof raw === "object"
      ? raw as Record<string, unknown> : { value: raw };
    const serverOk = resultObject["ok"] !== false && !response.is_error;
    const outcome: "ok" | "refused" | "error" = serverOk
      ? "ok"
      : (typeof resultObject["gate"] === "string"
        || String(resultObject["reason"] ?? "").includes("grant")
        ? "refused" : "error");
    const auditId = Number(resultObject["audit_id"]);
    const reason = resultObject["reason"] ?? resultObject["error"];
    const preview = JSON.stringify(resultObject).slice(0, TOOL_RESULT_PREVIEW_CHARS);
    // Server-gated tools return their authoritative audit_id. If a
    // future MCP tool fails before it can audit, add a local error row.
    if (!Number.isFinite(auditId)) {
      const row = writeAgentAuditRow(
        opts, toolCall.name, toolCall.input, target, cls, outcome,
        typeof reason === "string" ? reason : undefined,
      );
      return {
        tool: toolCall.name, args: toolCall.input, outcome,
        audit_id: row.id, reason: typeof reason === "string" ? reason : undefined,
        result_preview: preview, classifier: cls,
        classifier_blocked: false, via: "mcp",
      };
    }
    return {
      tool: toolCall.name, args: toolCall.input, outcome,
      audit_id: auditId,
      reason: typeof reason === "string" ? reason : undefined,
      result_preview: preview, classifier: cls,
      classifier_blocked: false, via: "mcp",
    };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    const row = writeAgentAuditRow(
      opts, toolCall.name, toolCall.input, target, cls, "error", detail,
    );
    return {
      tool: toolCall.name, args: toolCall.input, outcome: "error",
      audit_id: row.id, reason: detail, classifier: cls,
      classifier_blocked: false, via: "mcp",
    };
  }
}


function normalizePlan(value: unknown): AgentPlan | null {
  if (!value || typeof value !== "object") return null;
  const obj = value as Record<string, unknown>;
  const calls = obj["tool_calls"];
  if (!Array.isArray(calls)) return null;
  const toolCalls: AgentToolCall[] = [];
  for (const raw of calls) {
    if (!raw || typeof raw !== "object") return null;
    const item = raw as Record<string, unknown>;
    if (typeof item["name"] !== "string" || !SUPPORTED_TOOLS.has(item["name"]))
      return null;
    if (!item["input"] || typeof item["input"] !== "object"
        || Array.isArray(item["input"])) return null;
    toolCalls.push({
      name: item["name"],
      input: item["input"] as Record<string, unknown>,
    });
  }
  const stop = obj["stop_reason"];
  const stopReason: AgentPlan["stop_reason"] = stop === "max_steps"
    ? "max_steps" : (toolCalls.length > 0 ? "tool_use" : "end_turn");
  return {
    reasoning: typeof obj["reasoning"] === "string"
      ? obj["reasoning"] : "Ollama returned no reasoning field.",
    tool_calls: toolCalls,
    stop_reason: stopReason,
  };
}

async function planWithOllama(
  prompt: string,
  observations: string[],
  url: string,
  model: string,
): Promise<AgentPlan | null> {
  const available = [...SUPPORTED_TOOLS].join(", ");
  const messages: Array<{ role: "system" | "user"; content: string }> = [
    {
      role: "system",
      content: `You are the AIOS S-rank agent. Use only these canonical MCP tools: ${available}. ` +
        "Return strict JSON only: {reasoning:string, tool_calls:[{name:string,input:object}], " +
        "stop_reason:'tool_use'|'end_turn'}. Never invent a tool. Never put shell commands " +
        "in the response. The server owns authorization; request only the smallest action needed.",
    },
    { role: "user", content: prompt },
  ];
  if (observations.length > 0) {
    messages.push({
      role: "user",
      content: "Observed MCP results from prior steps. Re-plan from these facts; do not repeat " +
        "a refused action unless the user prompt clearly requires a different safe action:\n" +
        observations.join("\n"),
    });
  }
  try {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), OLLAMA_TIMEOUT_MS);
    try {
      const response = await fetch(`${url}/api/chat`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          model,
          messages,
          stream: false,
          format: "json",
        }),
        signal: ctrl.signal,
      });
      if (!response.ok) return null;
      const body = await response.json() as { message?: { content?: string } };
      const text = body.message?.content;
      if (!text) return null;
      return normalizePlan(JSON.parse(text));
    } finally {
      clearTimeout(timer);
    }
  } catch {
    return null;
  }
}

function planWithStub(prompt: string, observations: string[]): AgentPlan {
  const lc = prompt.toLowerCase();
  const calls: AgentToolCall[] = [];
  let reasoning = "Deterministic fallback: Ollama unavailable; selected a bounded MCP read/action from prompt keywords.";
  if (/(audit\s*verify|verify\s*chain|chain\s*verify)/.test(lc)) {
    calls.push({ name: "aios.audit.verify", input: {} });
    reasoning = "Fallback plan: verify the audit chain through MCP.";
  } else if (/(audit\s*tail|tail\s*audit|recent\s*rows?)/.test(lc)) {
    const match = lc.match(/(\d+)/);
    calls.push({ name: "aios.audit.tail", input: { n: match ? Number(match[1]) : 10 } });
    reasoning = "Fallback plan: inspect recent audit rows through MCP.";
  } else if (/(list\s*process|process\s*list|ps\b)/.test(lc)) {
    calls.push({ name: "aios.process.list", input: {} });
    reasoning = "Fallback plan: list processes through MCP.";
  } else if (/(read\s+file|fs\.read|cat\s+)/.test(lc)) {
    const match = lc.match(/(?:read|fs\.read|cat)\s+([\w./-]+)/);
    if (match) {
      calls.push({ name: "aios.fs.read", input: { path: match[1] } });
      reasoning = `Fallback plan: read ${match[1]} through MCP.`;
    }
  } else if (/(scan|nmap|ports?\b|recon)/.test(lc)) {
    const match = lc.match(/(?:scan|nmap|recon)\s+([\w.:/-]+)/);
    if (match) {
      calls.push({ name: "pentest.nmap", input: { target: match[1] } });
      reasoning = `Fallback plan: nmap ${match[1]} through MCP; PEP grant required.`;
    }
  } else if (/(nikto|web\s*scan|webserver)/.test(lc)) {
    const match = lc.match(/(?:nikto|web)\s+([\w.:/-]+)/);
    if (match) calls.push({ name: "pentest.nikto", input: { target: match[1] } });
  }
  if (observations.length > 0 && calls.length > 0
      && observations[observations.length - 1]?.includes("refused")) {
    return { reasoning: "Fallback stopped after a refused MCP action.", tool_calls: [], stop_reason: "end_turn" };
  }
  return { reasoning, tool_calls: calls,
           stop_reason: calls.length > 0 ? "tool_use" : "end_turn" };
}


export async function runAgentLoop(opts: AgentLoopOptions): Promise<AgentLoopResult> {
  const bridge = new McpBridge();
  const steps: AgentStep[] = [];
  const observations: string[] = [];
  let totalToolCalls = 0;
  let totalRefused = 0;
  let model: { kind: "ollama" | "stub"; name: string } = {
    kind: "stub", name: "deterministic-stub",
  };
  let policyRevision = "sprint-2-rule-pack-v1";
  let stopReason: AgentLoopResult["stop_reason"] = "end_turn";
  let mcpTools: string[] = [];

  try {
    const ready = await bridge.start();
    mcpTools = ready.tools;
    const ollamaAvailable = await detectOllama(opts.ollama_url);
    if (ollamaAvailable) model = { kind: "ollama", name: opts.ollama_model };

    for (let i = 0; i < opts.max_steps; i++) {
      const ollamaPlan = model.kind === "ollama"
        ? await planWithOllama(opts.prompt, observations,
                              opts.ollama_url, opts.ollama_model)
        : null;
      const plan = ollamaPlan ?? planWithStub(opts.prompt, observations);
      policyRevision = classify("agent.step", undefined,
        { prompt: opts.prompt.slice(0, 256), step: i }).policy_revision;
      if (plan.tool_calls.length === 0 || plan.stop_reason === "end_turn") {
        steps.push({ step: i, plan, tool_results: [] });
        stopReason = "end_turn";
        break;
      }
      const results: AgentToolResult[] = [];
      for (const toolCall of plan.tool_calls) {
        totalToolCalls++;
        const result = await dispatchAgentToolCall(bridge, toolCall, opts);
        if (result.outcome === "refused") totalRefused++;
        results.push(result);
        observations.push(JSON.stringify({
          tool: result.tool, outcome: result.outcome,
          audit_id: result.audit_id, reason: result.reason,
          result_preview: result.result_preview,
        }));
      }
      steps.push({ step: i, plan, tool_results: results });
      if (results.every((item) => item.outcome === "refused")) {
        stopReason = "abort";
        break;
      }
      if (i === opts.max_steps - 1) stopReason = "max_steps";
    }
  } catch (error) {
    observations.push(`agent bridge failure: ${error instanceof Error ? error.message : String(error)}`);
    stopReason = "abort";
  } finally {
    bridge.close();
  }

  return {
    steps,
    total_steps: steps.length,
    stop_reason: stopReason,
    total_tool_calls: totalToolCalls,
    total_refused: totalRefused,
    classifier_policy_revision: policyRevision,
    model,
    mcp_tools: mcpTools,
  };
}

async function detectOllama(url: string): Promise<boolean> {
  try {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), 1_500);
    try {
      const response = await fetch(`${url}/api/tags`, { signal: ctrl.signal });
      return response.ok;
    } finally {
      clearTimeout(timer);
    }
  } catch {
    return false;
  }
}

export type { AuditRow };
