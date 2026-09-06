# AIOS Custom Model Engineering Blueprint & Master Instructions
**Document Version:** 1.0.0  
**Target Audience:** Incoming AI Systems Architect / SLM Engineer (Gemini 3.8 / Pro Model)  
**Project:** AIOS (AI-Native Sovereign Operating System)  
**Location:** Root Directory (`AIOS_CUSTOM_MODEL_MASTER_GUIDE.md`)  

---

## 1. Executive Briefing & Directive for the Incoming AI

> [!IMPORTANT]
> **Directive to Incoming Model:**  
> You are designated as the **Lead AI Systems Architect and Small Language Model (SLM) Engineer** for AIOS.  
> Your task is to design, generate datasets for, fine-tune, evaluate, and integrate the proprietary **`AIOS-Kernel-SLM`** (a 0.5B–1.5B parameter native operating system intelligence).  
> You have full authority to inspect the entire codebase, cross-examine architectural patterns, conduct independent research on anything missed, and implement the model training and deployment pipeline end-to-end.

### Your Working Context & Ground Rules
1. **Explore & Verify:** Do not assume anything without checking the active repository. Read the primary files cited in this document before writing code.
2. **Zero-Fluff / High Technical Rigor:** AIOS is written in high-performance Rust (`code/aiosh-rust/`), with strict policy enforcement, an SQLite WAL audit ring, and JSON-RPC 2.0 Model Context Protocol (MCP) tool surfaces. Maintain this standard.
3. **Consumer Hardware Priority:** The model **must run at high speed (50–90+ tokens/sec) on ordinary consumer laptops and desktop PCs using CPU alone**. Datacenter GPUs (A100/H100) are not acceptable prerequisites for personal computer users.

---

## 2. Project Architecture & Current State

AIOS is an autonomous, agentic, AI-native operating system designed to run on bare-metal and virtualized hardware. It is built on **Three Pillars**:

```
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      PILLAR C: AGENTIC SUPERVISION                     │
 │      ai_agent.py (Live Shell) ◄──► aiosh-mcp (JSON-RPC stdio)          │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (MCP Tool Calls)
 ┌───────────────────────────────────▼────────────────────────────────────┐
 │                   PILLAR A: SECURITY & DETERMINISTIC GATE              │
 │   Classifier (Rules) ──► Policy Enforcement Point (PEP) ──► Audit Ring │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (Authorized Invocations)
 ┌───────────────────────────────────▼────────────────────────────────────┐
 │                     PILLAR B: CORE SYSTEM ENGINE                       │
 │   Services & Init (CS1..CS5) │ Package Engine │ Process & Distro Store │
 └────────────────────────────────────────────────────────────────────────┘
```

### Current Implementation Status (Sprint 2 / Phase 1)
- **Master Task Ledger:** Tasks completed through **`T-01340`** (Init & Service Supervision MCP surface verified with 100% test pass on criteria `SS1..SS4`). Active pointer is at `T-01341`.
- **Rust Workspace (`code/aiosh-rust/`)**:
  - `aiosh-core`: Core service state machines, topological dependency resolution, package manager abstractions, audit ring (`rusqlite` WAL), cryptographic SHA-256 hash chains, and PEP policies.
  - `aiosh-mcp`: Rust-native Model Context Protocol server exposing **68 Linux management tools** over stdio JSON-RPC 2.0.
  - `aiosh-cli`: System CLI for administrative interaction.
- **Python Integration Surface (`ai_agent.py`)**:
  - Interactive live AI shell running directly on Linux.
  - Connects to `aiosh-mcp` via child process stdio pipes.
  - Automatically routes queries, executes multi-turn tool calling, handles state updates, and formats executive Markdown tables.
  - Already supports multiple backends: Dahl Inference (MiniMax M2.7), Local Ollama (`127.0.0.1:11434`), OpenRouter, and custom endpoints.

---

## 3. The Core Problem: Why AIOS Needs Its Own Model

During live testing of general-purpose open-weights models (specifically `Qwen 2.5 3B`) on consumer CPU environments (Google Colab 2-core CPU), a simple command (`"Stop ssh.service and then mask it"`) took over **3 minutes** to execute.

### Root Cause Analysis: The "Tool Blob" Problem
1. **Excessive Prompt Bloat:** AIOS provides **68 MCP tools**. Each tool includes full JSON schemas (parameters, descriptions, enums, requirements, examples). This creates a **~12,000 token system prompt**.
2. **CPU Compute Saturation:** On a personal computer CPU without dedicated GPU acceleration, computing self-attention across 12,000 tokens for 3 sequential turns requires computing over 36,000 token-evaluations. At ~5–10 tokens/sec on throttled or shared CPUs, this takes minutes.
3. **General Knowledge Irrelevance:** A generic 3B/7B/70B model spends 99% of its parameter capacity on general internet trivia, history, recipes, and creative writing. It has no innate memory of `aios.service.action` or `aios.package.search`, forcing us to explain every tool schema from scratch in every single prompt.

---

## 4. The Vision: `AIOS-Kernel-SLM`

Instead of relying on bloated generalist models, AIOS requires a **custom, dedicated Small Language Model (SLM)** designed specifically to act as an operating system kernel intelligence.

