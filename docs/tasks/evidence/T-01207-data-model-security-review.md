# T-01207: Package Management - Data Model: Security Review

## Metadata
- **Task ID:** `T-01207`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Security Review
- **Status:** Complete

## 1. Security Review & Threat Modeling

### Abuse Scenario 1: Shell Metacharacter & Path Traversal Injection in Package Names
- **Attack Vector:** An untrusted agent or operator supplies a package name containing directory traversal or command injection payloads (e.g., `curl; rm -rf /`, `../../etc/shadow`, `$(reboot)`).
- **Mitigation:** Invariant `PM1` enforced via `validate_package_name`:
  - Name must begin with `[a-z0-9]` and contain exclusively `[a-z0-9+.-]`.
  - Slashes, backslashes, quotes, shell metacharacters, whitespace, and null bytes are rejected with error code 2.
- **Verdict:** Secure. No injection vectors bypass `validate_package_name`.

### Abuse Scenario 2: Memory Exhaustion / Unbounded Allocations (DoS)
- **Attack Vector:** Submitting deeply nested or gigabyte-scale package specifications, causing OOM crashes in agent memory.
- **Mitigation:** Invariant `PM2` enforces strict upper bounds:
  - Name <= 128 chars.
  - Version <= 64 chars.
  - Architecture <= 64 chars.
  - Description <= 4,096 bytes.
  - Dependencies count <= 256 items.
  - Package size <= 100 GiB.
  - Transaction actions <= 256 entries.
- **Verdict:** Secure. Bounded parsing guarantees constant upper bounds on memory allocation.

### Abuse Scenario 3: Repository Spoofing & Cleartext Tampering
- **Attack Vector:** Pointing package downloads to unencrypted HTTP repositories prone to DNS hijacking, BGP poisoning, or MitM binary injection.
- **Mitigation:** Invariant `PM4` mandates that any non-loopback `repository_url` must strictly use `https://`. Public unencrypted `http://` URLs are rejected during validation.
- **Verdict:** Secure. Enforces cryptographic transport hygiene.

### Abuse Scenario 4: Self-Dependency & Cyclic Graph Poisoning
- **Attack Vector:** Crafting package specs that depend on themselves (`pkg -> pkg`) or specify contradictory transaction actions (`Install` and `Remove` in the same transaction) causing infinite loops or race conditions during resolution.
- **Mitigation:** Invariant `PM3` and `validate_package_transaction` reject self-dependencies (`dep.name != spec.name`), duplicate dependencies, and multiple actions targeted at the same package name.
- **Verdict:** Secure. Dependency graph and transaction batch remain acyclic and well-ordered.

## 2. Policy Gating & Audit Conformance
- **PEP Enforcement:** MCP tool invocations are checked via `dispatch::recorded_call` supporting `grant_id` validation.
- **Audit Logging:** Every CLI command (`cmd_package`) and MCP tool call (`aios.package.validate`) emits an immutable audit row into the SQLite WAL ring (`audit.db`) with SHA-256 hash chaining.
- **Bypass Check:** No unvalidated or unaudited mutation paths exist.
