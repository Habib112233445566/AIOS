# AIOS — Classifier Primitive & Neural Substrate Research Note

> **Purpose.** Bridge the abstract neuron / Dynamic Neural Topology
> research substrate (carried in
> `docs/research/AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md` and
> `docs/research/AIOS-SUPERINTELLIGENCE-2026-08-20.md`) into the
> concrete, shipped AIOS rule-pack classifier primitive. Make the
> relationship — and the gap — explicit.
>
> **Scope.** This note answers three questions, honestly:
>
> 1. What is a neuron (and what is *not* one)?
> 2. What is the rule-pack classifier primitive that AIOS ships in
>    Sprint 1.5 + Sprint 2, and where does it sit on the
>    neuron-vs-rule spectrum?
> 3. What does this mean for Pillar-C strategy: is the shipped
>    classifier a stand-in for a future neuron-inspired topology, or a
>    separate, deliberate, principled boundary?
>
> The short answer to (3) is: **separate, deliberate, principled
> boundary.** The classifier is not on the path toward a brain; it is
> the *audit reproducibility boundary* that lets us keep an LLM agent
> in scope. The neuron-inspired dynamic topology is a parallel
> research substrate we preserve (V1 substrate + the DNT note) but
> don't ship.
>
> **Authoring note.** Sourced from the user's earlier research
> conversations (Vertus AI vendor assessment + neuron-as-parameter
> distinction + spiking-neuron frontier), Wikipedia authoritative
> articles on Neuron / Artificial Neuron / Spiking Neural Network /
> Action Potential / Neuroplasticity, Anthropic Constitutional AI
> (arXiv:2212.08073), and the shipped AIOS code at Sprint 2.

---

## 1. What a neuron is — and what the word is used to mean

### 1.1 A real biological neuron

A **biological neuron** is a living, excitable cell. Its defining
properties (from the *Neuron* and *Action potential* Wikipedia
articles):

- **Structure:** a soma (cell body), branching dendrites (input),
  a single axon (output cable, up to a metre long in humans), and
  axon terminals that form synapses with other cells.
- **Mechanism:** ion pumps + voltage-gated ion channels maintain a
  ~-70 mV membrane potential. When incoming synaptic currents push
  the potential above a ~-55 mV threshold, **voltage-gated sodium
  channels snap open** and the membrane fires an all-or-nothing
  electrochemical pulse called an **action potential** ("spike").
  Sodium inactivates, potassium opens, the membrane repolarises, and
  there is a brief **refractory period** during which the neuron
  cannot fire again. The spike propagates down the axon and triggers
  neurotransmitter release at the terminals.
- **Biology:** a human brain contains on the order of 86 billion
  neurons, each a metabolising, gene-expressing cell with up to ~10⁴
  synapses. A single cortical pyramidal neuron can receive input
  from >10 000 others.

The key takeaway: **a neuron is a stateful, time-dependent, threshold-
firing, self-rewiring physical cell.** It is not a number.

### 1.2 The four things called "neuron" in AI

When a vendor says "neuron," they are almost always using one of
four meanings — and three of those are not what the word means in
biology.

| # | Term | What it actually is | Where it's used | Equivalent |
|---|------|---------------------|------------------|------------|
| 1 | **Biological neuron** | Living cell with membrane potential, spikes, refractory period, neurochemistry | Real brains | 86 × 10⁹ in human neocortex |
| 2 | **Artificial neuron (McCulloch–Pitts, 1943)** | `y = φ(Σ wᵢ xᵢ + b)` — a weighted sum of inputs passed through a nonlinearity | Every dense layer of an LLM | A **parameter** |
| 3 | **Spiking neuron** | Time-dependent dynamical model: leaky integrate-and-fire, Izhikevich, Hodgkin–Huxley | SNN research, neuromorphic hardware (Intel Loihi, IBM TrueNorth) | Membrane potential + spike events |
| 4 | **Vendor-claimed "meganeuron"** (e.g. Vertus AI "8 Meganeuron Architecture") | Marketing label; the engineering is almost always #2 with a routing layer | Vendor pitch decks | A parameter wearing a neuron costume |

From the *Artificial neuron* Wikipedia article (verbatim):
> The classical description of neurons as "threshold logic" units …
> can be considered a **caricature model**.

That is, the artificial neuron is a *caricature* of the biological
neuron. The word "neuron" in an LLM context means **parameter**.

