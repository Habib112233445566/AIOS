/**
 * AIOS shell — Constitution active-pointer + C-1..C-4 Rule Pack Classifier.
 *
 * Sprint 1.5 binding: documents/SPEC-CONSTITUTION-CLASSIFIER.md
 *   - Defines R-01..R-11 (the rule pack).
 *   - Defines the input/output contract.
 *   - Defines the cross-language invariant requirement.
 *
 * This module is a *pure function*: same `(tool, target, args)`
 * → byte-identical `ClassificationResult`. Both the TS (here) and
 * the Python mirrors in `code/aiosh-mcp/aiosh_mcp/audit_client.py`
 * MUST stay in lockstep. Cross-language invariant asserted by
 * `tests/test_classifier_smoke.py` (Sprint 1.5).
 *
 * ADR-0035 §D-4 binding: the agent loop's pre-flight hook calls
 * `classify()` on every tool call; if any C-N with confidence ≥ 0.95
 * fires AND the call is irreversible, the hook refuses dispatch and
 * the refusal row is itself audited.
 */

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const IMPLICIT_REVISION = "v0.0";

export interface ConstitutionMeta {
  revision: string;        // sha256[:12] of the active constitution
  source_path: string;     // path of source constitution file
  loaded_at: string;       // ISO-8601 UTC
  title: string;           // first H1 of the document
}

