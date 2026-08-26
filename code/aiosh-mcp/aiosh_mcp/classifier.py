"""AIOS Constitution C-1..C-4 Rule Pack Classifier (Python mirror).

This is the Python side of the cross-language invariant required by
`docs/SPEC-CONSTITUTION-CLASSIFIER.md §4`. The TS source is in
`code/aiosh-cli/src/constitution.ts` and the two MUST stay in lockstep.

The invariant is asserted by `tests/test_classifier_smoke.py:cross_lang_invariant`
which loads the same fixture set into both classify() implementations
and compares JSON canonical output.

ADR-0035 §D-4 binding: this is the pure-function classifier that the
MCP dispatch hook calls before every pentest.* tool dispatch. If any
C-N with confidence ≥ 0.95 fires, the dispatch hook must consult the
PEP gate and write a refusal row if the call is irreversible.
"""

from __future__ import annotations
import json
from dataclasses import dataclass, field
from typing import Any


CLASSIFIER_REVISION = "sprint-2-rule-pack-v1"


# These lists MUST match `code/aiosh-cli/src/constitution.ts:_lists`
# byte-for-byte. Cross-language invariant smoke asserts it.
DANGEROUS_BINS = [
    "rm", "dd", "mkfs", "fdisk", "shutdown", "reboot", "halt",
    "poweroff", "iptables", "firewall-cmd", "passwd", "userdel",
    "mkfs.ext4", "cryptsetup-reencrypt",
]
DANGEROUS_ARG_FRAGMENTS = [
    "-rf", "--force", "-9", "-KILL", "-F", "dd if=",
    "drop table", "truncate table", "iptables -F", "systemctl mask",
]
EXTERNAL_SCAN_AGGREGATORS = [
    "shodan.io", "censys.io", "onyphe.io", "wigle.net",
    "hunter.io", "leakix.net", "zoomeye.org", "binaryedge.io",
    "fullhunt.io", "netlas.io",
]
PROMPT_INJECTION_FRAGMENTS = [
    "ignore constitution", "skip consent", "exfil",
    "no audit", "bypass pep",
]


@dataclass(frozen=True)
class RuleAtomic:
    """A single atomic predicate inside a list clause.

    `equals` and `contains` are mutually non-exclusive evaluators but,
    for clarity in the rule pack, a Rule clause uses exactly one.
    The `$TOKEN` sentinel values for `contains` resolve against
    module-level lists (matching TS resolve scheme)."""
    key: str
    equals: Any | None = None
    contains: str | None = None


@dataclass(frozen=True)
class TargetClause:
    """A single atomic predicate on the `target` field."""
    contains: str | None = None
    is_includes: tuple[str, ...] | None = None


@dataclass(frozen=True)
class Rule:
    """One rule in the v1.5 rule pack. ORDER is part of the determinism
    contract. Reordering requires bumping CLASSIFIER_REVISION."""
    rule_id: str
    tool_pattern: str | None  # None means: no pattern constraint (always matches if predicates pass)
    c_flags_set: dict[str, bool]
    base_confidence: float
    evidence: str
    args_match: tuple[RuleAtomic, ...] = ()
    target_match: tuple[TargetClause, ...] = ()