### 1.3 The honest separation

These four categories are not on a single spectrum. They are
different *kinds of things*. The risk is conflating them:

- "We have 8 million neurons" in the sense of LLM dense layers
  (parameters) is **not** remarkable; it is roughly one small dense
  layer.
- "We have 8 million neurons" in the sense of spiking neurons
  (Hodgkin–Huxley) **would** be remarkable; nobody ships that.
- "We have 8 million neurons" in the sense of biological neurons is
  **microscopic**; a fruit fly has more.

The audit reproducible boundary is whether the meaning is **stated
explicitly**. Saying "parameter" instead of "neuron" in the spec is
a small cost; using "neuron" to mean "parameter" in a vendor pitch
is a large confusion.

---

## 2. What "Dynamic Neural Topology" means in the literature

The user's earlier sessions referenced a vendor claim (Vertus AI)
that "our systems form new neural connections for every query using
Brain-like cognitive computing … Unique neural topology per query."
Translated into engineering terms, that claim has four components
(see `AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md` for the full
mapping):

1. A **large pool** of compute units ("millions").
2. A **per-query selector** that chooses which subset to wire up.
3. **Variable topology** — not just variable activation.
4. **Compute proportional to difficulty**.

Each of these is a real research area with shipped, partially-shipped,
and unbuilt variants:

| Component | Existing paradigm | Closest published | Status |
|-----------|-------------------|-------------------|--------|
| 1 — large pool | Mixture-of-Experts routing | Shazeer et al. 2017 ("Outrageously Large Neural Networks: the Sparsely-Gated Mixture-of-Experts Layer") | **Shipped at scale** at Google, Mistral, etc. |
| 2 — per-query selector | Sparsely-gated MoE top-k; Hash Layers | Shazeer 2017; Chen et al. 2021 (Hash Layers for large sparse models) | **Shipped** |
| 3 — variable topology | Dynamic Neural Networks survey (Han et al. 2021); DENs | Dynamic neural network architectures | **Research** (sample efficient, training-stable) |
| 3′ — generated weights | Hypernetworks | Ha et al. 2016 | **Niche** |
| 4 — variable compute | Adaptive Computation Time (ACT) | Graves 2016 | **Shipped** in some research systems |
| Biologically plausible | Spiking Neural Networks + neuromorphic | Maass 1997; Intel Loihi; IBM TrueNorth | **Hardware-shipped** (Intel Loihi 2), **not competitive with LLMs** on language |
| Self-rewiring | Continual learning, Dynamically Expandable Networks | Yoon et al. 2018 | **Research**; catastrophic forgetting is unsolved |
| Per-query topology generation | **No published evidence of end-to-end real systems that generate a brand-new topology per query** | — | **Vendor claim without published proof** |

The literature-level honest position: **components 1, 2, 4 are
shipped at scale; component 3 is research; biological plausibility is
shipped only in non-language domains; per-query topology generation
is unverified.**

The Dynamic Neural Topology note in this repo concludes the same:
"this is real research but not the same thing as a complete
artificial nervous system. … When somebody says 'unique neural
topology per query,' ask for the BPF / the routing trace / the
hash function. If they cannot produce it, the claim is marketing."

---

## 3. The Vertus AI vendor assessment (extracted from earlier sessions)

From the user's earlier deep-dive into vertus.ai (full URL capture
in the historical session memory):

**Positional claims.** "Architecting the Future of Superintelligence.
A cognitive intelligence company building systems that manage
billions, reason across domains, and evolve toward consciousness."

**Product lines.** Intelligent Investing (algorithmic trading);
Financial Infrastructure; Superintelligence API; Consciousness
Research. Tier-1 engagements reportedly require \$10M–\$50M AUM.

**Models.** "8 Meganeuron Architecture" (`8Mn-General`, `8Mn-Coding`)
and "16 Meganeuron Architecture" (`16Mn-General`, `16Mn-Coding`).
"More advanced systems coming soon."

**Founders.** Alexander W. Foster (algorithmic strategy background);
Michal Prywata (MIT-incubated medical robotics, "world's first
plant communication networks"); Julius Franck (retail algorithmic
infrastructure).

