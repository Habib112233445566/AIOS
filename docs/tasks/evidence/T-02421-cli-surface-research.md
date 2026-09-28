# T-02421: Sandbox Enforcement CLI Surface Research

## 1. Executive Summary & Objective
This research document establishes the user-facing command-line interface (CLI) requirements, ergonomics, POSIX arguments parsing conventions, and prior art for **Sandbox Enforcement** within the AIOS CLI (`aiosh sandbox`).

---

## 2. Analysis of Existing Codebase & Prior Art

### A. Current CLI Surface in `code/aiosh-rust/aiosh-cli`
1. **`aiosh run` (`cmd_run`)**:
   - Spawns command with Landlock rules and seccomp denylist.
   - Accepts only `--target <dir>`.
   - Lacks profile selection (`--profile`), resource limits, capabilities inspection, and explicit sandbox management.
2. **`aiosh-sandbox` standalone binary**:
   - Takes `--profile <name_or_json>` and executes commands.
   - Provides low-level execve wrapper, but not integrated into the multi-command `aiosh` shell binary with audit context, PEP verification, and JSON formatting.
3. **Established CLI Subcommand Patterns in `aiosh`**:
   - Subcommands like `aiosh pep`, `aiosh audit`, `aiosh capability`, and `aiosh session` provide structured subcommands (`list`, `show`, `probe`, `exec`) with `--json` output flags and standard return codes (0 for success, 1 for failure, 2 for usage errors).

---

## 3. Fact vs. Assumption Separation

| Topic | Verified Fact | Working Assumption |
|---|---|---|
| **Delimiter Parsing** | Subprocess commands often contain their own dashes (e.g., `ls -la`, `curl -s`). | `aiosh sandbox exec` must support the standard `--` argument delimiter to separate sandbox options from child command arguments. |
| **Exit Code Forwarding** | Shell callers expect the exit code of `aiosh sandbox exec` to match the child process exit code. | Return the child's exit code on termination (e.g. 0 on success, 127 for not found, 128+signal on kill). |
| **Audit Requirement** | Every state-changing CLI execution must write an audit row. | `aiosh sandbox exec` must record an audit row with `tool="sandbox"` and attach the resulting hash. |

---

## 4. Proposed CLI Command Taxonomy

1. **`aiosh sandbox profiles [--json]`**:
   - Lists registered containment profiles (`standard`, `strict`, `permissive`, etc.), their resource limits, and filesystem rules.
2. **`aiosh sandbox probe [--json]`**:
   - Probes the host OS kernel and reports Landlock LSM support, ABI version, Seccomp-BPF status, and process privilege status.
3. **`aiosh sandbox exec [--profile <name>] [--grant <id>] [--cwd <path>] -- <bin> <args...>`**:
   - Executes a command under the specified profile with watchdog timeout, environment filtering, and audit logging.

---

## 5. Unknowns & Decisions Needed
- **Q1: How should `--profile` default?**
  - *Decision*: Default to `standard`, which provides balanced Landlock filesystem containment and Seccomp-BPF denylisting without breaking standard tool execution.
- **Q2: Should `aiosh sandbox` be accessible via aliases?**
  - *Decision*: Support `aiosh sandbox` and `aiosh sb` aliases in the main dispatch router.
