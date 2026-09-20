# T-01681: Kernel Module Management Documentation Research

## Sub-Epic
Kernel Module Management / Documentation (T-01681)

## Objective
Research authoritative documentation, standards, verification criteria, and search semantics for the Kernel Module Management Documentation subsystem in AIOS.

## Research Findings

### 1. Authoritative Linux & Security Standards
- **Linux Manual Pages**:
  - `modprobe(8)` & `modprobe.d(5)`: Governs module resolution, dependency trees, and directives (`alias`, `blacklist`, `install`, `options`, `remove`, `softdep`).
  - `modules-load.d(5)`: Governs boot-time autoloading under `/etc/modules-load.d/` and `/run/modules-load.d/`.
  - `lsmod(8)` & `proc(5)`: Details `/proc/modules` formatting (Name, Size, Refcount, Dependents, State, Load Address).
  - `modinfo(8)`: Extracts ELF metadata (`license`, `author`, `description`, `version`, `vermagic`, `parm`).
  - `sysctl(8)`: `kernel.modules_disabled` (prevents any further module loading once set to 1).
- **CIS Distribution Benchmarks**:
  - Benchmark §1.1.1: Filesystem module disabling (`cramfs`, `freevxfs`, `jffs2`, `hfs`, `hfsplus`, `udf`).
  - Benchmark §3.4: Rare network protocols (`dccp`, `sctp`, `rds`, `tipc`).
- **Kernel Security Infrastructure**:
  - `CONFIG_MODULE_SIG`: Cryptographic signature checking for kernel modules.
  - Kernel Lockdown LSM: Integrity mode disables unsigned module loading.

### 2. Architectural Design of Documentation Subsystem
- **Component**: `aiosh_core::kernel_module_doc`
- **Primary Abstractions**:
  - `KernelModuleDocIndex`: In-memory registry containing canonical topics, categories, and references.
  - `DocTopic`: Structured document containing title, category, summary, sections, tags, examples, and citations.
  - `DocSearchResult`: Scored match containing relevance score, topic ID, title, and matching snippet.
- **Access Vectors**:
  - Operator CLI: `aiosh mod doc list`, `aiosh mod doc get <topic>`, `aiosh mod doc search <query>`.
  - Agent MCP Tool: `aios.kernel_module.doc` (methods: `list`, `get`, `search`).

### 3. Verification Criteria (KD1..KD6)
- **KD1**: Offline Self-Contained Registry — No external network calls; statically embedded in `aiosh-core`.
- **KD2**: Authoritative References — Every topic must cite authoritative man-pages, kernel docs, or CIS benchmark sections.
- **KD3**: Deterministic Search & Ranking — Exact topic ID matches rank highest, followed by tag matches, title matches, and content matches.
- **KD4**: Multi-Format Rendering — Supports human-readable Markdown terminal output and structured JSON for automated pipelines.
- **KD5**: Memory Bounds & Safe Lookups — Bounded topic store with case-insensitive, sanitized lookups.
- **KD6**: Cross-Surface Consistency — Uniform results across Rust library, CLI, and MCP surfaces.