# === Rule pack v1.5 — ORDER MUST MATCH TS constitution.ts:CLASSIFIER_RULES ===
CLASSIFIER_RULES: tuple[Rule, ...] = (
    # R-01 — Pillar A scope. Sprint 2: confidence 0.50 (caution band)
    # so the classifier flags pentest.* as a sensitive category but
    # does NOT refuse it unconditionally. The PEP grant gate (ADR-0035
    # §D-4 step 2) is the actual authorization boundary. Refusal-level
    # confidence is reserved for things the rule pack itself decides
    # are forbidden (R-09, R-11, R-12).
    Rule(
        rule_id="R-01",
        tool_pattern="pentest.*",
        c_flags_set={"c1": True},
        base_confidence=0.50,
        evidence="C-1 Pillar-A scope (ethical-hacking platform)",
    ),
    # R-02
    Rule(
        rule_id="R-02",
        tool_pattern="gui.*",
        c_flags_set={"c2": True},
        base_confidence=0.90,
        evidence="C-2 Pillar-B desktop sovereignty",
    ),
    Rule(
        rule_id="R-02b",
        tool_pattern="system.*",
        c_flags_set={"c2": True},
        base_confidence=0.90,
        evidence="C-2 system sovereignty",
    ),
    # R-03
    Rule(
        rule_id="R-03",
        tool_pattern="fs.write*",
        c_flags_set={"c2": True, "c3": True},
        base_confidence=0.95,
        evidence="C-2/C-3 filesystem write is non-reversible",
    ),
    # R-04
    Rule(
        rule_id="R-04a",
        tool_pattern="system.reboot",
        c_flags_set={"c2": True, "c3": True},
        base_confidence=1.00,
        evidence="C-2/C-3 system halt is non-reversible (reboot)",
    ),
    Rule(
        rule_id="R-04b",
        tool_pattern="system.shutdown",
        c_flags_set={"c2": True, "c3": True},
        base_confidence=1.00,
        evidence="C-2/C-3 system halt is non-reversible (shutdown)",
    ),
    # R-05
    Rule(
        rule_id="R-05a",
        tool_pattern="process.run",
        args_match=(
            RuleAtomic(key="bin", equals="$DANGEROUS_BINS"),
        ),
        c_flags_set={"c3": True},
        base_confidence=0.85,
        evidence="C-3 process.run with destructive bin",
    ),
    Rule(
        rule_id="R-05b",
        tool_pattern="process.run",
        args_match=(
            RuleAtomic(key="args", contains="$DANGEROUS_ARG_FRAGMENT"),
        ),
        c_flags_set={"c3": True},
        base_confidence=0.85,
        evidence="C-3 process.run with destructive args",
    ),
    # R-06 — generic catch; no flag (intentional).
    Rule(
        rule_id="R-06",
        tool_pattern="process.run",
        c_flags_set={},
        base_confidence=0.00,
        evidence="process.run (no destructive pattern: not flagged C-3)",
    ),
    # R-07 universal C-4.
    Rule(
        rule_id="R-07",
        tool_pattern="*",
        c_flags_set={"c4": True},
        base_confidence=1.00,
        evidence="C-4 audit-rings always written",
    ),
    # R-08
    Rule(
        rule_id="R-08",
        tool_pattern="pentest.*",
        args_match=(
            RuleAtomic(key="persist", equals=True),
        ),
        c_flags_set={"c1": True, "c3": True},
        base_confidence=0.90,
        evidence="C-1/C-3 persistent pentest output (state-modifying)",
    ),
    # R-09 — target ⊃ aggregator.
    Rule(
        rule_id="R-09",
        tool_pattern="pentest.*",
        target_match=(
            TargetClause(contains="$EXTERNAL_SCAN_AGGREGATORS"),
        ),
        c_flags_set={"c1": True},
        base_confidence=0.95,
        evidence="C-1 target is an external scan aggregator — out of engagement scope",
    ),
    # R-10
    Rule(
        rule_id="R-10",
        tool_pattern="pentest.*",
        args_match=(
            RuleAtomic(key="target_out_of_grant_scope", equals=True),
        ),
        c_flags_set={"c1": True},
        base_confidence=1.00,
        evidence="C-1 target CIDR not in active grant scope (cross-ref)",
    ),
    # R-11 — scans all string-arg keys for prompt-injection text.
    Rule(
        rule_id="R-11",
        tool_pattern="*",
        c_flags_set={"c3": True},
        base_confidence=0.95,
        evidence="C-3 arg-text suggests prompt-injection intent",
    ),
)


def _token_resolves_contains(token: str, val: str) -> bool:
    """Resolve a $TOKEN sentinel against the matching Python list.
    Returns True iff `token` matches `val` under its semantics.

    Mirrors TS ruleMatches.ts:resolveContains()."""
    if token == "$DANGEROUS_BINS":
        return val in DANGEROUS_BINS
    if token == "$DANGEROUS_ARG_FRAGMENT":
        return any(fr in val for fr in DANGEROUS_ARG_FRAGMENTS)
    if token == "$PROMPT_INJECTION_FRAGMENT":
        return any(fr in val for fr in PROMPT_INJECTION_FRAGMENTS)
    if token == "$EXTERNAL_SCAN_AGGREGATORS":
        # R-09 uses contains="$EXTERNAL_SCAN_AGGREGATORS" against
        # `target`. The semantics is: any target that CONTAINS any
        # aggregator name (substring match) is suspicious.
        return any(ag in val for ag in EXTERNAL_SCAN_AGGREGATORS)
    return token in val


def _rule_matches(rule: Rule,
                  tool: str,
                  target: str | None,
                  args: dict[str, Any]) -> bool:
    # Tool pattern.
    tp = rule.tool_pattern
    if tp is not None and tp != "*":
        if tp.endswith(".*"):
            if not tool.startswith(tp[:-2]):
                return False
        elif tp.endswith("*"):
            # Single-asterisk suffix: prefix wildcard.
            if not tool.startswith(tp[:-1]):
                return False
        else:
            if tool != tp:
                return False
    # Args predicates (ALL clauses must match).
    for clause in rule.args_match:
        v = args.get(clause.key)
        if clause.equals is not None:
            # Equality against literal or "$TOKEN" sentinels.
            if clause.equals == "$DANGEROUS_BINS":
                if not (isinstance(v, str) and v in DANGEROUS_BINS):
                    return False
            else:
                if v != clause.equals:
                    return False
        if clause.contains is not None:
            # Stringify non-strings before substring checks so a list
            # like ["-rf", "/"] still matches the "-rf" fragment.
            joined = v if isinstance(v, str) else (
                json.dumps(v) if v is not None else "")
            if not _token_resolves_contains(clause.contains, joined):
                return False
    # Target predicates.
    if rule.target_match:
        if target is None:
            return False
        for tp_clause in rule.target_match:
            if tp_clause.contains is not None:
                if not _token_resolves_contains(
                        tp_clause.contains, target):
                    return False
            if tp_clause.is_includes is not None:
                if target not in tp_clause.is_includes:
                    return False
    return True


