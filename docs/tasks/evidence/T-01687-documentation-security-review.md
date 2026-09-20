# T-01687: Kernel Module Documentation Security Review

## Sub-Epic
Kernel Module Management / Documentation (T-01687)

## Overview
Comprehensive security review and threat modeling of the Kernel Module Management Documentation subsystem (`KernelModuleDocIndex`, `aiosh mod doc`, and `aios.kernel_module.doc`).

## Attack Surfaces Evaluated
1. **In-Memory Registry**: `aiosh_core::kernel_module_doc::KernelModuleDocIndex`.
2. **CLI Operator Subcommands**: `aiosh mod doc [list|get|search]`.
3. **MCP Tool Surface**: `aios.kernel_module.doc`.
4. **Markdown Rendering Engine**: `format_topic_markdown`.

## Threat Analysis & Abuse Scenarios (DS-01..DS-06)

### DS-01: Terminal Injection & Escape Sequences
- **Threat:** User-supplied queries, topic IDs, or markdown content containing ANSI escape codes or control characters attempting terminal manipulation or deceptive prompt spoofing.
- **Analysis:** All CLI outputs pass through `sanitize_terminal` before terminal emission. Control characters and escape codes are stripped.
- **Verdict:** SAFE.

### DS-02: Algorithmic Complexity & ReDoS Exhaustion
- **Threat:** Massive search strings causing high CPU or memory consumption during topic search.
- **Analysis:** `search()` utilizes case-insensitive linear substring matching (`contains`) and token matching rather than unconstrained regular expressions. To guarantee deterministic bounded execution, a maximum query length (256 characters) will be enforced in T-01688 hardening.
- **Verdict:** ACTION ITEM for T-01688.

### DS-03: Arbitrary File System Reads & Path Traversal
- **Threat:** Callers supplying path arguments (`../../etc/shadow`) to `get_topic`.
- **Analysis:** `KernelModuleDocIndex` is statically compiled in memory. Topic lookups are matched against in-memory topic identifiers (`id`). Zero file system read calls are executed.
- **Verdict:** SAFE.

### DS-04: Non-Root Execution Safety
- **Threat:** Unprivileged agents or users attempting kernel state mutation through documentation commands.
- **Analysis:** Documentation commands are strictly read-only and passive. Zero kernel syscalls (`init_module`, `delete_module`) are performed. Unprivileged execution is guaranteed.
- **Verdict:** SAFE.

### DS-05: Information Disclosure & Host Fingerprinting
- **Threat:** Documentation leaking host architecture, private cryptographic keys, or active kernel addresses.
- **Analysis:** All canonical topics contain public, vendor-neutral educational references (Linux man pages, CIS distribution benchmarks, and modprobe syntaxes). No host-specific runtime telemetry or secrets are stored.
- **Verdict:** SAFE.

### DS-06: Audit Record Provenance & Integrity
- **Threat:** Unlogged queries or unescaped malicious strings injected into the SQLite audit ring.
- **Analysis:** CLI operations invoke `classify_and_emit` and MCP operations invoke `dispatch::recorded_call`, preserving structured JSON logs with audit hash-chaining.
- **Verdict:** SAFE.

## Hardening Recommendations for T-01688
1. Enforce a 256-character length ceiling on `query` and `topic_id` arguments.
2. Enforce an upper bound of 50 results returned from `KernelModuleDocIndex::search` to prevent unbounded serialization overhead.
3. Reject control characters in search query inputs.