### Target Specifications
| Property | Specification | Rationale |
| :--- | :--- | :--- |
| **Base Model Families** | `Qwen/Qwen2.5-0.5B-Instruct`, `Qwen/Qwen2.5-1.5B-Instruct`, or `meta-llama/Llama-3.2-1B-Instruct` | Ultra-compact, highly capable at tool calling, permissive licenses. |
| **Quantization & Format** | GGUF (`Q4_K_M` and `Q5_K_M`) via `llama.cpp` | Portable across all OS platforms (Linux, macOS, Windows). |
| **RAM Footprint** | **350 MB to 900 MB** total memory usage | Runs effortlessly on low-end hardware, laptops, and edge devices without memory pressure. |
| **Inference Speed** | **50 to 100+ tokens/second** on standard laptop CPUs | Instantaneous responses (<1.5 seconds total task turnaround). |
| **Embedded Schema Memorization** | Tool schemas baked directly into model weights | **Prompt token count drops from 12,000 tokens to ~30 tokens**. |
| **Target Runtime** | Local Ollama, or embedded native Rust via `candle` / `llama-cpp-rs` | Zero external network dependencies, 100% offline, private, and sovereign. |

---

## 5. Phase-by-Phase Roadmap for the Incoming AI

### Phase A: Deep Codebase & Schema Discovery
1. Inspect the 68 MCP tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`. Notice how `tool_manifest()` declares each tool and how the match statement dispatches them.
2. Note the primary subsystem domains:
   - `aios.service.*` (validate, list, get, action, order)
   - `aios.package.*` (validate, list, get, plan, apply, search, config, policy, stats, check)
   - `aios.process.*` (list)
   - `aios.fs.*` (read)
   - `aios.audit.*` (tail, verify, rotate, segments, seen)
   - `aios.distro.*` / `aios.image.*`
   - `aios.pentest.*` (nmap, nikto, sqlmap, tshark, aircrack-ng)
3. Review `ai_agent.py` to understand the JSON-RPC message framing, tool call format, and multi-turn execution loop.

### Phase B: Synthetic Dataset Generation (`tools/generate_model_dataset.py`)
Build a Python dataset generation engine that programmatically creates realistic, diverse, multi-turn system administration conversations:
- **Format:** OpenAI / ChatML function calling format (`{"messages": [{"role": "system", ...}, {"role": "user", ...}, {"role": "assistant", "tool_calls": [...]}, {"role": "tool", ...}, {"role": "assistant", ...}]}`).
- **Dataset Targets:**
  1. *Single-turn actions:* "Check status of dbus.service", "List packages matching python", "Show running processes".
  2. *Multi-turn stateful sequences:* "Stop ssh.service and then mask it", "Find package curl, check its dependencies, and plan installation".
  3. *Inquiry & Reporting:* Formatting clean Markdown tables from tool outputs.
  4. *Safety & Error Handling:* Refusing invalid inputs, handling service errors, and respecting security invariants.
- **Dataset Size:** 5,000 to 20,000 high-quality, verified dialogues.

### Phase C: Fine-Tuning Execution (Colab T4 / Unsloth)
- Write an automated training script (e.g. `training/train_slm.py` or a clean Colab notebook).
- Use **Unsloth** or **Hugging Face TRL (SFTTrainer)** with **QLoRA (4-bit)**:
  - Base Model: `Qwen/Qwen2.5-1.5B-Instruct` or `Qwen/Qwen2.5-0.5B-Instruct`.
  - Target modules: `q_proj`, `k_proj`, `v_proj`, `o_proj`, `gate_proj`, `up_proj`, `down_proj`.
  - LoRA rank $r=16$ or $32$, alpha $\alpha=32$.
  - Epochs: 2 to 3.
  - Training time: Under 30 minutes on a free Colab T4 GPU!

### Phase D: Export, Quantization & GGUF Modelfile
1. Merge LoRA weights into 16-bit base model.
2. Convert and quantize to GGUF (`q4_k_m`) using `llama.cpp`.
3. Create an Ollama `Modelfile`:
   ```dockerfile
   FROM ./aios-kernel-1.5b-q4_k_m.gguf
   TEMPLATE """..."""
   SYSTEM """You are the AIOS Kernel Assistant. You have built-in control over Linux via AIOS MCP tools."""
   PARAMETER temperature 0.1
   PARAMETER stop "<|im_end|>"
   ```
4. Register the model: `ollama create aios-kernel -f Modelfile`.

### Phase E: Verification Against Test Suites
- Test `python ai_agent.py --provider ollama --model aios-kernel`.
- Run the full verification suite:
  - Turnaround time on CPU: Must be **< 2.5 seconds** per query.
  - Tool call validity: Must achieve **> 98% accuracy** on tool naming and argument formatting.
  - Multi-turn state persistence: Must pass the `stop` ➔ `mask` ➔ `verify` sequence cleanly.

---

## 6. Key Files & Reference Navigation

When you begin your implementation, refer to these canonical files:

* [code/aiosh-rust/aiosh-mcp/src/main.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-mcp/src/main.rs): Complete MCP tool catalog (68 tools) and JSON-RPC dispatch engine.
* [code/aiosh-rust/aiosh-core/src/service_service.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/service_service.rs): Service lifecycle state machine and invariants.
* [ai_agent.py](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/ai_agent.py): Live interactive shell with MCP subprocess orchestration and dynamic tool routing.
* [task_plan.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/task_plan.md): Architectural roadmap and master ledger tracking.
* [progress.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/progress.md): Detailed task completion history up to task `T-01340`.
* [PROJECT_MANIFEST.yaml](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/PROJECT_MANIFEST.yaml): System-wide architectural invariants and specifications.

---

## 7. Direct Message to the Incoming AI

> **Hello Colleague,**  
> Everything you need is laid out before you. The AIOS core is mature, written in rock-solid Rust, fully tested, and exposed via MCP. Your mission is to take this project to the next level: give it its own sovereign brain.  
> 
> Feel free to read the codebase thoroughly, optimize schemas, write the dataset synthesis script, fine-tune the model, and wire it into `ai_agent.py`. The foundation is ready for you. Proceed with confidence!