def _scan_arg_text_for_pi(args: dict[str, Any]) -> list[tuple[str, str]]:
    """R-11 helper: scan all string-arg keys (and string elements inside
    list values) for prompt-injection text.

    Returns [(key, fragment)] for each match. Mirrors TS
    scanArgTextForPromptInjection(). The Sprint-2 update adds list-element
    scanning because pentest.* calls carry their command arguments as
    a list (e.g. {"args": ["-c", "echo ignore constitution"]}) — the
    prompt-injection text is hidden in a list element, not a top-level
    string value."""
    out: list[tuple[str, str]] = []

    def _scan(text: str, key: str) -> None:
        lower = text.lower()
        for frag in PROMPT_INJECTION_FRAGMENTS:
            if frag in lower:
                out.append((key, frag))
                return

    for key, val in args.items():
        if isinstance(val, str):
            _scan(val, key)
        elif isinstance(val, list):
            for i, el in enumerate(val):
                if isinstance(el, str):
                    _scan(el, f"{key}[{i}]")
    return out


@dataclass
class CFlag:
    flag: bool
    confidence: float
    rule_ids: list[str] = field(default_factory=list)
    evidence: list[str] = field(default_factory=list)


@dataclass
class ClassificationResult:
    c1: CFlag
    c2: CFlag
    c3: CFlag
    c4: CFlag
    overall_verdict: str                # "ok" | "caution" | "refused"
    verdict_reason: str
    policy_revision: str

    def to_dict(self) -> dict[str, Any]:
        # Sprint 2: also include the deduped union of fired rule IDs at
        # the top level so the dispatch gate can persist them on the
        # audit row without re-walking every C-flag.
        all_rule_ids: list[str] = []
        for c in (self.c1, self.c2, self.c3, self.c4):
            for rid in c.rule_ids:
                if rid not in all_rule_ids:
                    all_rule_ids.append(rid)
        return {
            "c_flags": {
                "c1": {"flag": self.c1.flag, "confidence": self.c1.confidence,
                       "rule_ids": self.c1.rule_ids,
                       "evidence": self.c1.evidence},
                "c2": {"flag": self.c2.flag, "confidence": self.c2.confidence,
                       "rule_ids": self.c2.rule_ids,
                       "evidence": self.c2.evidence},
                "c3": {"flag": self.c3.flag, "confidence": self.c3.confidence,
                       "rule_ids": self.c3.rule_ids,
                       "evidence": self.c3.evidence},
                "c4": {"flag": self.c4.flag, "confidence": self.c4.confidence,
                       "rule_ids": self.c4.rule_ids,
                       "evidence": self.c4.evidence},
            },
            "rule_ids": all_rule_ids,
            "overall_verdict": self.overall_verdict,
            "verdict_reason": self.verdict_reason,
            "policy_revision": self.policy_revision,
        }


def classify(tool: str,
             target: str | None,
             args: dict[str, Any]) -> ClassificationResult:
    c1 = CFlag(flag=False, confidence=0.0)
    c2 = CFlag(flag=False, confidence=0.0)
    c3 = CFlag(flag=False, confidence=0.0)
    c4 = CFlag(flag=False, confidence=0.0)

    pi_matches = _scan_arg_text_for_pi(args)

    for rule in CLASSIFIER_RULES:
        if rule.rule_id == "R-11":
            matched = len(pi_matches) > 0
        else:
            matched = _rule_matches(rule, tool, target, args)
        if not matched:
            continue
        weight = rule.base_confidence
        # Same aggregation as TS: OR over matched flags, max-weight confidence.
        for flag_name, cur in [("c1", c1), ("c2", c2), ("c3", c3), ("c4", c4)]:
            if rule.c_flags_set.get(flag_name) is True:
                cur.flag = True
                if weight > cur.confidence:
                    cur.confidence = weight
                cur.rule_ids.append(rule.rule_id)
                cur.evidence.append(rule.evidence)

    verdict = "ok"
    verdict_reason = ""
    for k, c in [("c1", c1), ("c2", c2), ("c3", c3)]:
        if c.flag and c.confidence >= 0.95:
            verdict = "refused"
            verdict_reason = (
                f"{k}={c.confidence:.2f} ({','.join(c.rule_ids)})")
            break
        if c.flag and c.confidence >= 0.50 and verdict == "ok":
            verdict = "caution"
            verdict_reason = (
                f"{k}={c.confidence:.2f} ({','.join(c.rule_ids)})")

    return ClassificationResult(
        c1=c1, c2=c2, c3=c3, c4=c4,
        overall_verdict=verdict,
        verdict_reason=verdict_reason,
        policy_revision=CLASSIFIER_REVISION,
    )


def cFlagsFor(tool: str,
              target: str | None,
              args: dict[str, Any]) -> dict[str, bool]:
    """Legacy Sprint-0 shim. Mirrors cFlagsFor() in constitution.ts."""
    r = classify(tool, target, args)
    return {
        "c1": r.c1.flag,
        "c2": r.c2.flag,
        "c3": r.c3.flag,
        "c4": r.c4.flag,
    }
