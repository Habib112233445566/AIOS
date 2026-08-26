> **V2 AMENDMENT (2026-08-20):** The product vision has been restated. AIOS is *"a Linux system for ethical hacking on the inside, a Windows-style desktop on the outside, with AI as a first-class S-rank kernel subsystem."* Three pillars: Pillar A (Linux ethical-hacking), Pillar B (Windows-like desktop), Pillar C (S-rank AI subsystem). This research file is preserved as **research substrate** informing userspace capability / IPC / scheduler / PEP / MCP designs; it is no longer the shipping-path definition. See `README.md` and `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0).

# Research Report V1.07: System ABI & Application Interface

**Volume:** V1 — Foundations  
**Status:** Draft  
**Version:** 1.0  
**Date:** 2026-07-16  

---

## Executive Summary

This report investigates the question: **What interface should an AI-native operating system expose to applications and agents?** Rather than comparing existing syscall mechanisms, we derive requirements from first principles and evaluate 15+ interface models against 11 criteria. The conclusion recommends a **three-layer hybrid architecture**: capability invocation at the kernel level, message passing at the system service level, and Intent ABI at the AI agent level.

---

## 1. First-Principles Derivation

### 1.1 What Must an OS Interface Provide?

From first principles, an OS interface is the contract between untrusted (or partially trusted) code and the system's privileged core. It must enable:

1. **Resource access** — read, write, execute, allocate, deallocate
2. **Identity management** — who or what is requesting, with what authority
3. **Operation dispatch** — what operation is being requested, on what object
4. **Authorization** — is the requestor permitted to perform this operation?
5. **Isolation** — does the request not violate other protection domains?
6. **Composition** — can results from one operation be fed into another?
7. **Audit** — can the operation be logged, traced, accounted?

### 1.2 AI-Native Requirements

AI agents introduce requirements absent from traditional OS design:

1. **Intent expression** — agents should declare *what* they want to accomplish, not *how*
2. **Dynamic capabilities** — agent needs vary per task; static permission sets are insufficient
3. **Semantic safety** — the interface must prevent "compositional escapes" (combining benign primitives into dangerous behavior)
4. **Multi-agent coordination** — agents need to delegate, share, and revoke access among themselves
5. **Audit at semantic level** — logs should record "agent summarized document" not "fd=3, read(fd, buf, 4096)"
6. **Non-generalizability** — the interface closure must not degenerate into a general syscall table

### 1.3 Security Requirements from V1.06 Threat Model

8 identified threat classes constrain interface design:
- **Prompt injection** → interface must support runtime revocation
- **Model extraction** → granular read limits on model endpoints
- **Capability escalation** → no ambient authority, monotonicity enforced
- **Agent-to-agent injection** → mediated inter-agent messaging
- **Deep capability chains** → bounded derivation depth
- **KV cache side channel** → memory isolation between agent contexts
- **TEE side channel** → interface for TEE lifecycle management
- **Model supply chain** → attestation-aware capability delegation

---

## 2. Model Analysis

### 2.1 Linux Syscall ABI

**Architecture:** ~450 syscalls, integer-numbered, parameters in registers. Process-centric namespace (PIDs, FDs). Access control via UID/GID + capabilities (Linux capabilities, not object capabilities).

**Security:** 6/10 — Ambient authority (UID/GID). Coarse-grained. No confinement without LSM (SELinux, AppArmor). Capability escalation via setuid/setcap.

**Performance:** 10/10 — Native execution, minimal overhead (~100 cycles for simple syscall).

**Scalability:** 4/10 — Global namespaces (PID, mount) are scalability bottlenecks.

**Verifiability:** 2/10 — 30M+ LOC kernel, no formal verification at scale.

**Capability safety:** 2/10 — Linux capabilities are POSIX capabilities, not object capabilities. No monotonicity, no delegation without setuid.

**Sandboxing:** 5/10 — Possible with seccomp-bpf, namespaces, but complex and porous.

**Distributed execution:** 2/10 — No native support; requires network layers.

**AI integration:** 2/10 — No concept of intent or agent identity.

**Human usability:** 7/10 — Familiar to millions of developers.

**Backward compatibility:** 10/10 — Decades of stable ABI.

**Future extensibility:** 3/10 — Adding syscalls is heavyweight (arch-specific, libc wrappers, etc.).

### 2.2 Windows NT Native API

**Architecture:** "System services" accessed via `syscall` instruction. Object manager-based: all resources are objects with ACLs. Handle-based access.

**Security:** 7/10 — Object-based ACLs are finer than POSIX. Handles provide a capability-like pattern. However, ambient authority via tokens.

**Performance:** 9/10 — Well-optimized, though syscall-heavy.

**Scalability:** 5/10 — Object namespace is global per session.

**Verifiability:** 2/10 — Millions of lines, no formal verification.

**Capability safety:** 4/10 — Handles are capability-like but backed by tokens with ambient rights.

**Sandboxing:** 6/10 — AppContainer, Integrity Levels, but complex to configure.

**AI integration:** 2/10 — No AI-native primitives.

**Assessment:** Evolutionary improvement over Linux but fundamentally same paradigm.

### 2.3 Mach Message Passing

**Architecture:** Microkernel with IPC as fundamental primitive. Messages sent to ports. Ports are capabilities. Memory can be mapped via "memory objects."

**Security:** 7/10 — Port-based capability model. No ambient authority.

**Performance:** 4/10 — Historically slow IPC (context switches for every message). Large message copies.

**Verifiability:** 4/10 — Simpler than monolithic but no formal verification achieved.

**Capability safety:** 7/10 — Port rights are capabilities. Send-once rights.

**Historical significance:** Influenced macOS (XNU) and GNU Hurd. Proved microkernel IPC could work but performance issues (Liedtke's critique).

### 2.4 seL4 IPC and Capability Invocation

**Architecture:** Endpoint-based IPC. Capabilities are first-class kernel objects in a capability space (CSpace). Badged endpoints for identity. Capability transfer embedded in IPC messages.

**Security:** 10/10 — Formally verified capability model. No ambient authority. Monotonic capability derivation.

**Performance:** 9/10 — Fastest microkernel IPC: 367 cycles (Arm), 770 cycles (x86_64). Fastpath for small messages.

**Scalability:** 7/10 — MCS extension adds time capabilities. Multi-core verified version exists.

**Verifiability:** 10/10 — Complete functional correctness proof (seL4). 9,000 LOC kernel.

**Capability safety:** 10/10 — Full object-capability model. Badge-based identity. Capability derivation with attenuation.

**Sandboxing:** 9/10 — Protection domains with private CSpace and address space. IOMMU integration.

**Distributed execution:** 5/10 — Primarily single-machine. CAmkES for component architecture.

**AI integration:** 4/10 — No AI primitives but suitable substrate.

**Human usability:** 3/10 — Steep learning curve. Manual CSpace management without CAmkES.

**Backward compatibility:** 2/10 — Not POSIX-compatible without LionsOS or similar.

**Assessment:** Best foundation. The verified core is irreplaceable. But pure seL4 API is too low-level for AI agents.

### 2.5 CHERI Capabilities

**Architecture:** Hardware-enforced capabilities in the ISA. Every register widened to 2× XLEN with metadata (bounds, permissions, type). Tagged memory for integrity. Monotonicity enforced in hardware.

**Security:** 10/10 — Hardware-enforced fine-grained memory protection. Software compartmentalization at function granularity. Proves: spatial memory safety, mitigates ~2/3 of CVEs.

**Performance:** 9/10 — 0-5% overhead for memory safety. Compartment IPC 90% faster than MMU-based.

**Scalability:** 8/10 — In-address-space capabilities scale to millions of objects.

**Verifiability:** 9/10 — Formal ISA models in Sail. Mechanized proofs of security properties.

**Capability safety:** 10/10 — Full monotonicity: permissions can only be shrunk, bounds can only be narrowed.

**Sandboxing:** 10/10 — Sub-object granularity. C/C++ compatible recompilation.

**Distributed execution:** 3/10 — Single-address-space model primarily.

**AI integration:** 4/10 — No AI primitives but excellent substrate for fine-grained memory safety.

**Deployment:** Industrial — Arm Morello (7nm), Microsoft CHERIoT (taped out), Codasip, Google AI accelerators.

**Assessment:** Essential hardware substrate for AINOS. CHERI-RISC-V (RVY) as primary target ISA enables hardware-enforced capabilities at the instruction level.

### 2.6 Capsicum (FreeBSD)

**Architecture:** Lightweight hybrid capability framework over POSIX. Two primitives: (1) capability mode — enter sandbox, lose global namespace access; (2) capability rights — refine FD operations via `cap_rights_limit()`. Casper service for controlled escalation.

**Security:** 7/10 — Strong confinement (no global namespace access). Hybrid model means historical bugs (CVE-2026-45259: sigqueue bypass) from POSIX compatibility.

**Performance:** 9/10 — Minimal overhead for capability mode.

**Adoption:** Shipped in FreeBSD since 9.0. Chromium sandbox.

**Capability safety:** 6/10 — File descriptors as capabilities but POSIX legacy leaks (e.g., procfs).

**Assessment:** Pragmatic migration path but insufficient for AINOS. The hybrid nature creates edge cases.

### 2.7 Barrelfish Multikernel

**Architecture:** Multikernel — separate OS instance per core, communicating via explicit messages. Flounder IDL for interface definitions. Monitor for cross-core coordination. Capability system with per-core state and distributed agreement protocols.

**Security:** 7/10 — Capability-based resource management. Monitor as trusted coordinator.

**Performance:** 7/10 — URPC competitive with L4 IPC. 2-cycle cross-HyperTransport message latency.

**Scalability:** 9/10 — No shared state. Designed for heterogeneous many-core.

**Verifiability:** 4/10 — Capability system has formal model but no full verification.

**Capability safety:** 8/10 — Well-designed capability system with ownership tracking.

**Distributed execution:** 9/10 — Natural fit: the machine IS a distributed system.

**AI integration:** 4/10 — Message-passing model maps to agent communication but needs semantic layer.

**Assessment:** Important architectural ideas (explicit communication, hardware-neutral structure, state replication). The message-passing IDL concept is valuable for service interfaces.

### 2.8 Fuchsia Zircon

**Architecture:** Microkernel with handle-based capabilities. All kernel objects accessed via handles with rights. Channels for IPC with handle transfer capability. VMO for memory sharing. Capability routing through component framework.

**Security:** 8/10 — Handle rights are checked on every syscall. Capability routing prevents ambient access. Strict hierarchy.

**Performance:** 7/10 — Channel IPC ~8K cycles (by Mi et al. 2019 measurement). Slower than seL4.

**Scalability:** 7/10 — Namespace-per-component prevents global bottlenecks.

**Verifiability:** 3/10 — No formal verification of Zircon.

**Capability safety:** 8/10 — Handle-based capabilities with rights limiting. To be deprecated in favor of capability routing.

**AI integration:** 4/10 — Component framework supports sandboxing but no AI primitives.

**Assessment:** Well-engineered capability system but no formal verification. Channel IPC slower than seL4.

### 2.9 Redox Syscall Model

**Architecture:** Rust microkernel. Minimal syscall interface resembling seL4: message passing, scheme registration, interrupt registration. No file-specific syscalls. Namespace manager (nsmgr) in userspace. CWD as capability. Bulk syscalls for batching.

**Security:** 8/10 — Capability-based with userspace namespace manager. Scheme-level isolation.

**Performance:** 7/10 — Bulk syscalls mitigate microkernel overhead. Rust safety reduces bugs.

**Verifiability:** 5/10 — No formal verification but Rust memory safety.

**Capability safety:** 7/10 — Namespace as capability. FD-based capabilities. Still evolving (FOSDEM 2026 presentation).

**Assessment:** Closest existing Rust microkernel to AINOS's kernel vision. The scheme-and-namespace model is directly relevant.

### 2.10 WASI/WebAssembly Component Model

**Architecture:** Component Model as "microkernel" for portable code. WASI as "OS services" on top. Interface types (WIT) for component interfaces. Canonical ABI for cross-boundary value transfer. Native async as of WASI 0.3 (June 2026). Stream splicing for zero-copy.

**Security:** 8/10 — Shared-nothing by default. Explicit imports/exports. No ambient authority. Capability-based resource access (WASI preview 2).

**Performance:** 8/10 — Near-native with WASM. Microservice chaining nanoseconds vs milliseconds. JIT compilation.

**Scalability:** 8/10 — Component composition in-process. Service chaining without network.

**Verifiability:** 6/10 — Formal semantics for core WASM. Component Model spec progressing.

**Capability safety:** 7/10 — Capability-based resource model. Components only access what's explicitly imported.

**Sandboxing:** 9/10 — Shared-nothing isolation. Linear memory sandbox.

**Distributed execution:** 9/10 — Component model designed for distribution. WASI 0.3 with middleware world.

**AI integration:** 6/10 — Wasm is used for AI model execution (llama.wasm, etc.). Component model enables safe plugin architectures.

**Assessment:** WASI Component Model is a second-system effect: it implements an OS-interface abstraction *within* an OS. For AINOS, components are a valuable sandboxing layer for untrusted agent code but not a replacement for the kernel interface.

### 2.11 AgenticOS Intent ABI

**Architecture:** Four-layer vertical design: Ghost Kernel (minimal trusted substrate, no syscalls exposed to agents) → Logic Shutter (intent recognition, policy mediation, capability tokens) → Agent Capsule (restricted runtime, Manifest-Only) → Semantic Boundary Gateway (external protocol proxy). Intent ABI replaces syscalls with structured semantic operations. Weaver generates dynamic capabilities from Manifest.

**Security:** 9/10 — Intent filter paradigm prevents compositional escapes. Manifest-Only Runtime: no undeclared capability exists. Ghost Kernel unreachable from agents.

**Performance:** 5/10 (estimated) — Deep mediation stack. Intent parsing overhead. Manifest verification at startup. No benchmarks available.

**Capability safety:** 9/10 — Dynamic capability generation from manifest. Non-generalizable interfaces (composition closure cannot degenerate into syscall table). Skill admission process prevents escalation.

**Sandboxing:** 9/10 — Manifest-enforced resource boundaries. No corresponding interfaces for undeclared capabilities. Semantic Boundary Gateway prevents raw protocol access.

**AI integration:** 10/10 — Designed specifically for AI agents. Intent declarations align with LLM task planning. Semantic operations match agent mental model.

**Human usability (agent developer):** 9/10 — "Declare intent, get capabilities" rather than "request specific resources."

**Backward compatibility:** 3/10 — Requires rewrites. No POSIX compatibility.

**Assessment:** The most innovative ABI for AI agents. The Intent ABI's "non-generalizable" property directly addresses the compositional escape problem. However: (1) unproven in real hardware, (2) deep mediation adds latency, (3) no formal verification yet. Best suited as the *agent-level* interface layer, not the kernel interface.

### 2.12 MCP (Model Context Protocol)

**Architecture:** JSON-RPC-based protocol for LLM tool access. Host → Client → Server. Resources, Prompts, Tools as primitives. Stateless as of 2026-07-28. Extensions framework (Tasks, MCP Apps). OAuth/OpenID Connect auth.

**Security:** 7/10 — Authorization framework. Tool-level capability isolation. However: JSON-RPC, no capability monotonicity.

**Performance:** N/A — Network protocol, not OS interface.

**AI integration:** 10/10 — Designed specifically for LLM agent tool use. Standardized in AI industry (Anthropic, OpenAI, etc.).

**Assessment:** MCP is the industry standard for *remote* agent-tool interaction. Not an OS interface but the interface AINOS should serve *as* an MCP host/server.

### 2.13 A2A (Agent-to-Agent Protocol)

**Architecture:** Google's protocol for agent-to-agent communication. Task-oriented with skill cards, agent cards. Focuses on inter-agent coordination, not OS interface.

**Relevance:** Defines patterns for agent discovery, capability advertisement, task delegation. AINOS should natively support these patterns at the OS level rather than requiring a protocol layer.

### 2.14 Zircon Channels (Additional Detail)

Fuchsia's key IPC primitive. Datagram-oriented. Max message size: ZX_CHANNEL_MAX_MSG_BYTES. Handles embedded in messages for capability transfer. Rights checked on read/write. `zx_channel_write_etc` for rights-limited transfers.

### 2.15 Comparison Summary

| Model | Security | Perf | Scal | Verif | CapSafe | Sandbox | Dist | AI | Usability | Compat | Future |
|-------|----------|------|------|-------|---------|---------|------|----|-----------|--------|--------|
| Linux syscall | 6 | 10 | 4 | 2 | 2 | 5 | 2 | 2 | 7 | 10 | 3 |
| Windows NT | 7 | 9 | 5 | 2 | 4 | 6 | 3 | 2 | 6 | 9 | 3 |
| Mach IPC | 7 | 4 | 5 | 4 | 7 | 6 | 4 | 3 | 4 | 5 | 4 |
| **seL4 IPC** | **10** | **9** | **7** | **10** | **10** | **9** | **5** | 4 | 3 | 2 | **9** |
| CHERI | **10** | **9** | 8 | **9** | **10** | **10** | 3 | 4 | 5 | 6 | **10** |
| Capsicum | 7 | **9** | 6 | 3 | 6 | 7 | 3 | 2 | 7 | 8 | 4 |
| Barrelfish | 7 | 7 | **9** | 4 | 8 | 7 | **9** | 4 | 4 | 3 | 7 |
| Fuchsia Zircon | 8 | 7 | 7 | 3 | 8 | 8 | 6 | 4 | 5 | 4 | 6 |
| Redox | 8 | 7 | 6 | 5 | 7 | 7 | 4 | 4 | 5 | 5 | 7 |
| WASI Component | 8 | 8 | 8 | 6 | 7 | **9** | **9** | 6 | 7 | 5 | 8 |
| **AgenticOS Intent** | **9** | 5 | 6 | 4 | **9** | **9** | 5 | **10** | **9** | 3 | **9** |
| MCP | 7 | N/A | N/A | N/A | 5 | 6 | 8 | **10** | **9** | 5 | **9** |

---

## 3. Recommendation: Three-Layer Hybrid Architecture

No single interface model satisfies all requirements. The recommendation is a **three-layer hybrid** where each layer serves a distinct purpose:

### Layer 1: Kernel Capability Invocation (seL4-derived)

**Scope:** Resource access, memory management, IPC setup, scheduling, interrupt handling.
**Model:** Pure capability invocation — all operations require presenting a capability.
**Design:**
- Endpoint-based IPC for cross-PD communication
- Capability space (CSpace) per protection domain
- Badged endpoints for identity-based access control
- Capability transfer embedded in IPC messages (like seL4)
- Kernel TCB: capability manager, IPC router, memory manager, scheduler, interrupt handler
- NO traditional syscalls; NO file descriptors; NO integer PIDs

**Why not pure seL4:** The seL4 API is too low-level for AI agents. Agents should not manually manage CSpace slots.

### Layer 2: Service Message Passing (Fuchsia/Barrelfish-inspired)

**Scope:** Inter-service communication, filesystem, networking, device access.
**Model:** Typed message passing with interface contracts (WIT/Flounder-style).
**Design:**
- Service interfaces defined in an IDL (WIT-based)
- Services communicate via channels/endpoints
- Interface types enable static verification of message contracts
- Capability delegation via channel handle transfer
- User-space service manager for service discovery and routing

**Why this layer:** Raw capability invocation lacks type safety and interface contracts. This layer provides the "OS services" interface without exposing kernel details.

### Layer 3: Intent ABI (AgenticOS-inspired)

**Scope:** AI agent task execution, tool use, resource access based on intent.
**Model:** Manifest-declared intent → capability synthesis → mediated execution.
**Design:**
- Agents submit structured intent declarations (Manifest) at startup
- Weaver (user-space service) synthesizes interface table from Manifest
- Logic Shutter mediates every intent invocation against policy
- Semantic gateway proxies external protocols
- NO raw syscalls, NO raw file access, NO raw network from agent context
- Interface set is non-generalizable (compositional closure != syscall table)

**Why this layer:** AI agents need semantic safety, not resource-level access. The Intent ABI prevents the "compositional escape" problem where agents combine benign primitives into dangerous behavior.

### Interaction Rules

| Layer | Accessed By | Accessible From | Example |
|-------|------------|----------------|---------|
| 1: Capability Invocation | User-space services, drivers | Layer 2 servers | `cap_invoke(endpoint, msg, caps)` |
| 2: Message Passing | User-space applications | Most processes | `service_call(filesystem, "read", path)` |
| 3: Intent ABI | AI agent runtimes | Agent capsules only | `intent_call("summarize", {document})` |

- An AI agent sees ONLY Layer 3 (Intent ABI)
- A traditional application sees Layers 1-2 (message passing over capabilities)
- System services see all three layers
- The kernel sees only Layer 1

### Mapping to Existing Models

- Layer 1 is seL4-style capability invocation
- Layer 2 is Fuchsia-style component messaging
- Layer 3 is AgenticOS-style Intent ABI
- CHERI capabilities HARDEN all three layers at the hardware level
- WASI components run AS an agent capsule runtime (Layer 3 consumer)
- MCP/A2A protocols run THROUGH the Semantic Boundary Gateway (Layer 3)
