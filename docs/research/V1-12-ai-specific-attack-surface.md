> **V2 AMENDMENT (2026-08-20):** The product vision has been restated. AIOS is *"a Linux system for ethical hacking on the inside, a Windows-style desktop on the outside, with AI as a first-class S-rank kernel subsystem."* Three pillars: Pillar A (Linux ethical-hacking), Pillar B (Windows-like desktop), Pillar C (S-rank AI subsystem). This research file is preserved as **research substrate** informing userspace capability / IPC / scheduler / PEP / MCP designs; it is no longer the shipping-path definition. See `README.md` and `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0).

# Research Report V1.12: AI-Specific Attack Surface for AI-Native Operating Systems

**Volume:** V1 — Foundations & Computer Science
**Status:** Complete
**Version:** 1.0
**Author:** AI Research Architect
**Date:** 2026-08-07
**Task:** AIOS-0013 (COMPLETED)
**Dependencies:** AIOS-0001 (Threat Model / V1-06), AIOS-0007 (AI Theory & ML / V1-05)
**Related:** V1-10 (AI Privilege Model), THREAT-0003 (Agent Privilege Escalation), V9.01 (Security Architecture v1.2), V9.02 (Security & Capabilities Architecture), RFC-0007 (AI Agent Privilege Model), SYS-0006 (Agent Delegation TLA+), V11.01 (AI Runtime)

---

## Abstract

This report performs deep research into the AI-specific attack surface of an AI-native operating system. It goes beyond the general AI-native threat model (AIOS-0001/V1-06) and the agent privilege-escalation model (AIOS-0002/THREAT-0003) to analyze, for each major AI attack family, the exact attack mechanics, the OS-level trust boundaries they cross, and the mitigations the OS can enforce — distinguishing what must be kernel-guaranteed (structure) from what remains model-verified (semantics). Seven attack families are covered: prompt injection, model extraction via API, training data poisoning, agent manipulation, capability escalation through natural language, multi-agent collusion, and model supply chain attacks. The report concludes with a defense-in-depth stack for AINOS, a mapping to existing AINOS mechanisms, a risk register update, and recommendations.

The central finding: **nearly all AI-specific attacks are ultimately *command* attacks — they attempt to induce the agent to invoke capabilities, tools, or delegation in ways the legitimate user did not intend.** The OS cannot make the model trustworthy, but it can make the *effects* of model misbehavior structurally bounded through the capability system, the PEP/Intent layer, and the audit subsystem. AINOS's existing architecture (V9.02, RFC-0007, kernel/src/ipc/pep.rs) already implements most of the required containment; this report validates that direction and identifies the residual gaps.

---

## 1. Research Questions

1. What are the distinct, mechanically different AI attack families, and where does each one enter the OS trust boundary?
2. Which attacks can the OS defeat *structurally* (capabilities, MAC, kernel mediation) and which can it only *detect* (anomaly detection, audit)?
3. Where does the PEP/intent layer sit relative to the model and to the capability system?
4. How do 2025-2026 research proposals (ProbeLogits, Governed MCP, SEAgent MAC, PeerGuard, AgenticOS intent ABI, OWASP Top 10 for LLM/Agentic Applications) map onto AINOS's existing mechanisms?
5. What is the minimum set of kernel changes needed to close the highest-severity AI attack vectors?

---

## 2. Threat Actors (AI-Specific)

| Actor | Motivation | Access | Primary Target |
|-------|-----------|--------|----------------|
| **Adversarial Prompt Injector** | Manipulate agent behavior via crafted input | Any input channel the agent reads (user, tool output, documents, web, email, IPC) | Prompt boundary, tool-selection loop |
| **Compromised Agent** | Escalate privileges, exfiltrate data | Valid capabilities, full agent API | Capability system, delegation chain |
| **Malicious Sub-Agent** | Inherit then exceed delegated authority | Capabilities attenuated from parent | Delegation chain, manifest enforcement |
| **Model Extractor** | Steal model weights / replicate decision boundary | Inference API, logit endpoints | Inference service, TEE, KV cache |
| **Data/Model Poisoner** | Corrupt model behavior, insert backdoors | Training pipeline, model store, fine-tuning feed | Model weights, RAG corpus, LoRA adapters |
| **Colluding Agents** | Bypass per-agent security by combining capabilities | Multiple valid agent identities | Inter-agent IPC, audit suppression |
| **Supply Chain Attacker** | Backdoor models/tools at distribution time | Model registry, MCP tool registry, package mirror | Model store, tool manifest, boot artifacts |

---

## 3. Attack Family 1: Prompt Injection

### 3.1 Mechanics

Prompt injection overrides or supplements the agent's system instructions with attacker-controlled content. The OS-relevant distinction is the **injection channel** — where the hostile content enters the agent's context:

| Channel | Vector | OS Trust Boundary Crossed |
|---------|--------|---------------------------|
| **Direct** | Malicious text in the user-facing input path (intent bar, voice, pasted document) | Intent ABI / PEP input moderation |
| **Indirect** | Content the agent *fetches*: web pages, emails, documents, tool output, IPC messages | Tool-call response path → context assembly |
| **Multi-modal** | Payloads embedded in images, audio, video | Media decode / vision pipeline → context assembly |
| **Persistent** | Injection written to agent memory (episodic, RAG cache) that re-executes on later sessions | Memory service → context assembly |
| **Recursive / self-exfiltration** | Agent reads its own generated output, which embeds injected instructions | Output path → next-step context |
| **Tool-output injection** | A tool's response (e.g., a fetched URL's content) contains control text | Tool response channel |

The 2025-2026 literature is unambiguous that this is the most prevalent and dangerous family. The 247-paper survey (arXiv:2606.10749) and Ferrag et al. (arXiv:2506.23260) both place injection variants at the top of agent threat rankings. Notably, "AI Agents May Always Fall for Prompt Injections" (Abdelnabi et al., arXiv:2605.17634) argues that with enough task complexity, adversarial content, or multi-step chains, *even well-defended agents* will eventually follow injected instructions — the model-level defense has no guaranteed fix.

### 3.2 OS-Level Mitigations

The OS cannot stop the model from *being* confused, but it can control what a confused model can *do*:

1. **Capability containment (primary, structural):** An injected agent holds only the capabilities derived from its manifest. Blast radius is bounded even if injection fully succeeds. This is the *load-bearing* mitigation — V1-06 and V9.02 already commit to it.
2. **PEP enforcement on every tool call (structural):** AINOS's `pep_check` (kernel/src/ipc/pep.rs) interposes on ToolCall/CapInvoke/Delegate/DataAccess actions with PERMIT/DENY/Audit decisions. Tool-name hashing (FNV-1a) means only manifest-declared tools resolve; injected instructions cannot mint new tool capabilities.
3. **Intent ABI separation (semantic + structural):** Per AgenticOS (arXiv:2606.21129), the OS separates user intent from tool output at the ABI level. Tool output is *data*, not *instructions* — it cannot directly drive the next capability invocation without passing the intent/PEP boundary. AINOS's three-layer ABI (V2.04) and Intent ABI (Layer 3) implement this.
4. **Input/output moderation interception points:** Microsoft's Agent Control Specification (ACS) eight interception points (pre_tool_call, post_model_call, input_moderation, pre_delegation, post_delegation, pre_output, post_output, error_boundary) map onto AINOS's capability invocation path — the PEP is the natural place to install these hooks.
5. **Detection (probabilistic):** Logit-based safety primitives (ProbeLogits, arXiv:2604.11943; Governed MCP, arXiv:2604.16870) detect unsafe tool invocations at inference time *below* the agent's privilege boundary. This is defense-in-depth, optional, and under evaluation in AIOS-0016.

**Verdict:** AINOS is architecturally sound on this family. The kernel guarantees effect-containment; model robustness remains the model's job.

---

## 4. Attack Family 2: Model Extraction via API

### 4.1 Mechanics

| Technique | Description | OS Surface |
|-----------|-------------|------------|
| **Query-based extraction** | Systematic input-output probing to reconstruct the model or its decision boundary | Inference API rate/usage accounting |
| **Logit-based extraction** | Exploiting exposed logits/log-probabilities to clone with far fewer queries | Logit channel, KV cache |
| **Side-channel extraction** | Timing/memory patterns reveal architecture or weights | Scheduler, TEE, KV cache tiering |

OWASP LLM10:2025 ("Unbounded Consumption & Model Extraction") classifies this as an LLM-specific vulnerability. Logit access dramatically reduces the query budget needed to clone a model — protecting the logit channel is the single highest-leverage OS mitigation.

### 4.2 OS-Level Mitigations

1. **Per-agent usage caps (structural):** The capability system can carry *resource quotas* (query count, tokens, rate) as part of a capability's metadata, enforced at the inference-service boundary — not in the model server, but in the kernel-mediated capability invocation.
2. **Logit access restriction (structural):** Top-k logits/log-probabilities are treated as a privileged channel, delivered only to capabilities explicitly granting them. Default agents receive sampling output only.
3. **Watermarking / fingerprinting (producer-side):** Embedding statistical watermarks in output distributions so cloned models can be detected downstream. The OS's role is to ensure the watermark survives the output pipeline (no stripping by intermediate agents).
4. **TEE-sealed weights (hardware):** Model weights resident in a TEE with signed attestation; the OS enforces that inference memory is sealed, and KV cache eviction to untrusted tiers is prevented for sensitive models (ties to AIOS-0029's memory tiers — `Pinned` tier hint).

**Verdict:** Mostly an inference-service concern, but AINOS can enforce quota + logit privilege + TEE sealing at the OS boundary.

---

## 5. Attack Family 3: Training Data Poisoning & Backdoor Insertion

### 5.1 Mechanics

| Attack | Description |
|--------|-------------|
| **Backdoor insertion** | Poison training data so a trigger reliably induces attacker-chosen behavior at inference |
| **Data contamination** | Corrupt the corpus to degrade quality / induce biases |
| **Fine-tuning hijacking** | Hostile dataset or LoRA adapter hijacks an otherwise clean base model |
| **Environment poisoning (2025-2026)** | Attackers poison the *external* content agents ingest during autonomous operation (web, tool descriptions, RAG sources) — effectively data poisoning at inference time (Boisvert et al., arXiv:2510.05159) |

The "Malice in Agentland" work (arXiv:2510.05159) documents the agent-specific supply chain: backdoors don't have to be in the base model — they can live in LoRA adapters, MCP tool definitions, RAG documents, or agent prompt templates.

### 5.2 OS-Level Mitigations

1. **Signed provenance chain (structural):** Every model, adapter, and tool manifest is content-addressed (hash) and signed. The OS refuses to load unsigned or mismatched artifacts. AINOS's Trust Boundary 5 (AI Runtime → Model) already requires hash + signature verification.
2. **RAG corpus integrity (structural):** Content-addressed memory (V1-06 §3.6) makes tamper-evident retrieval stores. Poisoned corpus entries fail integrity checks rather than entering agent context.
3. **Environment-poisoning containment (structural):** The PEP's tool-response path means externally fetched content is *data*, subject to input moderation at the PEP — a hostile URL's content cannot directly mint capabilities.
4. **Differential privacy / sanitization (training-side):** Out of OS scope, but the OS must *enforce* that training runs in isolated domains with audited data flows.

**Verdict:** AINOS's content-addressed loading + PEP moderation covers the OS-relevant part. Model-internal backdoors (trigger behaviors that activate against *authorized* capabilities) remain undetectable structurally — this is where ProbeLogits-style detection (AIOS-0016) or runtime anomaly detection (audit subsystem) is the only backstop.

---

## 6. Attack Family 4: Agent Manipulation (Goal Hijacking, Plan & Argument Control)

### 6.1 Mechanics

| Attack | Description | OS-Relevant Step |
|--------|-------------|------------------|
| **Goal hijacking** | Attacker overrides the agent's goal mid-task | None structurally — happens inside the model |
| **Plan manipulation** | Attacker steers the multi-step plan toward dangerous tool sequences | Tool-call sequence visible to PEP/audit |
| **Tool selection manipulation** | Attacker forces use of a dangerous tool | PEP tool-name check — blocked if not in manifest |
| **Argument manipulation** | Attacker controls tool-call arguments | PEP/argument validation — blocked if schema-violating |
| **Context drift** | Slow deviation from intent over long sessions | Audit: cross-session anomaly detection |

### 6.2 OS-Level Mitigations

1. **PEP action validation (structural):** Tool calls are validated against the manifest *and* argument schemas (SYS-0006 P7 — output schema conformance; THREAT-0003 AS-10 — strict schema validation). Argument manipulation that produces schema-violating or capability-violating calls is denied at the PEP.
2. **Capability-argument binding (structural):** Capability invocation carries intent; the kernel checks the invoked operation is within the capability's rights (already the case in MVP-M2's `derive.rs` rights narrowing).
3. **Delegation interception (structural):** `pre_delegation`/`post_delegation` hooks (ACS) installed in the delegation path enforce DCC P1-P7 — a hijacked agent cannot delegate authority it does not hold (THREAT-0003 AS-2, AS-6).
4. **Audit + anomaly detection (detective):** AINOS's audit subsystem (AIOS-0042/0055) records every capability invocation; cross-session context drift or plan anomalies surface as audit anomalies. Plan-trajectory alignment verification (per Aura architecture, cited in V1-06) is a detector, not a preventer.

**Verdict:** Goal/plan manipulation cannot be prevented by the OS; its *effects* can be contained (PEP) and *detected* (audit). Argument manipulation is structurally defeated by schema + capability validation.

---

## 7. Attack Family 5: Capability Escalation Through Natural Language

### 7.1 Mechanics

This is the AI-specific variant of the **confused deputy** and **privilege escalation** problems, first formalized for agents by Ji et al. (arXiv:2601.11893, SEAgent). The agent is the deputy: it holds real capabilities (file write, network, shell), and prompt-injected or socially-engineered natural language causes it to chain those capabilities into actions the user never authorized. Because the agent acts in natural language, an attacker needs no memory-corruption exploit — only *wording*. The same paper's SEAgent framework and THREAT-0003 (AIOS-0002 deliverable) catalog the escalation paths:

- AS-1 Manifest bypass, AS-4 right amplification, AS-6 depth evasion, AS-9 policy weakening (THREAT-0003).
- NL-driven chaining: benign-capability composition into privileged workflows (e.g., read config + write config + execute = privilege escalation assembled from three individually-benign tools).

### 7.2 OS-Level Mitigations

1. **SEAgent-style MAC (structural):** Ji et al. propose an attribute-based MAC over agent-tool interactions with an information-flow graph (IFG) that blocks unauthorized privilege escalation. AINOS's PEP + manifest model is functionally a simplified MAC; the IFG is the enrichment — capability invocations can be checked against an information-flow policy (who may read/write what, derived from manifest trust levels).
2. **Monotonic rights narrowing (structural):** MVP-M2's derivation rules enforce `rights(child) ⊆ rights(parent)`; a compromised agent cannot mint wider capabilities (THREAT-0003 AS-4; V1-10 §2.1).
3. **High-risk operation gating (semantic + structural):** Operations crossing a risk threshold require MFA or fresh user confirmation (THREAT-0003 AS-3 mitigation). The PEP can mark high-risk tool names and force an audit+confirm path.
4. **Confused-deputy elimination (structural):** No ambient authority — every action passes through the agent's own capability set (V1-06 §2.2). The attacker cannot "borrow" the system's privileges through the agent.

**Verdict:** The capability system is the *only* defense that scales here — the OS guarantees that whatever natural language achieves inside the model, the *invocations* are still checked against the manifest. THREAT-0003 already covers the mechanism detail; V1-12 adds the SEAgent IFG as a formal enrichment.

---

## 8. Attack Family 6: Multi-Agent Collusion

### 8.1 Mechanics

| Attack | Description |
|--------|-------------|
| **Agent-to-agent injection** | A malicious agent sends injection to a peer via IPC |
| **Collusion for capability combination** | Two+ agents each with disjoint capabilities combine them through coordinated tool calls to achieve what neither could alone |
| **Horizontal delegation / depth evasion** | Sibling agents form a delegation "ring" to reset depth counters (THREAT-0003 AS-6) |
| **Audit suppression** | Colluding agents distribute malicious actions so no single agent's audit trail looks anomalous |

### 8.2 OS-Level Mitigations

1. **Absolute chain depth (structural):** Depth is measured from the delegation-chain root, not relative — sibling delegation preserves depth (THREAT-0003 AS-6 fix already adopted).
2. **Per-agent capability bounds + audit (structural + detective):** Each agent's audit log is separate; combination attacks require cross-agent correlation. The audit subsystem's anomaly detection must support *cross-agent* correlation (collusion detection; RISK-0025).
3. **PeerGuard mutual reasoning (detective, model-level):** Fan & Li (arXiv:2505.11642) propose agents reviewing each other's reasoning traces. This is a model-level defense the OS can *facilitate* (by providing verifiable execution traces) but not guarantee.
4. **IPC mediation (structural):** Inter-agent communication rides AINOS's capability-mediated IPC (V5.02); an agent cannot open an endpoint to another agent without the right capability. Collusion is bounded to agents that are *supposed* to communicate.

**Verdict:** Structurally bounded (IPC + depth + per-agent caps); residual risk is coordinated action across legitimately-communicating agents — requires cross-agent audit correlation, and is the justification for the audit subsystem's design.

---

## 9. Attack Family 7: Model Supply Chain Attacks

### 9.1 Mechanics

Attackers upload trojaned base models, LoRA adapters, MCP tool definitions, or prompt templates to registries; victims deploy them and the backdoor activates later. AINOS-specific surfaces:

| Surface | Attack |
|---------|--------|
| Model store (weights) | Compromised weights with trigger behavior (RISK-0011) |
| Tool registry (MCP tools) | Malicious tool that looks benign in manifest, misbehaves at runtime |
| Agent templates / prompt packs | Backdoored agent "personalities" that leak data |
| Boot/update path | Compromised update artifacts |

### 9.2 OS-Level Mitigations

1. **Cryptographic provenance (structural):** Content-addressable + signed artifacts at *every* load point: model weights, adapters, tool manifests, agent templates. AINOS already verifies model hash + signature (Trust Boundary 5); extend the same to tool manifests (RFC-0007 manifest-only runtime).
2. **Manifest-only runtime (structural):** RFC-0007's manifest-only runtime means a tool's *declared* surface is all it can ever do. A backdoored MCP tool still cannot exceed its manifest capabilities — the backdoor's blast radius is the manifest, not the machine.
3. **Governed MCP six-layer validation (structural + detective):** Son's Governed MCP (arXiv:2604.16870) proposes schema validation, trust tiering, adversarial pre-filtering, and logit safety gates for every MCP tool call. AINOS's PEP (which already gates tool calls via hash_tool_name + manifest lookup + trust levels) is the natural home; the six layers are a checklist to extend it against.
4. **Quarantine + rollback (operational):** The recovery strategy from V1-06 (block load, alert, quarantine artifact) applies; the OS must support signed-rollback of agent/tool state.

**Verdict:** AINOS's manifest + signature posture directly addresses this family. The gap is *post-deployment* behavior verification (a legitimately-signed malicious tool), which is PEP/detection territory.

---

## 10. OS-Level Defense-in-Depth Stack (Synthesis)

| Layer | Mechanism | AINOS Status | Attack Families Covered |
|-------|-----------|--------------|------------------------|
| **L0 — Hardware/TEE** | Sealed weights, attested inference, IOMMU | Partially (TEE in V9.01); AIOS-0016 pending | Extraction, supply chain |
| **L1 — Kernel capability system** | CDT + CloudCaps guards + monotonic narrowing + per-PD quotas | ✅ Implemented (MVP-M2) | Escalation-NL, injection effects, collusion |
| **L1.5 — Resource quotas** | Query/rate/token caps on inference capabilities | Gap — to add | Extraction |
| **L2 — Delegation (DCC)** | P1-P7, absolute depth, signed DCT | ✅ Specified (SYS-0006, RFC-0007) | Collusion, escalation-NL |
| **L2.5 — PEP** | Manifest check, tool hash, action/schema validation, high-risk gating | ✅ Implemented (kernel/src/ipc/pep.rs) | Injection, manipulation, escalation-NL, supply chain |
| **L3 — Intent ABI** | Intent as first-class primitive; tool output ≠ instructions | ✅ Specified (V2.04) | Injection (indirect/multi-modal/persistent) |
| **L4 — Audit** | Capability-invocation log + cross-agent anomaly correlation | ✅ Logging implemented; correlation is a gap | Manipulation, collusion, escalation |
| **L5 — Optional logit primitives** | ProbeLogits / Governed MCP safety gates | ⚠️ Under evaluation (AIOS-0016) | Injection, poisoning backdoors |

**Key architectural insight:** layers L1-L4 are *structural* — they hold even if the model is fully compromised. L5 is *semantic* — it can fail, so it must never be load-bearing.

---

## 11. Risk Register Additions (AIOS-0013)

| Risk ID | Description | Severity | Mitigation Owner |
|---------|-------------|----------|------------------|
| RISK-0022 | Indirect/persistent prompt injection defeats intent verification (P2 probabilistic) | Critical | L1 containment + L2.5 PEP (containment primary) |
| RISK-0023 | Logit channel exposure enables cheap model cloning | High | Logit-capability restriction + quotas (L1.5) |
| RISK-0024 | Environment poisoning of RAG/tool-fetch content enters agent context | High | Content-addressed memory + PEP input moderation |
| RISK-0025 | Cross-agent collusion evades single-agent audit | High | Cross-agent audit correlation (L4 gap) |
| RISK-0026 | Signed-but-malicious MCP tool ships a backdoor within manifest scope | Medium | Governed MCP six-layer validation + behavior detection |
| RISK-0027 | Post-quantum threat to DCT/Ed25519 delegation signatures | Medium | PQ migration plan (CRYSTALS-Dilithium) |

---

## 12. Recommendations for AINOS

1. **Ratify the containment-first posture:** The OS's job is to make model compromise *expensive* (bounded blast radius), not to make the model safe. No changes to the core capability architecture (V9.02/ADR-0010) are needed as a result of this research — the AIOS-0002 verdict holds.

2. **Add resource-quota metadata to capabilities (L1.5):** Extend `Capability` with optional quota fields (query rate, token budget, call count) enforced at the inference-service capability invocation. This is the single most concrete new kernel mechanism this research calls for, and it directly closes the model-extraction family (RISK-0023).

3. **Treat the logit channel as a privileged capability:** Top-k logits/log-probabilities require an explicit logit-granting capability; default agents get sampling output only.

4. **Implement cross-agent audit correlation (L4):** Extend the audit subsystem's anomaly detection to correlate capability invocations across agent IDs (collusion detection, RISK-0025). This is a natural follow-on task.

5. **Reuse Governed MCP's six-layer checklist to harden the PEP:** schema validation, trust tiering, adversarial pre-filtering, logit gates — as a hardening pass on kernel/src/ipc/pep.rs (follow-on task).

6. **Proceed with AIOS-0016 (ProbeLogits evaluation) as defense-in-depth only:** L5 must remain optional and never load-bearing, given the architectural cost (GPU in kernel) and probabilistic nature.

7. **Keep the OWASP Top 10 for LLM/Agentic Applications and CSA Agentic Trust Framework as the certification checklist** for agents and tools entering the system (imported from V1-10 §7).

---

## 13. Validation Against Existing AINOS Architecture

| AINOS Commitment | This Research | Verdict |
|------------------|---------------|---------|
| Capability containment as primary defense (V9.02, ADR-0010) | Confirmed as *the* load-bearing mitigation for injection/manipulation/escalation | ✅ Validated |
| Manifest-only runtime (RFC-0007) | Directly defeats supply-chain tool backdoors (blast radius = manifest) | ✅ Validated |
| PEP enforcement on tool calls (kernel/src/ipc/pep.rs) | Matches Governed MCP's kernel-level tool governance and ACS interception points | ✅ Validated (extendable) |
| Intent ABI separation (V2.04, AgenticOS) | The only structural counter to indirect/multi-modal injection | ✅ Validated |
| Content-addressed model loading (Trust Boundary 5) | Required for supply-chain + poisoning families | ✅ Validated |
| Three-layer authorization (RFC-0007) | Convergent with SEAgent MAC + SentinelAgent three-point verification | ✅ Validated |
| Audit subsystem (AIOS-0042) | Necessary but insufficient alone — needs cross-agent correlation | ⚠️ Gap identified |
| ProbeLogits / Governed MCP kernel primitives | Defense-in-depth only; not load-bearing (AIOS-0016) | ⚠️ Pending evaluation |

**Conclusion:** The 2025-2026 literature on AI agent security converges on exactly the architecture AINOS has already committed to (capability containment + manifest + PEP + intent separation + audit). No architectural reversals are required. The concrete next steps are additive: resource quotas (L1.5), logit privilege, and cross-agent audit correlation.

---

## 14. References

1. Anonymous (2026). Toward Secure LLM Agents: Threat Surfaces, Attacks, Defenses, and Evaluation. *arXiv:2606.10749* (247-paper survey).
2. Ferrag, M. A., et al. (2025). From Prompt Injections to Protocol Exploits: Threats in LLM-Powered AI Agents. *arXiv:2506.23260*.
3. Abdelnabi, S., et al. (2026). AI Agents May Always Fall for Prompt Injections. *arXiv:2605.17634*.
4. Ji, et al. (2026). Taming Various Privilege Escalation in LLM-Based Agent Systems: A Mandatory Access Control Framework (SEAgent). *arXiv:2601.11893*.
5. Boisvert, J., et al. (2025). Malice in Agentland: Down the Rabbit Hole of Backdoors in the AI Supply Chain. *arXiv:2510.05159*.
6. Fan, M., Li, Y. (2025). PeerGuard: Defending Multi-Agent Systems Against Backdoor Attacks Through Mutual Reasoning. *arXiv:2505.11642*.
7. Zhao, Z., et al. (2026). AgenticOS: An Intent-Oriented Secure Operating System Architecture for Autonomous AI Agents. *arXiv:2606.21129*.
8. Son, D. (2026). Governed MCP: Kernel-Level Tool Governance for AI Agents via Logit-Based Safety Primitives. *arXiv:2604.16870*.
9. Son, D. (2026). ProbeLogits: Kernel-Level LLM Inference Primitives for AI-Native Operating Systems. *arXiv:2604.11943*.
10. Pirch, L., et al. (2026). Toward Securing AI Agents Like Operating Systems. *arXiv:2605.14932*.
11. OWASP (2025). OWASP Top 10 for LLM & Agentic Applications — LLM10:2025 Unbounded Consumption / Model Extraction via API.
12. Cloud Security Alliance (2026). Agentic Trust Framework and Research Notes.
13. Microsoft (2026). Agent Control Specification (ACS) — Eight standardized interception points for agent governance.
14. Klein, G., et al. (2009). seL4: Formal verification of an OS kernel. *SOSP*.
15. Watson, R. N. M., et al. (2015). CHERI: A hybrid capability-system architecture. *IEEE S&P*.
16. Patil, K. (2026). SentinelAgent: Intent-Verified Delegation Chains. *arXiv:2604.02767*.

---

## 📅 Day Tracking

| Field | Value |
|-------|-------|
| Task | AIOS-0013 |
| Started | 2026-08-07 |
| Completed | 2026-08-07 |
| Estimated | 10 days |
| Actual | 1 day |
| Days Saved | 9 days |
| Status | ✅ COMPLETED (9 days ahead of schedule) |
