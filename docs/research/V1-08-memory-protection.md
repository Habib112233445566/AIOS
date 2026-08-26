> **V2 AMENDMENT (2026-08-20):** The product vision has been restated. AIOS is *"a Linux system for ethical hacking on the inside, a Windows-style desktop on the outside, with AI as a first-class S-rank kernel subsystem."* Three pillars: Pillar A (Linux ethical-hacking), Pillar B (Windows-like desktop), Pillar C (S-rank AI subsystem). This research file is preserved as **research substrate** informing userspace capability / IPC / scheduler / PEP / MCP designs; it is no longer the shipping-path definition. See `README.md` and `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0).

# Research Report V1.08: Memory Protection Architecture

**Volume:** V1 — Foundations  
**Status:** Draft  
**Version:** 1.0  
**Date:** 2026-07-16  

---

## Executive Summary

This report investigates memory protection models for AINOS from first principles. The recommendation is a **CHERI-centric hybrid memory model**: hardware capabilities for fine-grained spatial safety and compartmentalization, PICASSO colored capabilities for temporal safety, traditional page tables for virtual memory abstraction, and GPU/NPU memory isolation via SMMU/IOMMU. KV cache side channels are mitigated at the serving runtime layer.

---

## 1. First-Principles Derivation

### 1.1 What Must Memory Protection Provide?

1. **Isolation** — PD A cannot read/write PD B's memory without authorization
2. **Controlled sharing** — PDs can grant specific memory access to others
3. **Spatial safety** — accesses within bounds of allocated region only
4. **Temporal safety** — no use-after-free, no double-free
5. **Granularity** — protection at page level (minimum), capability level (ideal)
6. **Performance** — protection must not dominate execution cost
7. **Verifiability** — protection invariants must be machine-checkable
8. **AI workload support** — GPU memory, NPU memory, KV cache isolation

### 1.2 Memory Protection Options

| Model | Mechanism | Granularity | Overhead | Verified | AI Support |
|-------|-----------|-------------|----------|----------|------------|
| Page tables (MMU) | HW page walk | 4KB pages | ~100 cycles (TLB miss) | No | Limited |
| CHERI capabilities | HW cap check | Byte-granular | 0-5% | Yes (T-CHERI) | No GPU support |
| MPK/PKU | PTE key + PKRU | Page-level | ~20 cycles (WRPKRU) | No | Limited |
| MPK (PKK, kernel) | PKU in kernel | Page-level | ~20 cycles | No | N/A |
| Software fault isolation | Bounds checking | Word-level | 5-15% | Partial | Limited |
| TEE (SGX/SEV) | HW encryption | Enclave | 5-20% | Partial | Yes (CC) |
| HMM/SMMU | IOMMU page tables | Page-level | Variable | No | Yes |

---

## 2. Model Analysis

### 2.1 CHERI Capabilities (Primary)

**Architecture (CHERI ISAv9, RVY):**
- Capabilities as hardware primitives: 2× XLEN registers with bounds, permissions, type tag
- Tagged memory: 1-bit tag per aligned YLEN region, hardware-managed
- Monotonicity: permissions can only shrink, bounds can only narrow
- Sealing: capabilities can be sealed against modification, providing immutable tokens
- Special capabilities: Root (Infinite), PCC (program counter capability), DDC (default data capability)

**Temporal safety (PICASSO, 2026):**
- Colored capabilities: add provenance identifier to otype field
- PVT (Provenance Validity Table): hardware-managed bit table
- On free: PVB invalidated → all dangling pointers instantly retracted
- No quarantine needed; memory immediately reusable
- Speculative out-of-order support (CHERI-Toooba)
- 5% geomean overhead on SPEC CPU

**Temporal safety (PoisonCap, 2026):**
- Poison capability format: strict use-after-free + initialization safety
- Hierarchical: capability bounds enable nested allocator privilege delegation
- Replaces Cornucopia shadow bitmap; automatic zeroing on reallocation

**Formal verification:** T-CHERI properties (Bauereiss et al.) — reachable capability monotonicity proven in Isabelle. Morello-Cerise extends to strong encapsulation proof with atomic invocation pairs.

**Compartmentalization:**
- In-address-space compartments: each library as compartment, cross-domain call overhead comparable to function call
- 90% less IPC overhead vs MMU-based compartmentalization
- Chromium prototype: memory-safe prototype as of late 2025

### 2.2 Page Tables (Virtual Memory)

**Architecture (seL4 model):**
- VSpace as top-level paging structure (PML4 on x86_64, PageTable on RISC-V)
- Intermediate paging structures (PDPT, PageDirectory, PageTable)
- Page capabilities: one frame capability per mapping
- Mapping with rights: read, write, execute, read-write
- RISC-V: page sizes variable (same object type), 4KB default

### 2.3 MPK/PKU (Fallback for x86_64)

**Architecture:**
- 4 bits in PTE → 16 keys
- PKRU register: 2 bits per key (AD, WD)
- WRPKRU: unprivileged instruction, ~20 cycles
- libmpk: arbitrary number of domains via key multiplexing

**Limitations:**
- Only 16 hardware keys
- Bypassable via readv/writev, ptrace, /proc/pid/mem
- No execute-only on Intel (feature on AMD Zen 3+)
- No formal verification

**PKK (Protection Keys for Kernelspace, IskiOS):**
- U/S bit set on all pages → PKU applies to kernel
- 8 keys for kernel, 8 for userspace
- Enables kernel XOM, shadow stacks

### 2.4 GPU/NPU Memory Isolation

**GHOST-ATTACK (2026):** GPU-originated exploitation via HMM — attacker GPU kernel can compromise host memory. SHELL defense: compiler + driver enforcement of shared-only access.

**CAMI (2026):** Context-aware GPU memory isolation — execution context binding to page ownership, hardware enforcement in MMU.

**MIGraine (2026):** MIG side channel via page fault contention — 96% accuracy fingerprinting LLMs across GPU instances. Affects A30, H100, both containerized and vGPU.

**GUARDAIN (2026):** NPU confidential computing — delegation-based memory semantics, SMMU-based isolation, no host trust required.

### 2.5 KV Cache Side Channels

**Problem:** Shared KV cache across tenants → timing side channel → attacker can reconstruct or infer user prompts.

**SafeKV (2025/2026):** Three-tier async detection (rule → classifier → LLM) → isolate sensitive KV blocks. Private-by-default, upgraded to shareable after verification. RDR safeguard for residual leakage.

**CachePrune (2026):** Fine-grained token-level sharing with sensitivity masking. 4.5x TTFT reduction vs full isolation.

**GeoCache (2026):** Orthogonal transformation of K/V matrices → provably lossless + computational content-level isolation. 0.014μs per head-token overhead. Operator rotation defeats KPA.

**KV-Cloak (2026):** Reversible matrix obfuscation with operator fusion. Reduces reconstruction to random noise.

---

## 3. Recommendation: CHERI-Centric Hybrid Memory Model

### Memory Protection Stack

| Layer | Mechanism | Scope | Basis |
|-------|-----------|-------|-------|
| 1. Capability safety | CHERI hardware capabilities | Intra-PD, in-address-space | T-CHERI verified |
| 2. Temporal safety | PICASSO colored caps → PoisonCap | Heap allocations | HW-enforced PVT |
| 3. Virtual memory | Page tables (MMU) | Inter-PD isolation | seL4 VSpace model |
| 4. Device memory | SMMU/IOMMU | GPU/NPU/PCIe | CAMI, SHELL |
| 5. Agent memory | Capsule-isolated + TEE | AI agent contexts | Per-capsule VSpace |
| 6. KV cache | SafeKV-style selective isolation | LLM inference | Detection + isolation |
| 7. Fallback | MPK (libmpk) | x86_64 non-CHERI | 16 keys, key multiplexing |

### Memory Model Rules

1. Every memory access requires a valid capability (CHERI)
2. Capability derivation is monotonic (permissions shrink only)
3. Freed memory → PVB invalidated → all stale capabilities trapped
4. Cross-PD memory requires explicit capability transfer (Layer 1 IPC)
5. Agent capsules have zero shared memory with other capsules by default
6. GPU memory: shared-only access via SHELL-enforced SMMU
7. KV cache: private-by-default, shareable only after SafeKV detection