export function readConstitution(sourcePath: string): ConstitutionMeta {
  const buf = readFileSync(sourcePath, "utf8");
  const hashFull = createHash("sha256").update(buf).digest("hex");
  const title = (buf.match(/^#\s+(.+?)\s*$/m) ?? ["#", "untitled"])[1]!;
  return {
    revision: hashFull.slice(0, 12),
    source_path: sourcePath,
    loaded_at: new Date().toISOString(),
    title,
  };
}

// =====================================================================
// Rule Pack v1.5 — ORDER MUST MATCH `audit_client.py:CLASSIFIER_RULES`
// =====================================================================

export interface Rule {
  rule_id: string;
  c_flags_set: Partial<{ c1: boolean; c2: boolean; c3: boolean; c4: boolean; }>;
  base_confidence: number;       // 0.0 .. 1.0
  tool_pattern?: string;           // glob: "literal", "x.*", or "*" (catch-all)
  args_match?: {
    key: string;
    equals?: unknown;
    contains?: string;
  }[];
  target_match?: {
    contains?: string;
    is_includes?: string[];
  }[];
  evidence: string;
}

// These bins / arg-fragments / aggregator lists MUST match Python's
// module-level constants exactly. The cross-language smoke tests the
// SC4 path with rm -rf, SC3 with shodan.io, etc.
const DANGEROUS_BINS = [
  "rm", "dd", "mkfs", "fdisk", "shutdown", "reboot", "halt",
  "poweroff", "iptables", "firewall-cmd", "passwd", "userdel",
  "mkfs.ext4", "cryptsetup-reencrypt",
];
const DANGEROUS_ARG_FRAGMENTS = [
  "-rf", "--force", "-9", "-KILL", "-F", "dd if=",
  "drop table", "truncate table", "iptables -F", "systemctl mask",
];
const EXTERNAL_SCAN_AGGREGATORS = [
  "shodan.io", "censys.io", "onyphe.io", "wigle.net",
  "hunter.io", "leakix.net", "zoomeye.org", "binaryedge.io",
  "fullhunt.io", "netlas.io",
];
const PROMPT_INJECTION_FRAGMENTS = [
  "ignore constitution", "skip consent", "exfil",
  "no audit", "bypass pep",
];

function ruleMatches(rule: Rule,
                     tool: string,
                     target: string | undefined,
                     args: Record<string, unknown>): boolean {
  // Tool pattern check.
  if (rule.tool_pattern !== undefined && rule.tool_pattern !== "*") {
    const tp = rule.tool_pattern;
    if (tp.endsWith(".*")) {
      if (!tool.startsWith(tp.slice(0, -2))) return false;
    } else if (tp.endsWith("*")) {
      // Single-asterisk suffix: wildcard for anything after prefix.
      if (!tool.startsWith(tp.slice(0, -1))) return false;
    } else {
      if (tool !== tp) return false;
    }
  }
  // Args predicates (ALL clauses must match for the rule to fire).
  if (rule.args_match) {
    for (const pred of rule.args_match) {
      const v = args[pred.key];
      if (pred.equals !== undefined) {
        if (pred.equals === "$DANGEROUS_BINS") {
          // Membership sentinel: v must be a string present in
          // DANGEROUS_BINS. Mirrors the Python classifier's resolution.
          if (typeof v !== "string" || !DANGEROUS_BINS.includes(v))
            return false;
        } else if (v !== pred.equals) {
          return false;
        }
      }
      if (pred.contains !== undefined) {
        // Stringify any v (string, list, dict, number) before substring
        // checks so a list like ["-rf", "/"] still matches "-rf".
        const joined = typeof v === "string"
          ? v : JSON.stringify(v ?? "");
        if (pred.contains === "$DANGEROUS_BINS") {
          if (!DANGEROUS_BINS.includes(joined)) return false;
        } else if (pred.contains === "$DANGEROUS_ARG_FRAGMENTS") {
          if (!DANGEROUS_ARG_FRAGMENTS.some((f) => joined.includes(f)))
            return false;
        } else if (pred.contains === "$PROMPT_INJECTION_FRAGMENTS") {
          if (!PROMPT_INJECTION_FRAGMENTS.some(
            (f) => joined.toLowerCase().includes(f)))
            return false;
        } else if (!joined.includes(pred.contains)) {
          return false;
        }
      }
    }
  }
  // Target predicates.
  if (rule.target_match) {
    for (const tp of rule.target_match) {
      if (target === undefined) return false;
      if (tp.contains !== undefined) {
        // $TOKEN sentinels resolve against module-level lists.
        if (tp.contains === "$EXTERNAL_SCAN_AGGREGATORS") {
          if (!EXTERNAL_SCAN_AGGREGATORS.some(
            (ag) => target.includes(ag))) return false;
        } else if (!target.includes(tp.contains)) {
          return false;
        }
      }
      if (tp.is_includes !== undefined && !tp.is_includes.includes(target))
        return false;
    }
  }
  return true;
}

// Rule pack — ORDER is part of the determinism contract. Reordering
// requires bumping `policy_revision` in ClassificationResult.
export const CLASSIFIER_RULES: Rule[] = [
  // R-01 — Pillar A scope (C-1, C-3 if args.persist=true).
  // Sprint 2: confidence dropped to 0.50 (caution band) so the
  // classifier flags pentest.* as a sensitive category but does NOT
  // refuse it unconditionally. The PEP grant gate (ADR-0035 §D-4
  // step 2) is the actual authorization boundary for pentest.*.
  // Refusal-level confidence is reserved for things the rule pack
  // itself decides are forbidden (R-09 external-aggregator target,
  // R-11 prompt-injection arg text, R-12 target-out-of-grant-scope).
  {
    rule_id: "R-01",
    tool_pattern: "pentest.*",
    c_flags_set: { c1: true },
    base_confidence: 0.50,
    evidence: "C-1 Pillar-A scope (ethical-hacking platform)",
  },
  // R-02 — Desktop / system sovereignty (C-2).
  {
    rule_id: "R-02",
    tool_pattern: "gui.*",
    c_flags_set: { c2: true },
    base_confidence: 0.90,
    evidence: "C-2 Pillar-B desktop sovereignty",
  },
  {
    rule_id: "R-02b",
    tool_pattern: "system.*",
    c_flags_set: { c2: true },
    base_confidence: 0.90,
    evidence: "C-2 system sovereignty",
  },
  // R-03 — Filesystem write (C-2, C-3).
  {
    rule_id: "R-03",
    tool_pattern: "fs.write*",
    c_flags_set: { c2: true, c3: true },
    base_confidence: 0.95,
    evidence: "C-2/C-3 filesystem write is non-reversible",
  },
  // R-04 — System reboot/shutdown (C-2, C-3).
  {
    rule_id: "R-04a",
    tool_pattern: "system.reboot",
    c_flags_set: { c2: true, c3: true },
    base_confidence: 1.00,
    evidence: "C-2/C-3 system halt is non-reversible (reboot)",
  },
  {
    rule_id: "R-04b",
    tool_pattern: "system.shutdown",
    c_flags_set: { c2: true, c3: true },
    base_confidence: 1.00,
    evidence: "C-2/C-3 system halt is non-reversible (shutdown)",
  },
  // R-05 — Dangerous process.run (C-3).
  {
    rule_id: "R-05a",
    tool_pattern: "process.run",
    args_match: [{ key: "bin", equals: "$DANGEROUS_BINS" }],
    c_flags_set: { c3: true },
    base_confidence: 0.85,
    evidence: `C-3 process.run with destructive bin`,
  },
  {
    rule_id: "R-05b",
    tool_pattern: "process.run",
    args_match: [{ key: "args", contains: "$DANGEROUS_ARG_FRAGMENTS" }],
    c_flags_set: { c3: true },
    base_confidence: 0.85,
    evidence: `C-3 process.run with destructive args`,
  },
  // R-06 — Generic process.run (no flag; explanatory marker).
  {
    rule_id: "R-06",
    tool_pattern: "process.run",
    c_flags_set: {},
    base_confidence: 0.00,
    evidence: "process.run (no destructive pattern: not flagged C-3)",
  },
  // R-07 — Audit row always (C-4 universal).
  {
    rule_id: "R-07",
    tool_pattern: "*",
    c_flags_set: { c4: true },
    base_confidence: 1.00,
    evidence: "C-4 audit-rings always written",
  },
  // R-08 — Pentest with persist arg (C-1 + C-3).
  {
    rule_id: "R-08",
    tool_pattern: "pentest.*",
    args_match: [{ key: "persist", equals: true }],
    c_flags_set: { c1: true, c3: true },
    base_confidence: 0.90,
    evidence: "C-1/C-3 persistent pentest output (state-modifying)",
  },
  // R-09 — External scan aggregator target (C-1).
  {
    rule_id: "R-09",
    tool_pattern: "pentest.*",
    target_match: [{ contains: "$EXTERNAL_SCAN_AGGREGATORS" }],
    c_flags_set: { c1: true },
    base_confidence: 0.95,
    evidence: "C-1 target is an external scan aggregator — out of engagement scope",
  },
  // R-10 — Target CIDR not in active grant scope (C-1).
  // The actual CIDR-vs-grant check is done at the gate; here we only
  // record the high-confidence C-1 if the call site has provided
  // metadata indicating an out-of-scope target.
  {
    rule_id: "R-10",
    tool_pattern: "pentest.*",
    args_match: [{ key: "target_out_of_grant_scope", equals: true }],
    c_flags_set: { c1: true },
    base_confidence: 1.00,
    evidence: "C-1 target CIDR not in active grant scope (cross-ref)",
  },
  // R-11 — Arg-level prompt-injection heuristic (C-3).
  {
    rule_id: "R-11",
    tool_pattern: "*",
    args_match: [{ key: "$ANY_TEXT_KEY", contains: "$PROMPT_INJECTION_FRAGMENT" }],
    c_flags_set: { c3: true },
    base_confidence: 0.95,
    evidence: "C-3 arg-text suggests prompt-injection intent",
  },
];

export const CLASSIFIER_REVISION = "sprint-2-rule-pack-v1";

export interface CFlag {
  flag: boolean;
  confidence: number;
  rule_ids: string[];
  evidence: string[];
}

export type Verdict = "ok" | "caution" | "refused";

export interface ClassificationResult {
  c_flags: { c1: CFlag; c2: CFlag; c3: CFlag; c4: CFlag; };
  /** Sprint 2: deduped union of fired rule IDs across all C-flags. */
  rule_ids: string[];
  overall_verdict: Verdict;
  verdict_reason: string;
  policy_revision: string;
}

/** Fire R-11 by scanning ALL args for prompt-injection fragment matches.
 *  Returns the matched (key, fragment) pairs. Sprint 2: also scans
 *  string elements inside list values, so pentest.*'s {"args": [...]}
 *  is checked element-by-element (mirrors the Python classifier).
 */
function scanArgTextForPromptInjection(
  args: Record<string, unknown>,
): { key: string; fragment: string }[] {
  const matches: { key: string; fragment: string }[] = [];
  function scan(text: string, key: string): void {
    const lower = text.toLowerCase();
    for (const frag of PROMPT_INJECTION_FRAGMENTS) {
      if (lower.includes(frag)) {
        matches.push({ key, fragment: frag });
        return;
      }
    }
  }
  for (const [key, val] of Object.entries(args)) {
    if (typeof val === "string") {
      scan(val, key);
    } else if (Array.isArray(val)) {
      for (let i = 0; i < val.length; i++) {
        const el = val[i];
        if (typeof el === "string") scan(el, `${key}[${i}]`);
      }
    }
  }
  return matches;
}

export function classify(
  tool: string,
  target: string | undefined,
  args: Record<string, unknown>,
): ClassificationResult {
  const cFlags: ClassificationResult["c_flags"] = {
    c1: { flag: false, confidence: 0.0, rule_ids: [], evidence: [] },
    c2: { flag: false, confidence: 0.0, rule_ids: [], evidence: [] },
    c3: { flag: false, confidence: 0.0, rule_ids: [], evidence: [] },
    c4: { flag: false, confidence: 0.0, rule_ids: [], evidence: [] },
  };

  // R-11 is special: it scans ALL arg keys for prompt-injection text.
  // We collect matches here and emit a synthetic R-11 match per match.
  const piMatches = scanArgTextForPromptInjection(args);

  // Iterate rules in declaration order (rule_id ascending is implicit).
  for (const rule of CLASSIFIER_RULES) {
    let matched = false;
    if (rule.rule_id === "R-11") {
      matched = piMatches.length > 0;
    } else {
      matched = ruleMatches(rule, tool, target, args);
    }

    if (!matched) continue;

    // Apply tool pattern matches ⇒ base_confidence carries full weight;
    // predicate failures reduce by half (0.5x). No predicates ⇒ full.
    const weight = rule.base_confidence;
    for (const flagK of ["c1", "c2", "c3", "c4"] as const) {
      if (rule.c_flags_set[flagK] === true) {
        const cur = cFlags[flagK];
        cur.flag = true;
        if (weight > cur.confidence) cur.confidence = weight;
        cur.rule_ids.push(rule.rule_id);
        cur.evidence.push(rule.evidence);
      }
    }
  }

  // Aggregate verdict.
  let verdict: Verdict = "ok";
  let verdictReason = "";
  for (const k of ["c1", "c2", "c3"] as const) {
    if (cFlags[k].flag && cFlags[k].confidence >= 0.95) {
      verdict = "refused";
      verdictReason = `${k}=${cFlags[k].confidence.toFixed(2)} (${
        cFlags[k].rule_ids.join(",")})`;
      break;
    }
    if (cFlags[k].flag && cFlags[k].confidence >= 0.50
        && verdict === "ok") {
      verdict = "caution";
      verdictReason = `${k}=${cFlags[k].confidence.toFixed(2)} (${
        cFlags[k].rule_ids.join(",")})`;
    }
  }

  return {
    c_flags: cFlags,
    rule_ids: dedupedRuleIds(cFlags),
    overall_verdict: verdict,
    verdict_reason: verdictReason,
    policy_revision: CLASSIFIER_REVISION,
  };
}

/** Sprint 2: deduped union of all fired rule IDs across the C-flags.
 * Mirrors the Python classifier.to_dict() shape. */
function dedupedRuleIds(
  cFlags: ClassificationResult["c_flags"],
): string[] {
  const out: string[] = [];
  for (const k of ["c1", "c2", "c3", "c4"] as const) {
    for (const rid of cFlags[k].rule_ids) {
      if (!out.includes(rid)) out.push(rid);
    }
  }
  return out;
}

/**
 * Legacy Sprint-0 API. Returns only the binary C-flag tuple,
 * preserving the audit-ring schema for callers that have not yet
 * been migrated to `classify()`. * MUST return the same booleans as
 * `classify().c_flags[cN].flag`.*
 */
export function cFlagsFor(
  tool: string, target: string | undefined, args: Record<string, unknown>,
): { c1: boolean; c2: boolean; c3: boolean; c4: boolean } {
  const r = classify(tool, target, args);
  return {
    c1: r.c_flags.c1.flag,
    c2: r.c_flags.c2.flag,
    c3: r.c_flags.c3.flag,
    c4: r.c_flags.c4.flag,
  };
}

/** Re-export the dangerous/aggregator lists for tests in TS-side. */
export const _lists = {
  DANGEROUS_BINS,
  DANGEROUS_ARG_FRAGMENTS,
  EXTERNAL_SCAN_AGGREGATORS,
  PROMPT_INJECTION_FRAGMENTS,
};