**Dynamic Neural Topology claim (verbatim from the user's paste):**

> *Our systems form new neural connections for every query using
> Brain-like cognitive computing. Millions of neurons interconnect in
> configurations adapted specifically to your problem. Simple
> question? Lightweight pathway. Complex reasoning? Dense
> interconnected topology. Like a brain forming new synapses.
> Query → Dynamic formation → Reasoning pathways → Decision. Unique
> neural topology per query.*

**Honest engineering translation, by clause:**

| Claim | Translation | Where this exists |
|-------|-------------|--------------------|
| "form new neural connections for every query" | A routing layer selects which sub-network handles the query | MoE (Shazeer 2017) |
| "Brain-like cognitive computing" | Marketing term; the engineering is conventional deep learning | No published vendor evidence |
| "Millions of neurons interconnect" | A pool of dense-layer parameters, possibly gated | Standard MoE |
| "configurations adapted specifically to your problem" | Routing depends on the input | MoE gating |
| "Simple question? Lightweight pathway" | Conditional compute (ACT or early-exit) | Graves 2016, Graves 2013 |
| "Complex reasoning? Dense interconnected topology" | Routing picks more experts | MoE gating |
| "Like a brain forming new synapses" | **This is the load-bearing claim.** It requires weight generation / connection formation at inference time. **No published evidence of this in any production-scale language system.** | Hypernetworks (research niche), Dynamic Neural Networks (research) |
| "Unique neural topology per query" | **Requires that no two queries share a routing path** — at scale this is neither implemented nor typically desirable (cache locality, amortised cost). | Not a published vendor system |

The honest assessment: the first six rows are real, and most of them
exist in published MoE + ACT systems. The seventh and eighth rows
are the actual load-bearing claim, and **there is no public evidence
that Vertus has built them.** Until the vendor publishes a technical
report or independent third party validates a benchmark, "Unique
neural topology per query" is best treated as **MoE + ACT + a
nervous-system metaphor.**

### 3.1 Open vendor-due-diligence questions
(For a future session where the user has more time, or for an
external researcher to file.)

1. **Where is the technical report?** No whitepaper on arXiv, no
   NeurIPS / ICML paper from the founders as of 2026-08-20.
2. **What does "meganeuron" measure?** Parameters (boring) or
   spiking units (extraordinary)? Both? An MoE expert (boring)? A
   re-routed cluster (interesting if true)?
3. **What is the routing function?** Sparsely-gated top-k? Hash
   function? A second neural net (hypernetwork)? A learned
   adjacency-matrix generator?
4. **Is the topology actually different per query, or are they
   claiming the activation pattern is different?** The latter is
   true of every neural net; the former is the claim that needs
   proof.
5. **Is there a deterministic replay?** Same query → same routing
   path → reproducible? If no, the system is non-deterministic at
   the routing layer; that has real engineering and audit
   consequences.
6. **Independent benchmark?** Any third-party evaluation of the
   8Mn-Coding model on standard coding benchmarks (HumanEval,
   MBPP, SWE-bench)?
7. **Domain age and corporate registration.** vertus.ai was
   registered 2025 (per WHOIS-style queries earlier in the
   session); three years is short for "we built an S-rank
   substrate." Not disqualifying, but a flag.
8. **Plant communication network** — Michal Prywata's background.
   Which lab, which paper, what was the actual experimental setup?
9. **"Broker money flow" data product** — independently documented?
10. **Founders' track record** — verifiable academic or industry
    publications under the same names?

---

## 4. The AIOS rule-pack classifier primitive — what it IS

The shipped AIOS rule-pack classifier (Sprint 1.5 + Sprint 2) is
**not** on the neuron-or-MoE-or-DNT spectrum at all. It is a
**deterministic, hash-stable rule pack** that gates every MCP tool
call before it reaches the agent loop or the host. Its
characteristics:

| Property | Value |
|----------|-------|
| Topology | **12 named rules** in fixed order (R-01 … R-12) |
| Topology changes per call? | **No.** Same `(tool, target, args)` tuple always yields the same verdict for a given `policy_revision`. |
| Inference time | < 1 ms per call |
| Self-rewiring? | **No.** New rules are added by human PR + `policy_revision` bump. |
| Variable compute? | **No.** All 12 rules are evaluated in order; an early short-circuit may skip the rest, but the *cost* is bounded by `O(rules)` and identical for any input that fails the same prefix. |
| Biologically inspired? | **No.** It is a rule pack modelled on a security policy, not on a neuron. |
| Audit reproducible? | **Yes, byte-for-byte.** The full `{policy_revision, classify_rule_ids, classify_evidence, classify_overall_verdict, classify_verdict_reason}` is persisted in the audit row; the chain hash is computed over a canonical proto that includes them. Verified byte-identical in `tests/test_classifier_smoke.py` Section D. |

The rule pack is **a safety boundary**, not a cognition engine. Its
job is to make a decision that:

1. The audit ring can **prove** was made (via `policy_revision` +
   rule IDs + evidence).
2. The agent loop **cannot bypass** (the gate fires before the MCP
   dispatch).
3. An LLM cannot talk its way past (deterministic — the model has
   no influence over `R-09` firing on `shodan.io`).

### 4.1 Why a rule pack, not an LLM judge

The Anthropic Constitutional AI paper (Bai et al. 2022,
arXiv:2212.08073) places LLM-judge feedback at **training time**. The
inference-time application of the Constitution is a *separate*
problem: it must be reproducible, hash-stable, fast, and adversarial-
robust. An LLM judge fails all four:

- **Reproducibility:** the same prompt can yield different
  judgements across calls (temperature ≠ 0) or models.
- **Hash-stable:** the canonical proto for the audit chain must
  produce identical bytes across Python and TypeScript writers;
  an LLM judge cannot.
- **Speed:** LLM-judge adds hundreds of ms to thousands of ms per
  tool call.
- **Adversarial-robustness:** prompt-injection into the judge input
  is a real attack surface.

The AIOS rule pack trades a smaller surface area for these four
properties. It is the **right primitive for the inference-time
boundary**. It is **not** the right primitive for cognition.

### 4.2 Where the rule pack could grow

The rule pack is bounded by what humans can enumerate. New attack
categories, new tools, new MITRE ATT&CK tactics → new R-rule +
`policy_revision` bump. This is the **explicit, version-stamped,
audited growth path**. It is slow and human-in-the-loop on purpose.

A *different* growth path — LLM-judge added at training time to
*propose* new rules — would trade reproducibility for coverage. We
deliberately do not do this yet; it would re-introduce the four
problems above.

---

## 5. Mapping back to Pillar-C strategy

AIOS Pillar-C is "AI as an S-rank kernel subsystem that controls
everything." ADR-0035 defines the S-rank subsystem. The Sprint 1.5
+ Sprint 2 work places the rule-pack classifier at the **safety
boundary**, not at the **cognition engine**. The architecture is:

```text
  user prompt
       │
       ▼
  agent loop (Ollama 0.22.1 / stub)
       │
       ▼
  planned tool call: (tool, target, args)
       │
       ▼
  ─── rule-pack classifier ─── gate #1
       │ (verdict: ok | caution | refused)
       │
       ▼
  ─── PEP grant check ──────── gate #2
       │
       ▼
  audit row (hash-chained, classifier-verdict-attached)
       │
       ▼
  MCP dispatch / host execution
       │
       ▼
  audit row (with sandbox components, exit code, etc.)
```

The **classifier** is a deterministic, hash-stable gate. The
**agent** is the cognition engine (today: an LLM via Ollama;
research-substrate future: a neuron-inspired dynamic topology). The
two are deliberately separated so that:

- The classifier can be **audited, replayed, tested, and shipped**
  without depending on the agent's training or weights.
- The agent can be **swapped** (Ollama, Claude, a future
  neuron-inspired topology, a third-party API) without changing
  the audit invariants.
- A future neuron-inspired dynamic topology agent would still go
  through the **same rule-pack classifier** before issuing any
  MCP tool — and the audit ring would still prove which `R-N`
  decided each action. The substrate changes; the boundary does
  not.

This is the **correct separation of concerns**. Conflating "the
agent that plans" with "the rule that authorises" would put a
neural network on both sides of the audit boundary, which makes
reproducibility impossible.

---

## 6. Honest position: the classifier is not "toward a brain"

The user's earlier intuition that "we could build a whole system
that works like a neuron system" is **correct** — and is preserved
in this repo as the V1 microkernel substrate + the DNT research
note. The Sprint-1.5 + Sprint-2 classifier is **not** a step toward
that goal. It is a step toward a **different** goal: an auditable,
reproducible, hash-stable inference-time safety boundary that works
whether the agent is an LLM or a future neuron-inspired topology.

Concretely:

| If the future agent is … | The rule-pack classifier still works because … |
|--------------------------|-------------------------------------------------|
| A 7B-parameter LLM (today) | R-01..R-12 don't care about the model's weights; they inspect the (tool, target, args) tuple. |
| A 1000B-parameter MoE LLM (next year) | Same. Routing is internal to the agent; the boundary is at the tool call. |
| A spiking-neural-network agent (research) | Same. The agent's topology is its concern; the rule pack observes only what crosses the tool-call boundary. |
| A dynamic-topology-of-neurons agent (research) | Same. Per-query topology happens inside the agent. The rule pack sees one (tool, target, args) tuple per call and decides. |

The classifier is therefore **a stable interface contract**, not a
stepping stone. It can be improved (more rules, lower false-positive
rate, calibrated confidence) without changing its role. It can be
backed by an LLM judge in *training mode* (to propose new rules)
without changing its role. But it should never *be* an LLM judge.

---

## 7. Where this leaves Pillar-C

Three deliberate choices are now locked in:

1. **The rule-pack classifier is the inference-time safety boundary.**
   Deterministic, hash-stable, audit-reproducible. Sprint 1.5 + Sprint 2.
2. **The agent loop is a separate, pluggable component.** Today
   Ollama + deterministic stub; tomorrow whatever the user
   prefers. Sprint 2.
3. **The neuron-inspired dynamic-topology substrate is preserved
   as research, not shipped.** The DNT note + V1 substrate are in
   the repo for traceability. They are **not** in the
   audit-critical path. They are **not** the rule-pack. They are
   **not** the agent. They are a separate, slower, more
   speculative research direction.

This three-way split — **deterministic safety boundary /
pluggable agent / preserved-but-unshipped neuron substrate** — is
the honest Pillar-C architecture as of Sprint 2.

A future session could:
- expand the rule pack (more R-rules, version-stamped)
- swap the agent for a neuron-inspired topology (research, not
  shipping target)
- harden the kernel-sandbox host (so `aiosh run` Landlock
  actually applies, per §11 honest position)

without re-architecting any of the three.

---

## 8. References

- *Neuron.* Wikipedia. <https://en.wikipedia.org/wiki/Neuron>
- *Artificial neuron.* Wikipedia. <https://en.wikipedia.org/wiki/Artificial_neuron>
- *Spiking neural network.* Wikipedia. <https://en.wikipedia.org/wiki/Spiking_neural_network>
- *Action potential.* Wikipedia. <https://en.wikipedia.org/wiki/Action_potential>
- *Neuroplasticity.* Wikipedia. <https://en.wikipedia.org/wiki/Neuroplasticity>
- Bai, Y. et al. (2022). *Constitutional AI: Harmlessness from AI
  Feedback.* arXiv:2212.08073.
  <https://arxiv.org/abs/2212.08073>
- Shazeer, N. et al. (2017). *Outrageously Large Neural Networks:
  The Sparsely-Gated Mixture-of-Experts Layer.* arXiv:1701.06538.
  <https://arxiv.org/abs/1701.06538>
- Han, Y. et al. (2021). *Dynamic Neural Networks: A Survey.*
  arXiv:2102.04906. <https://arxiv.org/abs/2102.04906>
- Ha, D. et al. (2016). *HyperNetworks.* arXiv:1609.09106.
  <https://arxiv.org/abs/1609.09106>
- Yoon, J. et al. (2018). *Lifelong Learning with Dynamically
  Expandable Networks.* arXiv:1708.01547.
  <https://arxiv.org/abs/1708.01547>
- Graves, A. (2016). *Adaptive Computation Time for Recurrent
  Neural Networks.* arXiv:1603.08983.
  <https://arxiv.org/abs/1603.08983>
- Maass, W. (1997). *Networks of spiking neurons: The third
  generation of neural network models.* Neural Networks, 10(9).
- Intel. *Neuromorphic Computing.* Research page.
  <https://www.intel.com/content/www/us/en/research/neuromorphic-computing.html>
- AIOS internal docs:
  - `docs/research/AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md`
  - `docs/research/AIOS-SUPERINTELLIGENCE-2026-08-20.md`
  - `docs/SPEC-CONSTITUTION-CLASSIFIER.md` (the rule-pack spec)
  - `docs/SPRINT-0.md` §8, §9, §10, §11 (rule pack + agent + sandbox)

EOF
