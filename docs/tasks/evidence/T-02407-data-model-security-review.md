# T-02407: Sandbox Enforcement Data Model Security Review

## 1. Threat Modeling & Scope
A formal security review was conducted on `code/aiosh-rust/aiosh-core/src/sandbox_data_model.rs` and its integration surfaces in `sandbox.rs` and `aiosh-sandbox/src/main.rs`.

## 2. Abuse Scenarios & Mitigations

### Abuse Scenario A: Directory Traversal via Path Injection
- **Vector**: An untrusted caller injects paths such as `/proc/../etc/shadow` or relative traversals (`../../../root/.ssh`) into `paths_ro` or `paths_rw`.
- **Mitigation**: `FilesystemPolicy::validate()` recursively inspects all paths across `paths_ro`, `paths_rw`, `paths_execute`, and `paths_denied` and rejects any path containing the substring `..` with `ERR_SANDBOX_INVALID_PATH`.
- **Verdict**: Mitigated.

### Abuse Scenario B: Ambiguous / Conflicting Access Grants
- **Vector**: A profile specifies a sensitive path in both `paths_ro` and `paths_rw`, or in both `paths_denied` and `paths_rw`, hoping to exploit order-of-evaluation flaws in the kernel Landlock builder.
- **Mitigation**: `FilesystemPolicy::validate()` checks set disjointness across access categories and returns `ERR_SANDBOX_POLICY_CONFLICT` if an overlap is detected.
- **Verdict**: Mitigated.

### Abuse Scenario C: Resource Starvation / DoS via Bound Manipulation
- **Vector**: An adversarial payload requests zero wall-clock time (causing panic/hang) or 100 TB of RAM allocation.
- **Mitigation**: `ResourceLimits::validate()` rigorously enforces lower and upper bounds:
  - Memory: [1 MB, 64 GB]
  - Wall-clock timeout: [10 ms, 3,600,000 ms]
  - Process count: [1, 4096]
  - Open file descriptors: [16, 65536]
- **Verdict**: Mitigated.

### Abuse Scenario D: Subprocess Command Hijacking
- **Vector**: Passing an empty or whitespace command string to trigger shell execution defaults or undefined behavior in `execv`.
- **Mitigation**: `SandboxExecutionRequest::validate()` checks `command.trim().is_empty()` and returns `ERR_SANDBOX_EMPTY_COMMAND`.
- **Verdict**: Mitigated.

### Abuse Scenario E: PEP Authorization Decoupling
- **Vector**: Executing a sandboxed command without provenance tracking or binding to an active capability grant.
- **Mitigation**: Data model explicitly captures `pep_grant_id` and `session_id`, ensuring downstream core services can enforce PEP token gating and write auditable causation chains.
- **Verdict**: Mitigated.

## 3. Review Conclusion
No unresolved vulnerabilities or policy bypasses exist. The data model enforces defensive-in-depth sanitization at the type boundary.
